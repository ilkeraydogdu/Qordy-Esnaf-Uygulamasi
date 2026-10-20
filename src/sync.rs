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
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(3))
            .user_agent("QordyPos/3.0.0 (Desktop; Offline-First)")
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
        let probe_url = "https://qordy.com/api/mobile/auth/login";

        match self.http_client.get(probe_url).send().await {
            Ok(resp) => resp.status().as_u16() < 500,
            Err(_) => {
                let fallback_url = "https://qordy.com/favicon.ico";
                match self.http_client.get(fallback_url).send().await {
                    Ok(resp) => resp.status().as_u16() < 500,
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

        info!("Kuyrukta {} adet bekleyen çevrimdışı işlem buluta aktarılıyor...", pending_items.len());

        let cloud_token = self.db.get_app_config("cloud_token");

        for (outbox_id, entity_type, payload_json) in pending_items {
            let mut success = false;

            if let Some(ref token) = cloud_token {
                // Canlı Qordy Mobil/Perakende API Entegrasyonu
                if entity_type == "SALE" {
                    if let Ok(sale) = serde_json::from_str::<crate::models::Sale>(&payload_json) {
                        let items: Vec<serde_json::Value> = sale
                            .items
                            .iter()
                            .map(|item| {
                                serde_json::json!({
                                    "product_id": item.product_id,
                                    "quantity": item.quantity,
                                    "price": item.unit_price,
                                    "total": item.total
                                })
                            })
                            .collect();

                        let mut checkout_payload = serde_json::json!({
                            "items": items,
                            "payment_method": sale.payment_method,
                            "amount": sale.grand_total,
                            "note": format!("Kasa Fiş No: {}", sale.receipt_no),
                        });

                        if let Some(ref cid) = sale.customer_id {
                            if !cid.is_empty() {
                                checkout_payload["customer_id"] = serde_json::Value::String(cid.clone());
                            }
                        }

                        let res = self
                            .http_client
                            .post("https://qordy.com/api/mobile/retail/checkout")
                            .header("Authorization", format!("Bearer {}", token))
                            .header("Content-Type", "application/json")
                            .json(&checkout_payload)
                            .send()
                            .await;

                        match res {
                            Ok(resp) => {
                                if resp.status().is_success() || resp.status().as_u16() == 409 {
                                    success = true;
                                } else {
                                    warn!("Bulut retail checkout yanıtı: {} (ID: {})", resp.status(), outbox_id);
                                }
                            }
                            Err(e) => {
                                warn!("Bulut retail checkout bağlantı hatası: {} (ID: {})", e, outbox_id);
                            }
                        }
                    }
                } else if entity_type == "CUSTOMER_PAYMENT" {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&payload_json) {
                        let cust_id = val["customer_id"].as_str().unwrap_or("");
                        let amount = val["amount"].as_f64().unwrap_or(0.0);
                        let pay_method = val["payment_method"].as_str().unwrap_or("CASH");

                        let res = self
                            .http_client
                            .post("https://qordy.com/api/mobile/retail/customers/collect")
                            .header("Authorization", format!("Bearer {}", token))
                            .header("Content-Type", "application/json")
                            .json(&serde_json::json!({
                                "customer_id": cust_id,
                                "amount": amount,
                                "payment_method": pay_method
                            }))
                            .send()
                            .await;

                        match res {
                            Ok(resp) => {
                                if resp.status().is_success() || resp.status().as_u16() == 409 {
                                    success = true;
                                }
                            }
                            Err(e) => warn!("Bulut tahsilat bağlantı hatası: {}", e),
                        }
                    }
                }
            }

            // Standart yedek senkronizasyon endpoint'i
            if !success {
                let sync_endpoint = format!("{}/api/retail/sync", self.cloud_url);
                let request_body = serde_json::json!({
                    "tenant_id": self.tenant_id,
                    "register_id": self.register_id,
                    "entity_type": entity_type,
                    "payload": serde_json::from_str::<serde_json::Value>(&payload_json).unwrap_or(serde_json::json!({})),
                    "synced_at": Local::now().to_rfc3339(),
                });

                if let Ok(resp) = self
                    .http_client
                    .post(&sync_endpoint)
                    .header("X-API-KEY", &self.api_key)
                    .header("X-Idempotency-Key", &outbox_id)
                    .header("Content-Type", "application/json")
                    .json(&request_body)
                    .send()
                    .await
                {
                    if resp.status().is_success() || resp.status().as_u16() == 409 {
                        success = true;
                    }
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
        if let Some(token) = self.db.get_app_config("cloud_token") {
            let client = crate::cloud::QordyCloudClient::default();
            if let Ok(products) = client.fetch_all_retail_products(&token).await {
                if !products.is_empty() {
                    let _ = self.db.sync_cloud_catalog(&products);
                    info!("Canlı Qordy Bulut'tan {} ürün başarıyla senkronize edildi ve yerel veritabanı güncellendi.", products.len());
                }
            }
        }
    }
}
