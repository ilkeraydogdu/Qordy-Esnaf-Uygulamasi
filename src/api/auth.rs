//! QORDY POS - Authentication & Session API Endpoints
//! Secure PIN/Password authentication, session validation, lock/unlock, and user management.

use crate::api::AppState;
use crate::models::{
    AuthResponse, ChangePinRequest, CreateCashierRequest, LoginRequest, UnlockRequest, User,
};
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct SessionQuery {
    pub token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SessionValidateResponse {
    pub valid: bool,
    pub session: Option<crate::models::Session>,
}

#[derive(Debug, Serialize)]
pub struct SimpleSuccessResponse {
    pub success: bool,
    pub message: Option<String>,
}

/// Token'ı Header (Authorization: Bearer veya X-Session-Token) veya Query'den ayıklar
fn extract_token(headers: &HeaderMap, query_token: Option<&str>) -> Option<String> {
    if let Some(auth_hdr) = headers.get("Authorization") {
        if let Ok(val) = auth_hdr.to_str() {
            if let Some(token) = val.strip_prefix("Bearer ") {
                return Some(token.trim().to_string());
            }
            return Some(val.trim().to_string());
        }
    }

    if let Some(hdr) = headers.get("X-Session-Token") {
        if let Ok(val) = hdr.to_str() {
            return Some(val.trim().to_string());
        }
    }

    query_token.map(|t| t.trim().to_string())
}

/// GET /api/auth/users - Aktif kullanıcıları (Kasiyer / Yönetici) listeler
pub async fn handle_get_users(State(state): State<Arc<AppState>>) -> Json<Vec<User>> {
    let users = state.db.list_active_users().unwrap_or_default();
    Json(users)
}

/// POST /api/auth/login - PIN veya Kullanıcı Adı/Şifre ile güvenli giriş
pub async fn handle_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Json<AuthResponse> {
    let remember = payload.remember_me.unwrap_or(true);

    if let Some(pin) = payload.pin {
        if !pin.trim().is_empty() {
            return match state.db.login_with_pin(&pin, remember) {
                Ok(resp) => Json(resp),
                Err(e) => Json(AuthResponse {
                    success: false,
                    token: None,
                    user: None,
                    is_locked: false,
                    error: Some(format!("Giriş hatası: {}", e)),
                }),
            };
        }
    }

    if let (Some(u), Some(p)) = (payload.username, payload.password) {
        let clean_u = u.trim();
        let clean_p = p.trim();

        // 1. Önce yerel veritabanında doğrula
        let local_attempt = state.db.login_with_password(clean_u, clean_p, remember);
        if let Ok(ref resp) = local_attempt {
            if resp.success {
                return Json(resp.clone());
            }
        }

        // 2. E-posta formatında ise veya yerelde henüz yoksa doğrudan Qordy Bulut'a bağlan
        if clean_u.contains('@') {
            let cloud = crate::cloud::QordyCloudClient::default();
            if let Ok(bundle) = cloud.fetch_full_store_bundle(clean_u, clean_p).await {
                if let Ok(_stats) = state.db.import_cloud_bundle(&bundle) {
                    state.sync.trigger_sync();

                    // Bulut aktarımı tamamlandıktan sonra yerel oturum aç
                    if let Ok(resp) = state.db.login_with_password(clean_u, clean_p, remember) {
                        if resp.success {
                            return Json(resp);
                        }
                    }
                }
            }
        }

        return match local_attempt {
            Ok(resp) => Json(resp),
            Err(e) => Json(AuthResponse {
                success: false,
                token: None,
                user: None,
                is_locked: false,
                error: Some(format!("Giriş hatası: {}", e)),
            }),
        };
    }

    Json(AuthResponse {
        success: false,
        token: None,
        user: None,
        is_locked: false,
        error: Some("Lütfen PIN veya kullanıcı adı/şifre giriniz.".to_string()),
    })
}

/// GET /api/auth/session - Mevcut oturumu doğrula (Kalıcı oturum kontrolü)
pub async fn handle_validate_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<SessionQuery>,
) -> Json<SessionValidateResponse> {
    let token_opt = extract_token(&headers, query.token.as_deref());

    if let Some(token) = token_opt {
        match state.db.validate_session(&token) {
            Ok(Some(session)) => Json(SessionValidateResponse {
                valid: true,
                session: Some(session),
            }),
            _ => Json(SessionValidateResponse {
                valid: false,
                session: None,
            }),
        }
    } else {
        Json(SessionValidateResponse {
            valid: false,
            session: None,
        })
    }
}

