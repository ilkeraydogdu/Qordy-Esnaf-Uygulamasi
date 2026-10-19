//! QORDY POS - Kasa & Vardiya Yönetimi API Handlers

use crate::api::AppState;
use crate::models::{CloseShiftRequest, OpenShiftRequest};
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;

pub async fn handle_get_active_shift(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let active = state.db.get_active_shift();
    Json(serde_json::json!({
        "success": true,
        "is_open": active.is_some(),
        "shift": active
    }))
}

pub async fn handle_open_shift(
    State(state): State<Arc<AppState>>,
    Json(req): Json<OpenShiftRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match state.db.open_shift(req) {
        Ok(shift) => Ok(Json(serde_json::json!({
            "success": true,
            "shift": shift,
            "message": "Kasa başarıyla açıldı, gün başı nakit devri kaydedildi."
        }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

pub async fn handle_close_shift(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CloseShiftRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match state.db.close_shift(req) {
        Ok(shift) => Ok(Json(serde_json::json!({
            "success": true,
            "shift": shift,
            "message": "Kasa başarıyla kapatıldı. Gün sonu mutabakatı tamamlandı."
        }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

pub async fn handle_x_report(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let report = state.db.get_x_report();
    Json(serde_json::json!({
        "success": true,
        "x_report": report
    }))
}

pub async fn handle_shift_history(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let history = state.db.get_shift_history();
    Json(serde_json::json!({
        "success": true,
        "shifts": history
    }))
}
