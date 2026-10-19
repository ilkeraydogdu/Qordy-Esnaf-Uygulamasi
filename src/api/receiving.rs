//! QORDY POS - Mal Kabul, Tedarikçi & Alış İrsaliyesi API Handlers

use crate::api::AppState;
use crate::models::{CreateGoodsReceiptRequest, CreateSupplierRequest};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use tracing::info;

pub async fn handle_get_suppliers(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let suppliers = state.db.get_suppliers();
    Json(serde_json::json!({
        "success": true,
        "suppliers": suppliers
    }))
}

pub async fn handle_create_supplier(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSupplierRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match state.db.create_supplier(req) {
        Ok(supplier) => Ok(Json(serde_json::json!({
            "success": true,
            "supplier": supplier,
            "message": "Tedarikçi başarıyla kaydedildi."
        }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

pub async fn handle_process_receiving(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateGoodsReceiptRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!(
        "📥 Mal Kabul İsteği: Tedarikçi: {}, İrsaliye: {}, {} Kalem",
        req.supplier_name,
        req.invoice_no,
        req.items.len()
    );

    match state.db.process_goods_receipt(req) {
        Ok(receipt) => Ok(Json(serde_json::json!({
            "success": true,
            "receipt": receipt,
            "message": "Mal kabul başarıyla tamamlandı. Stoklar ve maliyetler güncellendi."
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn handle_receiving_history(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let list = state.db.get_goods_receipts();
    Json(serde_json::json!({
        "success": true,
        "receipts": list
    }))
}

pub async fn handle_receiving_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match state.db.get_goods_receipt_detail(&id) {
        Some(detail) => Ok(Json(serde_json::json!({
            "success": true,
            "receipt": detail
        }))),
        None => Err((StatusCode::NOT_FOUND, "Mal kabul kaydı bulunamadı".to_string())),
    }
}
