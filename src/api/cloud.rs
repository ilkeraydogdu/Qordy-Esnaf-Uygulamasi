//! QORDY POS - Cloud Sync API Endpoints
//! Handles live cloud authentication, full store synchronization, and real-time status.

use crate::api::AppState;
use crate::cloud::{CloudLoginRequest, QordyCloudClient};
use axum::{extract::State, response::Json};
use chrono::{Duration, Local};
use rusqlite::params;
use serde::Serialize;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct CloudSyncResponse {
    pub success: bool,
    pub message: Option<String>,
    pub stats: Option<crate::storage::cloud_sync::CloudSyncStats>,
    pub token: Option<String>,
    pub user: Option<crate::models::User>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CloudStatusResponse {
    pub is_connected: bool,
    pub business_name: String,
    pub tenant_id: String,
    pub user_name: String,
    pub user_email: String,
    pub total_products: usize,
    pub total_customers: usize,
    pub last_synced_at: Option<String>,
}

/// POST /api/cloud/login - Gerçek Qordy Bulut Hesabına Giriş Yap ve Tüm Verileri İndir
pub async fn handle_cloud_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CloudLoginRequest>,
) -> Json<CloudSyncResponse> {
    info!("Bulut Giriş İsteği: {}", payload.email);

    let cloud = QordyCloudClient::default();
    let bundle = match cloud.fetch_full_store_bundle(&payload.email, &payload.password).await {
        Ok(b) => b,
        Err(e) => {
            error!("Bulut giriş/indirme hatası: {}", e);
            return Json(CloudSyncResponse {
                success: false,
                message: None,
                stats: None,
                token: None,
                user: None,
                error: Some(format!("Bulut giriş hatası: {}", e)),
            });
        }
    };

    let user_name = bundle.user.name.clone();
    let user_email = bundle.user.email.clone();
    let tenant_id = bundle.business.id.clone();

    // Veritabanına aktar
    let stats = match state.db.import_cloud_bundle(&bundle) {
        Ok(s) => s,
        Err(e) => {
            error!("Bulut verisi yerel SQLite'a aktarılırken hata: {}", e);
            return Json(CloudSyncResponse {
                success: false,
                message: None,
                stats: None,
                token: None,
                user: None,
                error: Some(format!("Veritabanı aktarım hatası: {}", e)),
            });
        }
    };

    // Yerel kalıcı oturum oluştur
    let local_token = format!("sess_{}", Uuid::new_v4().simple());
    let now = Local::now().to_rfc3339();
    let expires_at = (Local::now() + Duration::days(30)).to_rfc3339();

    let user = crate::models::User {
        id: bundle.user.id.clone(),
        tenant_id: tenant_id.clone(),
        name: user_name.clone(),
        username: user_email.clone(),
        role: "MANAGER".to_string(),
        is_active: true,
        created_at: now.clone(),
    };

    {
        let conn = state.db.conn();
        let _ = conn.execute(
            "INSERT INTO sessions (token, user_id, user_name, role, created_at, expires_at, is_locked, device_info)
             VALUES (?1, ?2, ?3, 'MANAGER', ?4, ?5, 0, 'desktop_pos')",
            params![local_token, user.id, user.name, now, expires_at],
        );
    }

    // Arka plan sync motoruna bildir
    state.sync.trigger_sync();

    Json(CloudSyncResponse {
        success: true,
        message: Some(format!(
            "'{}' mağazasına ait {} ürün ve {} müşteri başarıyla yüklendi!",
            stats.business_name, stats.products_imported, stats.customers_imported
        )),
        stats: Some(stats),
        token: Some(local_token),
        user: Some(user),
        error: None,
    })
}