/// POST /api/auth/lock - Ekranı anında kilitle
pub async fn handle_lock_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Json<SimpleSuccessResponse> {
    if let Some(token) = extract_token(&headers, None) {
        let _ = state.db.lock_session(&token);
        Json(SimpleSuccessResponse {
            success: true,
            message: Some("Ekran kilitlendi.".to_string()),
        })
    } else {
        Json(SimpleSuccessResponse {
            success: false,
            message: Some("Oturum bulunamadı.".to_string()),
        })
    }
}

/// POST /api/auth/unlock - Kilitli ekranı PIN ile aç
pub async fn handle_unlock_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<UnlockRequest>,
) -> Json<SimpleSuccessResponse> {
    if let Some(token) = extract_token(&headers, None) {
        match state.db.unlock_session(&token, &payload.pin) {
            Ok(true) => Json(SimpleSuccessResponse {
                success: true,
                message: Some("Kilit başarıyla açıldı.".to_string()),
            }),
            Ok(false) => Json(SimpleSuccessResponse {
                success: false,
                message: Some("Hatalı PIN kodu. Lütfen tekrar deneyin.".to_string()),
            }),
            Err(e) => Json(SimpleSuccessResponse {
                success: false,
                message: Some(format!("Kilit açma hatası: {}", e)),
            }),
        }
    } else {
        Json(SimpleSuccessResponse {
            success: false,
            message: Some("Geçerli oturum belirteci bulunamadı.".to_string()),
        })
    }
}

/// POST /api/auth/logout - Oturumu sonlandır
pub async fn handle_logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Json<SimpleSuccessResponse> {
    if let Some(token) = extract_token(&headers, None) {
        let _ = state.db.logout_session(&token);
    }
    Json(SimpleSuccessResponse {
        success: true,
        message: Some("Oturum kapatıldı.".to_string()),
    })
}

/// POST /api/auth/change-pin - Kullanıcı PIN kodunu güncelle
pub async fn handle_change_pin(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ChangePinRequest>,
) -> Json<SimpleSuccessResponse> {
    let token_opt = extract_token(&headers, None);
    let target_user_id = if let Some(uid) = payload.user_id {
        uid
    } else if let Some(token) = token_opt {
        match state.db.validate_session(&token) {
            Ok(Some(sess)) => sess.user_id,
            _ => {
                return Json(SimpleSuccessResponse {
                    success: false,
                    message: Some("Geçerli bir oturum bulunamadı.".to_string()),
                });
            }
        }
    } else {
        return Json(SimpleSuccessResponse {
            success: false,
            message: Some("Kullanıcı ID veya oturum gereklidir.".to_string()),
        });
    };

    match state.db.update_user_pin(&target_user_id, &payload.new_pin) {
        Ok(true) => Json(SimpleSuccessResponse {
            success: true,
            message: Some("PIN kodu başarıyla güncellendi.".to_string()),
        }),
        Ok(false) => Json(SimpleSuccessResponse {
            success: false,
            message: Some("Kullanıcı bulunamadı.".to_string()),
        }),
        Err(e) => Json(SimpleSuccessResponse {
            success: false,
            message: Some(format!("PIN güncellenemedi: {}", e)),
        }),
    }
}

/// POST /api/auth/register-cashier - Yeni kasiyer/personel ekle
pub async fn handle_register_cashier(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCashierRequest>,
) -> Json<SimpleSuccessResponse> {
    let role = payload.role.as_deref().unwrap_or("CASHIER");
    match state.db.create_cashier(&payload.name, &payload.username, &payload.pin, role) {
        Ok(_) => Json(SimpleSuccessResponse {
            success: true,
            message: Some(format!("Kasiyer '{}' başarıyla oluşturuldu.", payload.name)),
        }),
        Err(e) => Json(SimpleSuccessResponse {
            success: false,
            message: Some(format!("Kasiyer oluşturulamadı: {}", e)),
        }),
    }
}
