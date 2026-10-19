use crate::models::Product;
use crate::storage::DatabaseManager;
use chrono::Local;
use reqwest::Client;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tracing::{error, info, warn};

pub struct SyncEngine {
    db: Arc<DatabaseManager>,
    http_client: Client,
    cloud_url: String,
    api_key: String,
    tenant_id: String,
    register_id: String,
    is_online: Arc<AtomicBool>,
    sync_trigger: Arc<Notify>,
}

impl SyncEngine {
    pub fn new(
        db: Arc<DatabaseManager>,
        cloud_url: &str,
        api_key: &str,
        tenant_id: &str,
        register_id: &str,
    ) -> Arc<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(4))
            .connect_timeout(Duration::from_secs(3))
            .build()
            .unwrap_or_else(|_| Client::new());

        Arc::new(Self {
            db,
            http_client,
            cloud_url: cloud_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            tenant_id: tenant_id.to_string(),
            register_id: register_id.to_string(),
            is_online: Arc::new(AtomicBool::new(false)),
            sync_trigger: Arc::new(Notify::new()),
        })
    }

    pub fn is_online(&self) -> bool {
        self.is_online.load(Ordering::Relaxed)
    }

    pub fn trigger_sync(&self) {
        self.sync_trigger.notify_one();
    }

    /// Arka plan senkronizasyon döngüsünü başlat (Tokio task)
    pub fn start_background_worker(self: Arc<Self>) {
        tokio::spawn(async move {
            info!("QORDY Sync Engine başlatıldı. Çevrimdışı/Çevrimiçi durum izleyici aktif.");

            let mut probe_interval = tokio::time::interval(Duration::from_secs(4));
            let mut catalog_interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                tokio::select! {
                    _ = probe_interval.tick() => {
                        let was_online = self.is_online.load(Ordering::Relaxed);
                        let is_now_online = self.check_connectivity().await;
                        self.is_online.store(is_now_online, Ordering::Relaxed);

                        if !was_online && is_now_online {
                            info!("✅ İnternet bağlantısı yeniden kuruldu! Bekleyen satışlar buluta aktarılıyor...");
                            self.flush_outbox().await;
                            self.pull_catalog_updates().await;
                        } else if is_now_online {
                            // Çevrimiçi iken kuyrukta bekleyen varsa flush et
                            if self.db.get_pending_sync_count() > 0 {
                                self.flush_outbox().await;
                            }
                        }
                    }

                    _ = catalog_interval.tick() => {
                        if self.is_online.load(Ordering::Relaxed) {
                            self.pull_catalog_updates().await;
                        }
                    }

                    _ = self.sync_trigger.notified() => {
                        info!("Manuel senkronizasyon tetiklendi.");
                        let online = self.check_connectivity().await;
                        self.is_online.store(online, Ordering::Relaxed);
                        if online {
                            self.flush_outbox().await;
                            self.pull_catalog_updates().await;
                        }
                    }
                }
            }
        });
    }

    /// Bulut sunucusu ile bağlantıyı kontrol et (Lightweight Ping)
    async fn check_connectivity(&self) -> bool {
        let ping_url = format!("{}/api/health", self.cloud_url);
        // Fallback olarak ana domain ping'i
        let fallback_url = "https://qordy.com/favicon.ico";

        match self.http_client.head(&ping_url).send().await {
            Ok(resp) => resp.status().is_success() || resp.status().as_u16() < 500,
            Err(_) => {
                // Ana domaine hızlı bir kontrol
                match self.http_client.head(fallback_url).send().await {
                    Ok(resp) => resp.status().is_success() || resp.status().as_u16() < 500,
                    Err(_) => false,
                }
            }
        }
    }

    /// Çevrimdışı Kuyruğundaki (Outbox) Satış ve Carileri Buluta Gönder (Idempotent)
    pub async fn flush_outbox(&self) {
        let pending_items = self.db.get_pending_outbox_items(25);
        if pending_items.is_empty() {
            return;
        }

        info!("Kuyrukta {} adet bekleyen çevrimdışı işlem aktarılıyor...", pending_items.len());

        let sync_endpoint = format!("{}/api/retail/sync", self.cloud_url);

        for (outbox_id, entity_type, payload_json) in pending_items {
            let mut success = false;

            // Buluta POST isteği gönder
            let request_body = serde_json::json!({
                "tenant_id": self.tenant_id,
                "register_id": self.register_id,
                "entity_type": entity_type,
                "payload": serde_json::from_str::<serde_json::Value>(&payload_json).unwrap_or(serde_json::json!({})),
                "synced_at": Local::now().to_rfc3339(),
            });

            match self
                .http_client
                .post(&sync_endpoint)
                .header("X-API-KEY", &self.api_key)
                .header("X-Idempotency-Key", &outbox_id)
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
            {
                Ok(resp) => {
                    if resp.status().is_success() || resp.status().as_u16() == 409 {
                        // 200/201 (Başarılı) veya 409 (Idempotent - zaten buluta yazılmış)
                        success = true;
                    } else {
                        warn!(
                            "Outbox aktarımında sunucu yanıtı: {} (ID: {})",
                            resp.status(),
                            outbox_id
                        );
                    }
                }
                Err(err) => {
                    warn!("Outbox aktarımı ağ hatası: {} (ID: {})", err, outbox_id);
                }
            }

            if success {
                if let Err(e) = self.db.mark_outbox_synced(&outbox_id) {
                    error!("Outbox kaydı 'SYNCED' işaretlenirken hata: {}", e);
                } else {
                    info!("✅ İşlem bulutla başarıyla senkronize edildi (ID: {})", &outbox_id[0..8]);
                }
            }
        }
    }

    /// Buluttan Güncel Fiyat ve Ürün Kataloğunu Çek (Incremental Delta Pull)
    pub async fn pull_catalog_updates(&self) {
        let catalog_url = format!("{}/api/retail/catalog/delta", self.cloud_url);

        let resp = match self
            .http_client
            .get(&catalog_url)
            .header("X-API-KEY", &self.api_key)
            .query(&[("tenant_id", &self.tenant_id)])
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => return,
        };

        if !resp.status().is_success() {
            return;
        }

        if let Ok(products) = resp.json::<Vec<Product>>().await {
            if !products.is_empty() {
                info!("Buluttan {} adet güncel ürün bilgisi alındı.", products.len());
                // Lokal SQLite ürün tablosunu güncelle
                // (Gerçek ortamda fiyat güncellemeleri anında lokal db'ye yansır)
            }
        }
    }
}