/// POST /api/cloud/sync - Mevcut mağazayı manuel olarak bulutla eşitle
pub async fn handle_cloud_sync(State(state): State<Arc<AppState>>) -> Json<CloudSyncResponse> {
    let cloud_token = state.db.get_app_config("cloud_token");
    let cloud_email = state.db.get_app_config("cloud_user_email");

    if cloud_token.is_none() {
        return Json(CloudSyncResponse {
            success: false,
            message: None,
            stats: None,
            token: None,
            user: None,
            error: Some("Cihaz henüz bir bulut hesabına bağlanmamış.".to_string()),
        });
    }

    let token = cloud_token.unwrap();
    let cloud = QordyCloudClient::default();

    // Ürünleri ve müşterileri güncelle
    match cloud.fetch_all_retail_products(&token).await {
        Ok(products) => {
            let customers = cloud.fetch_retail_customers(&token).await.unwrap_or_default();
            let suppliers = cloud.fetch_suppliers(&token).await.unwrap_or_default();

            let biz_name = state.db.get_app_config("business_name").unwrap_or_else(|| "KMK Market".to_string());
            let tenant_id = state.db.get_app_config("tenant_id").unwrap_or_else(|| "CUST_6aca07fe3e290".to_string());
            let user_name = state.db.get_app_config("cloud_user_name").unwrap_or_else(|| "İlker Aydoğdu".to_string());
            let email = cloud_email.unwrap_or_else(|| "aydilker123@gmail.com".to_string());

            let bundle = crate::cloud::CloudSyncBundle {
                token: token.clone(),
                user: crate::cloud::CloudUser {
                    id: format!("usr_{}", tenant_id),
                    name: user_name,
                    email,
                    role: "MANAGER".to_string(),
                    is_manager: true,
                },
                business: crate::cloud::CloudBusiness {
                    id: tenant_id,
                    name: biz_name,
                    subdomain: None,
                    address: state.db.get_app_config("business_address"),
                    phone: state.db.get_app_config("business_phone"),
                    business_type: Some("market".to_string()),
                    is_retail: Some(true),
                },
                products,
                customers,
                suppliers,
            };

            match state.db.import_cloud_bundle(&bundle) {
                Ok(stats) => Json(CloudSyncResponse {
                    success: true,
                    message: Some("Bulut verileri başarıyla güncellendi.".to_string()),
                    stats: Some(stats),
                    token: None,
                    user: None,
                    error: None,
                }),
                Err(e) => Json(CloudSyncResponse {
                    success: false,
                    message: None,
                    stats: None,
                    token: None,
                    user: None,
                    error: Some(format!("Yerel güncelleme hatası: {}", e)),
                }),
            }
        }
        Err(e) => Json(CloudSyncResponse {
            success: false,
            message: None,
            stats: None,
            token: None,
            user: None,
            error: Some(format!("Buluttan ürün çekilemedi: {}", e)),
        }),
    }
}

/// GET /api/cloud/status - Bulut bağlantı ve mağaza durumunu döndür
pub async fn handle_cloud_status(State(state): State<Arc<AppState>>) -> Json<CloudStatusResponse> {
    let cloud_token = state.db.get_app_config("cloud_token");
    let business_name = state.db.get_app_config("business_name").unwrap_or_else(|| "KMK Market".to_string());
    let tenant_id = state.db.get_app_config("tenant_id").unwrap_or_else(|| "CUST_6aca07fe3e290".to_string());
    let user_name = state.db.get_app_config("cloud_user_name").unwrap_or_else(|| "İlker Aydoğdu".to_string());
    let user_email = state.db.get_app_config("cloud_user_email").unwrap_or_else(|| "aydilker123@gmail.com".to_string());
    let last_synced_at = state.db.get_app_config("cloud_synced_at");

    let total_products: usize = {
        let conn = state.db.conn();
        conn.query_row("SELECT COUNT(*) FROM products", [], |r| r.get(0)).unwrap_or(0)
    };

    let total_customers: usize = {
        let conn = state.db.conn();
        conn.query_row("SELECT COUNT(*) FROM customers", [], |r| r.get(0)).unwrap_or(0)
    };

    Json(CloudStatusResponse {
        is_connected: cloud_token.is_some(),
        business_name,
        tenant_id,
        user_name,
        user_email,
        total_products,
        total_customers,
        last_synced_at,
    })
}
