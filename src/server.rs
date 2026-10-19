use crate::barcode::BarcodeParser;
use crate::escpos::ReceiptPrinter;
use crate::models::{
    CartItem, CheckoutRequest, CheckoutResponse, Customer, PosStatus, Product, ZReport,
};
use crate::storage::DatabaseManager;
use crate::sync::SyncEngine;
use crate::ui;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

pub struct AppState {
    pub db: Arc<DatabaseManager>,
    pub sync: Arc<SyncEngine>,
}

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
pub struct CustomerPaymentRequest {
    pub customer_id: String,
    pub amount: f64,
    pub note: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(handle_index))
        .route("/api/status", get(handle_status))
        .route("/api/products/quick", get(handle_quick_products))
        .route("/api/products/search", get(handle_search_products))
        .route("/api/barcode/scan", post(handle_barcode_scan))
        .route("/api/checkout", post(handle_checkout))
        .route("/api/customers", get(handle_customers))
        .route("/api/customers/payment", post(handle_customer_payment))
        .route("/api/reports/z", get(handle_z_report))
        .route("/api/sync/trigger", post(handle_sync_trigger))
        .layer(cors)
        .with_state(state)
}

async fn handle_index() -> Html<&'static str> {
    Html(ui::get_pos_html())
}

async fn handle_status(State(state): State<Arc<AppState>>) -> Json<PosStatus> {
    let online = state.sync.is_online();
    let status = state.db.get_pos_status(online);
    Json(status)
}

async fn handle_quick_products(State(state): State<Arc<AppState>>) -> Json<Vec<Product>> {
    let products = state.db.get_quick_products();
    Json(products)
}

async fn handle_search_products(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Json<Vec<Product>> {
    let q = query.q.unwrap_or_default();
    let products = state.db.search_products(&q);
    Json(products)
}

async fn handle_barcode_scan(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BarcodeScanRequest>,
) -> Json<BarcodeScanResponse> {
    let parsed = BarcodeParser::parse(&req.barcode);

    let parsed_json = ParsedBarcodeJson {
        raw_barcode: parsed.raw_barcode.clone(),
        lookup_code: parsed.lookup_code.clone(),
        item_code: parsed.item_code.clone(),
        is_weighed: parsed.is_weighed,
        weight_kg: parsed.weight_kg,
        embedded_price: parsed.embedded_price,
    };

    // Önce lookup_code ile ürün ara
    let product_opt = state.db.find_product_by_barcode(&parsed.lookup_code);

    if let Some(product) = product_opt {
        // Terazi veya adetli ürün için kalem oluştur
        let (qty, total) = if let Some(w) = parsed.weight_kg {
            let total = (w * product.price * 100.0).round() / 100.0;
            (w, total)
        } else if let Some(p) = parsed.embedded_price {
            let qty = if product.price > 0.0 {
                ((p / product.price) * 1000.0).round() / 1000.0
            } else {
                1.0
            };
            (qty, p)
        } else {
            (1.0, product.price)
        };

        let suggested_item = CartItem {
            product_id: product.id.clone(),
            barcode: parsed.raw_barcode.clone(),
            name: product.name.clone(),
            unit_price: product.price,
            quantity: qty,
            unit: product.unit.clone(),
            vat_rate: product.vat_rate,
            discount: 0.0,
            total,
            is_weighed: product.is_weighed || parsed.is_weighed,
            notes: None,
        };

        Json(BarcodeScanResponse {
            found: true,
            product: Some(product),
            parsed: parsed_json,
            suggested_item: Some(suggested_item),
        })
    } else {
        Json(BarcodeScanResponse {
            found: false,
            product: None,
            parsed: parsed_json,
            suggested_item: None,
        })
    }
}

async fn handle_checkout(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CheckoutRequest>,
) -> Result<Json<CheckoutResponse>, (StatusCode, String)> {
    match state.db.process_checkout(req) {
        Ok(sale) => {
            info!("✅ Satış başarıyla tamamlandı: Fiş #{} - Tutar: {:.2} TL", sale.receipt_no, sale.grand_total);

            // Termal Fiş ESC/POS baytlarını üret
            let _receipt_bytes = ReceiptPrinter::build_sale_receipt(&sale);

            // Çevrimiçi ise arka planda anında bulut aktarımını tetikle
            state.sync.trigger_sync();

            Ok(Json(CheckoutResponse {
                success: true,
                sale_id: sale.id,
                receipt_no: sale.receipt_no,
                grand_total: sale.grand_total,
                change_amount: sale.change_amount,
                sync_status: sale.sync_status,
                printed: true,
                message: "Satış başarıyla tamamlandı ve fiş üretildi.".to_string(),
            }))
        }
        Err(err) => {
            error!("Satış kaydı sırasında hata: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("İşlem hatası: {}", err),
            ))
        }
    }
}

async fn handle_customers(State(state): State<Arc<AppState>>) -> Json<Vec<Customer>> {
    let list = state.db.get_customers();
    Json(list)
}

async fn handle_customer_payment(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CustomerPaymentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let note = req.note.as_deref().unwrap_or("Tahsilat");
    match state.db.record_customer_payment(&req.customer_id, req.amount, note) {
        Ok(entry) => {
            state.sync.trigger_sync();
            Ok(Json(serde_json::json!({
                "success": true,
                "entry_id": entry.id,
                "amount": entry.amount,
                "balance_after": entry.balance_after,
                "message": "Tahsilat kaydedildi."
            })))
        }
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Tahsilat hatası: {}", err),
        )),
    }
}

async fn handle_z_report(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ZReport>, (StatusCode, String)> {
    match state.db.generate_z_report() {
        Ok(report) => {
            let _report_bytes = ReceiptPrinter::build_z_report(&report);
            Ok(Json(report))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Hata: {}", e))),
    }
}

async fn handle_sync_trigger(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    state.sync.trigger_sync();
    Json(serde_json::json!({
        "success": true,
        "message": "Senkronizasyon tetiklendi."
    }))
}

pub async fn start_server(state: Arc<AppState>, port: u16) -> anyhow::Result<()> {
    let app = create_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("🚀 QORDY POS HTTP Server dinlemede: http://{}", addr);

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!("HTTP Sunucu hatası: {}", e);
        }
    });

    Ok(())
}
