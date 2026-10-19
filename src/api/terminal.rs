//! QORDY POS - POS Terminal & Beko GMP3 API Handlers

use crate::api::AppState;
use crate::terminal::{TerminalConfig, TerminalPaymentRequest, TerminalPaymentResult};
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

pub async fn handle_terminal_pay(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TerminalPaymentRequest>,
) -> Result<Json<TerminalPaymentResult>, (StatusCode, String)> {
    let terminal = state.terminal_mgr.get_terminal();
    match terminal.process_payment(&req).await {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err((StatusCode::BAD_GATEWAY, format!("Terminal hatası: {}", e))),
    }
}

pub async fn handle_terminal_test(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let terminal = state.terminal_mgr.get_terminal();
    let mode = terminal.get_mode();
    let connected = terminal.test_connection().await.unwrap_or(false);
    Json(serde_json::json!({
        "mode": mode,
        "connected": connected
    }))
}

pub async fn handle_get_terminal_config(
    State(state): State<Arc<AppState>>,
) -> Json<TerminalConfig> {
    let cfg = state.terminal_mgr.get_config();
    Json(cfg)
}

pub async fn handle_save_terminal_config(
    State(state): State<Arc<AppState>>,
    Json(cfg): Json<TerminalConfig>,
) -> Json<serde_json::Value> {
    state.terminal_mgr.update_config(cfg);
    Json(serde_json::json!({
        "success": true,
        "message": "Terminal ayarları güncellendi."
    }))
}
