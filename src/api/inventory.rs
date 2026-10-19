//! QORDY POS - Products, Stock Audit, Waste & Labels API Handlers

use crate::api::AppState;
use crate::models::{
    CreateProductRequest, CreateWasteRequest, Product, ShelfLabel, StockCountItem,
    StockCountSession, WasteRecord,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CategoryFilterQuery {
    pub category: Option<String>,
}

#[derive(Deserialize)]
pub struct StartCountRequest {
    pub title: String,
    pub count_type: String,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct ScanCountItemRequest {
    pub barcode: String,
    pub qty: f64,
}

// ==========================================
// Ürün Yönetimi (CRUD)
// ==========================================
pub async fn handle_get_products(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CategoryFilterQuery>,
) -> Json<Vec<Product>> {
    let list = state.db.get_all_products(query.category.as_deref());
    Json(list)
}

pub async fn handle_create_product(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<Product>, (StatusCode, String)> {
    match state.db.create_product(req) {
        Ok(p) => Ok(Json(p)),
        Err(e) => Err((StatusCode::BAD_REQUEST, format!("Ürün eklenemedi: {}", e))),
    }
}

pub async fn handle_update_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<Product>, (StatusCode, String)> {
    match state.db.update_product(&id, req) {
        Ok(p) => Ok(Json(p)),
        Err(e) => Err((StatusCode::BAD_REQUEST, format!("Ürün güncellenemedi: {}", e))),
    }
}

pub async fn handle_delete_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    match state.db.delete_product(&id) {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::BAD_REQUEST, format!("Ürün silinemedi: {}", e))),
    }
}

pub async fn handle_get_categories(State(state): State<Arc<AppState>>) -> Json<Vec<String>> {
    let list = state.db.get_categories();
    Json(list)
}

// ==========================================
// Stok Sayım & Envanter Denetimi
// ==========================================
pub async fn handle_get_stock_sessions(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<StockCountSession>> {
    let list = state.db.get_stock_count_sessions();
    Json(list)
}

pub async fn handle_start_stock_session(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StartCountRequest>,
) -> Result<Json<StockCountSession>, (StatusCode, String)> {
    match state
        .db
        .start_stock_count(&req.title, &req.count_type, req.notes.as_deref())
    {
        Ok(s) => Ok(Json(s)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Sayım başlatılamadı: {}", e),
        )),
    }
}

pub async fn handle_get_stock_items(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Vec<StockCountItem>> {
    let list = state.db.get_stock_count_items(&id);
    Json(list)
}

pub async fn handle_scan_stock_item(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(req): Json<ScanCountItemRequest>,
) -> Result<Json<StockCountItem>, (StatusCode, String)> {
    match state
        .db
        .add_stock_count_item(&session_id, &req.barcode, req.qty)
    {
        Ok(item) => Ok(Json(item)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            format!("Kalem eklenemedi: {}", e),
        )),
    }
}

pub async fn handle_commit_stock_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    match state.db.commit_stock_count(&id) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Sayım onaylanamadı: {}", e),
        )),
    }
}

// ==========================================
// Fire / Zayi Takibi
// ==========================================
pub async fn handle_get_waste(State(state): State<Arc<AppState>>) -> Json<Vec<WasteRecord>> {
    let list = state.db.get_waste_records();
    Json(list)
}

pub async fn handle_create_waste(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateWasteRequest>,
) -> Result<Json<WasteRecord>, (StatusCode, String)> {
    match state.db.record_waste(req) {
        Ok(rec) => Ok(Json(rec)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            format!("Fire kaydedilemedi: {}", e),
        )),
    }
}

// ==========================================
// Raf Etiketleri
// ==========================================
pub async fn handle_get_shelf_labels(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CategoryFilterQuery>,
) -> Json<Vec<ShelfLabel>> {
    let labels = state.db.get_shelf_labels(query.category.as_deref());
    Json(labels)
}
