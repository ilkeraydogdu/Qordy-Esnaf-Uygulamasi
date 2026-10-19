//! QORDY POS - Şubeler ve Transfer API Handlers

use crate::api::AppState;
use crate::models::CreateStockTransferRequest;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use tracing::info;

pub async fn handle_get_branches(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let branches = state.db.get_branches();
    Json(serde_json::json!({
        "success": true,
        "branches": branches
    }))
}

pub async fn handle_create_transfer(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateStockTransferRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!(
        "🚚 Şubeler Arası Transfer Talebi: {} -> {}, {} Kalem",
        req.from_branch_name,
        req.to_branch_name,
        req.items.len()
    );

    match state.db.create_stock_transfer(req) {
        Ok(transfer) => Ok(Json(serde_json::json!({
            "success": true,
            "transfer": transfer,
            "message": "Transfer sevk talebi başarıyla oluşturuldu."
        }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

pub async fn handle_get_transfers(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let list = state.db.get_stock_transfers();
    Json(serde_json::json!({
        "success": true,
        "transfers": list
    }))
}

pub async fn handle_get_transfer_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match state.db.get_stock_transfer_detail(&id) {
        Some(detail) => Ok(Json(serde_json::json!({
            "success": true,
            "transfer": detail
        }))),
        None => Err((StatusCode::NOT_FOUND, "Transfer kaydı bulunamadı".to_string())),
    }
}

pub async fn handle_receive_transfer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match state.db.receive_stock_transfer(&id) {
        Ok(transfer) => Ok(Json(serde_json::json!({
            "success": true,
            "transfer": transfer,
            "message": "Transfer başarıyla teslim alındı ve stoğa eklendi."
        }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}
