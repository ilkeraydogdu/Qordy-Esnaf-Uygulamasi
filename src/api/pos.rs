//! QORDY POS - Core POS, Barcode, Cart & Checkout API Handlers

use crate::api::AppState;
use crate::barcode::BarcodeParser;
use crate::models::{
    CartItem, CheckoutRequest, CheckoutResponse, ParkedCart, PosStatus, Product,
    RefundRequest, RefundResponse, Sale,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

#[derive(Deserialize)]
pub struct BarcodeScanRequest {
    pub barcode: String,
}

#[derive(Serialize)]
pub struct BarcodeScanResponse {
    pub found: bool,
    pub product: Option<Product>,
    pub parsed: ParsedBarcodeJson,
    pub suggested_item: Option<CartItem>,
}

#[derive(Serialize)]
pub struct ParsedBarcodeJson {
    pub raw_barcode: String,
    pub lookup_code: String,
    pub item_code: Option<String>,
    pub is_weighed: bool,
    pub weight_kg: Option<f64>,
    pub embedded_price: Option<f64>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    #[allow(dead_code)]
    pub category: Option<String>,
}

#[derive(Deserialize)]
pub struct ParkCartRequest {
    pub note: String,
    pub items: Vec<CartItem>,
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<usize>,
}

pub async fn handle_status(State(state): State<Arc<AppState>>) -> Json<PosStatus> {
    let is_online = state.sync.is_online();
    let status = state.db.get_pos_status(is_online);
    Json(status)
}

pub async fn handle_quick_products(State(state): State<Arc<AppState>>) -> Json<Vec<Product>> {
    let list = state.db.get_quick_products();
    Json(list)
}

pub async fn handle_search_products(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Json<Vec<Product>> {
    let q = query.q.unwrap_or_default();
    let results = state.db.search_products(&q);
    Json(results)
}

pub async fn handle_barcode_scan(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BarcodeScanRequest>,
) -> Json<BarcodeScanResponse> {
    let parsed = BarcodeParser::parse(&req.barcode);

    let product_opt = state.db.find_product_by_barcode(&parsed.lookup_code);

    let found = product_opt.is_some();
    let mut suggested_item = None;

    if let Some(ref prod) = product_opt {
        let (qty, total) = if parsed.is_weighed {
            if let Some(kg) = parsed.weight_kg {
                (kg, (kg * prod.price * 100.0).round() / 100.0)
            } else if let Some(price) = parsed.embedded_price {
                let calculated_kg = price / prod.price;
                (calculated_kg, price)
            } else {
                (1.0, prod.price)
            }
        } else {
            (1.0, prod.price)
        };

        suggested_item = Some(CartItem {
            product_id: prod.id.clone(),
            barcode: req.barcode.clone(),
            name: prod.name.clone(),
            unit_price: prod.price,
            quantity: qty,
            unit: prod.unit.clone(),
            vat_rate: prod.vat_rate,
            discount: 0.0,
            total,
            is_weighed: prod.is_weighed,
            notes: None,
        });
    }

    Json(BarcodeScanResponse {
        found,
        product: product_opt,
        parsed: ParsedBarcodeJson {
            raw_barcode: parsed.raw_barcode,
            lookup_code: parsed.lookup_code,
            item_code: parsed.item_code,
            is_weighed: parsed.is_weighed,
            weight_kg: parsed.weight_kg,
            embedded_price: parsed.embedded_price,
        },
        suggested_item,
    })
}

pub async fn handle_checkout(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CheckoutRequest>,
) -> Result<Json<CheckoutResponse>, (StatusCode, String)> {
    info!(
        "🛒 Hızlı Satış İsteği: {} Kalem, Ödeme: {}",
        req.items.len(),
        req.payment_method
    );

    match state.db.process_checkout(req) {
        Ok(sale) => {
            // CFD WebSocket yayını
            let _ = state.ws_tx.send(
                serde_json::json!({
                    "type": "CFD_UPDATE",
                    "data": {
                        "state": "PAYMENT_COMPLETED",
                        "change": sale.change_amount,
                        "grand_total": sale.grand_total,
                        "payment_method": sale.payment_method
                    }
                })
                .to_string(),
            );

            Ok(Json(CheckoutResponse {
                success: true,
                sale_id: sale.id,
                receipt_no: sale.receipt_no,
                grand_total: sale.grand_total,
                change_amount: sale.change_amount,
                sync_status: sale.sync_status,
                printed: true,
                message: "Satış başarıyla kaydedildi.".to_string(),
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Satış işlenemedi: {}", e),
        )),
    }
}

pub async fn handle_sync_trigger(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    state.sync.trigger_sync();
    Json(serde_json::json!({ "status": "triggered" }))
}

pub async fn handle_get_sales_history(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HistoryQuery>,
) -> Json<Vec<Sale>> {
    let limit = query.limit.unwrap_or(100);
    let sales = state.db.get_sales_history(limit);
    Json(sales)
}

pub async fn handle_process_refund(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefundRequest>,
) -> Result<Json<RefundResponse>, (StatusCode, String)> {
    match state.db.process_refund(req) {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            format!("İade işlemi başarısız: {}", e),
        )),
    }
}

pub async fn handle_get_parked_carts(State(state): State<Arc<AppState>>) -> Json<Vec<ParkedCart>> {
    let list = state.db.get_parked_carts();
    Json(list)
}

pub async fn handle_park_cart(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ParkCartRequest>,
) -> Result<Json<ParkedCart>, (StatusCode, String)> {
    match state.db.park_cart(&req.note, req.items) {
        Ok(p) => Ok(Json(p)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Sepet bekletilemedi: {}", e),
        )),
    }
}

pub async fn handle_delete_parked_cart(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    match state.db.delete_parked_cart(&id) {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::BAD_REQUEST, format!("Silinemedi: {}", e))),
    }
}
