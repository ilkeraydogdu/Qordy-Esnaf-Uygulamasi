//! QORDY POS - Kampanya ve İndirim API Handlers

use crate::api::AppState;
use crate::models::{CartItem, CreateCampaignRequest};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

pub async fn handle_get_campaigns(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let campaigns = state.db.get_campaigns();
    Json(serde_json::json!({
        "success": true,
        "campaigns": campaigns
    }))
}

pub async fn handle_create_campaign(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCampaignRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match state.db.create_campaign(req) {
        Ok(camp) => Ok(Json(serde_json::json!({
            "success": true,
            "campaign": camp,
            "message": "Kampanya başarıyla oluşturuldu."
        }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

pub async fn handle_toggle_campaign(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match state.db.toggle_campaign(&id) {
        Ok(is_active) => Ok(Json(serde_json::json!({
            "success": true,
            "is_active": is_active,
            "message": if is_active { "Kampanya aktif edildi." } else { "Kampanya durduruldu." }
        }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

pub async fn handle_delete_campaign(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match state.db.delete_campaign(&id) {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "Kampanya silindi."
        }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

#[derive(Deserialize)]
pub struct CalculateCampaignRequest {
    pub subtotal: f64,
    pub items: Vec<CartItem>,
}

pub async fn handle_calculate_campaign(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CalculateCampaignRequest>,
) -> Json<serde_json::Value> {
    let (discount, camp_name) = state
        .db
        .calculate_campaign_discount(req.subtotal, &req.items);

    Json(serde_json::json!({
        "success": true,
        "discount_amount": discount.to_string(),
        "applied_campaign": camp_name
    }))
}
