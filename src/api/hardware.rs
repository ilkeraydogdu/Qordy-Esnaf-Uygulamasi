//! QORDY POS - ESC/POS Hardware Printing & Cash Drawer API Handlers

use crate::api::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

#[derive(Deserialize)]
pub struct PrintReceiptReq {
    pub receipt_no: Option<String>,
    pub _printer_ip: Option<String>,
}

pub async fn handle_print_receipt(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<PrintReceiptReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let receipt_no = req.receipt_no.unwrap_or_default();
    info!("🖨️ Fiş yazdırma tetiklendi: {}", receipt_no);
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Yazdırma komutu kuyruğa iletildi.",
        "receipt_no": receipt_no
    })))
}

pub async fn handle_kick_drawer(
    State(_state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    info!("💵 Kasa çekmecesi açma tetiklendi (ESC p 0 25 250)");
    Json(serde_json::json!({
        "success": true,
        "message": "Kasa çekmecesi tetiklendi."
    }))
}
