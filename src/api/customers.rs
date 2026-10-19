//! QORDY POS - Customers & Veresiye Ledger API Handlers

use crate::api::AppState;
use crate::models::{CreateCustomerRequest, Customer, LedgerEntry};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CustomerPaymentRequest {
    pub customer_id: String,
    pub amount: f64,
    pub note: Option<String>,
}

pub async fn handle_customers(State(state): State<Arc<AppState>>) -> Json<Vec<Customer>> {
    let list = state.db.get_customers();
    Json(list)
}

pub async fn handle_create_customer(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCustomerRequest>,
) -> Result<Json<Customer>, (StatusCode, String)> {
    match state.db.create_customer(req) {
        Ok(c) => Ok(Json(c)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            format!("Müşteri oluşturulamadı: {}", e),
        )),
    }
}

pub async fn handle_customer_ledger(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Vec<LedgerEntry>> {
    let entries = state.db.get_customer_ledger(&id);
    Json(entries)
}

pub async fn handle_customer_payment(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CustomerPaymentRequest>,
) -> Result<Json<LedgerEntry>, (StatusCode, String)> {
    let note = req.note.as_deref().unwrap_or("Elden Nakit Tahsilat");
    match state
        .db
        .record_customer_payment(&req.customer_id, req.amount, note)
    {
        Ok(entry) => Ok(Json(entry)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            format!("Tahsilat kaydedilemedi: {}", e),
        )),
    }
}
