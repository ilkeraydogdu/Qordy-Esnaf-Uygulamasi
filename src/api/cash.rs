//! QORDY POS - Cash Movements & Z-Report API Handlers

use crate::api::AppState;
use crate::models::{CashMovement, CreateCashMovementRequest, ZReport};
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

pub async fn handle_z_report(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ZReport>, (StatusCode, String)> {
    match state.db.generate_z_report() {
        Ok(z) => Ok(Json(z)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Z raporu üretilemedi: {}", e),
        )),
    }
}

pub async fn handle_get_cash_movements(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<CashMovement>> {
    let list = state.db.get_cash_movements();
    Json(list)
}

pub async fn handle_create_cash_movement(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCashMovementRequest>,
) -> Result<Json<CashMovement>, (StatusCode, String)> {
    match state
        .db
        .record_cash_movement(&req.movement_type, req.amount, &req.reason)
    {
        Ok(m) => Ok(Json(m)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            format!("Kasa hareketi kaydedilemedi: {}", e),
        )),
    }
}
