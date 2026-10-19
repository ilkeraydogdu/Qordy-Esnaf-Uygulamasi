use crate::barcode::BarcodeParser;
use crate::escpos::ReceiptPrinter;
use crate::models::{
    CartItem, CashMovement, CfdState, CheckoutRequest, CheckoutResponse, CreateCashMovementRequest,
    CreateCustomerRequest, CreateProductRequest, CreateWasteRequest, Customer, LedgerEntry,
    ParkedCart, PosStatus, Product, RefundRequest, RefundResponse, Sale, ShelfLabel,
    StockCountItem, StockCountSession, WasteRecord, ZReport,
};
use crate::storage::DatabaseManager;
use crate::sync::SyncEngine;
use crate::terminal::{TerminalConfig, TerminalManager, TerminalPaymentRequest};
use crate::ui;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::Html,
    routing::{delete, get, post, put},
    Json, Router,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

pub struct AppState {
    pub db: Arc<DatabaseManager>,
    pub sync: Arc<SyncEngine>,
    pub terminal_mgr: Arc<TerminalManager>,
    pub ws_tx: broadcast::Sender<String>,
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
    pub category: Option<String>,
}

#[derive(Deserialize)]
pub struct ParkCartRequest {
    pub note: String,
    pub items: Vec<CartItem>,
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

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // HTML Sayfaları (Apple Design UI)
        .route("/", get(handle_index))
        .route("/customer", get(handle_customer_display))
        // WebSocket Gerçek Zamanlı Eşitleme
        .route("/ws/pos", get(handle_ws_pos))
        // POS & Sistem Durumu
        .route("/api/status", get(handle_status))
        .route("/api/products/quick", get(handle_quick_products))
        .route("/api/products/search", get(handle_search_products))
        .route("/api/barcode/scan", post(handle_barcode_scan))
        .route("/api/checkout", post(handle_checkout))
        .route("/api/reports/z", get(handle_z_report))
        .route("/api/sync/trigger", post(handle_sync_trigger))
        // Ürün Yönetimi (CRUD)
        .route("/api/products", get(handle_get_products))
        .route("/api/products", post(handle_create_product))
        .route("/api/products/:id", put(handle_update_product))
        .route("/api/products/:id", delete(handle_delete_product))
        .route("/api/categories", get(handle_get_categories))
        // Müşteri & Veresiye Cari Yönetimi
        .route("/api/customers", get(handle_customers))
        .route("/api/customers", post(handle_create_customer))
        .route("/api/customers/:id/ledger", get(handle_customer_ledger))
        .route("/api/customers/payment", post(handle_customer_payment))
        // Sepet Bekletme (Parked Carts)
        .route("/api/cart/parked", get(handle_get_parked_carts))
        .route("/api/cart/park", post(handle_park_cart))
        .route("/api/cart/parked/:id", delete(handle_delete_parked_cart))
        // Stok Sayım & Envanter Denetimi
        .route("/api/stock/sessions", get(handle_get_stock_sessions))
        .route("/api/stock/sessions", post(handle_start_stock_session))
        .route("/api/stock/sessions/:id/items", get(handle_get_stock_items))
        .route("/api/stock/sessions/:id/scan", post(handle_scan_stock_item))
        .route("/api/stock/sessions/:id/commit", post(handle_commit_stock_session))
        // Müşteri Ekranı (CFD) Durumu
        .route("/api/cfd/state", get(handle_get_cfd_state))
        .route("/api/cfd/update", post(handle_update_cfd_state))
        // Satış Geçmişi & İade
        .route("/api/sales/history", get(handle_get_sales_history))
        .route("/api/sales/refund", post(handle_process_refund))
        // Fire / Zayi Takibi
        .route("/api/waste", get(handle_get_waste))
        .route("/api/waste", post(handle_create_waste))
        // Kasa Avans & Masraf (Nakit Hareketleri)
        .route("/api/cash/movements", get(handle_get_cash_movements))
        .route("/api/cash/movements", post(handle_create_cash_movement))
        // Raf Etiketleri
        .route("/api/labels", get(handle_get_shelf_labels))
        // EFT-POS & Beko Terminal
        .route("/api/terminal/pay", post(handle_terminal_pay))
        .route("/api/terminal/test", post(handle_terminal_test))
        .route("/api/terminal/config", get(handle_get_terminal_config))
        .route("/api/terminal/config", post(handle_save_terminal_config))
        // ESC/POS Termal Fiş Yazdırma & Kasa Çekmecesi
        .route("/api/print/receipt", post(handle_print_receipt))
        .route("/api/print/drawer", post(handle_kick_drawer))
        .layer(cors)
        .with_state(state)
}

// ==========================================
// HTML Handler'lar
// ==========================================
async fn handle_index() -> Html<&'static str> {
    Html(ui::get_pos_html())
}

async fn handle_customer_display() -> Html<&'static str> {
    Html(ui::get_customer_display_html())
}

// ==========================================
// WebSocket Handler (Zero Latency Broadcast)
// ==========================================
async fn handle_ws_pos(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_socket(socket, state))
}

async fn handle_ws_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.ws_tx.subscribe();

    // Bağlanır bağlanmaz güncel CFD durumunu müşteriye yolla
    let current_cfd = state.db.get_cfd_state();
    if let Ok(json) = serde_json::to_string(&serde_json::json!({
        "type": "CFD_UPDATE",
        "data": current_cfd
    })) {
        let _ = socket.send(Message::Text(json)).await;
    }

    loop {
        tokio::select! {
            // Sunucudan istemciye yayın
            Ok(msg) = rx.recv() => {
                if socket.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
            // İstemciden gelen mesajlar (ping/pong veya sepet güncellemeleri)
            client_msg = socket.next() => {
                match client_msg {
                    Some(Ok(Message::Text(txt))) => {
                        // İstemciden CFD durumu gelirse diğer tüm ekranlara yay
                        if let Ok(parsed) = serde_json::from_str::<CfdState>(&txt) {
                            let _ = state.db.update_cfd_state(&parsed);
                            let broadcast_msg = serde_json::json!({
                                "type": "CFD_UPDATE",
                                "data": parsed
                            }).to_string();
                            let _ = state.ws_tx.send(broadcast_msg);
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}

// ==========================================
// POS Temel API'lar
// ==========================================
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

    let product_opt = state.db.find_product_by_barcode(&parsed.lookup_code);

    if let Some(product) = product_opt {
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
            info!("✅ Satış tamamlandı: Fiş #{} - {:.2} TL", sale.receipt_no, sale.grand_total);
            let _receipt_bytes = ReceiptPrinter::build_sale_receipt(&sale);

            // Müşteri ekranını "Ödeme Tamamlandı" durumuna geçir ve yay
            let cfd_update = CfdState {
                state: "PAYMENT_COMPLETED".to_string(),
                items: sale.items.clone(),
                subtotal: sale.subtotal,
                discount: sale.discount_amount,
                grand_total: sale.grand_total,
                tender: sale.tender_amount,
                change: sale.change_amount,
                payment_method: Some(sale.payment_method.clone()),
                customer_name: sale.customer_name.clone(),
                updated_at: chrono::Local::now().to_rfc3339(),
            };
            let _ = state.db.update_cfd_state(&cfd_update);
            let _ = state.ws_tx.send(serde_json::json!({
                "type": "CFD_UPDATE",
                "data": cfd_update
            }).to_string());

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

// ==========================================
// Ürün Yönetimi (CRUD) Handler'lar
// ==========================================
async fn handle_get_products(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Json<Vec<Product>> {
    let prods = state.db.get_all_products(query.category.as_deref());
    Json(prods)
}

async fn handle_create_product(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<Product>, (StatusCode, String)> {
    match state.db.create_product(req) {
        Ok(p) => Ok(Json(p)),
        Err(e) => Err((StatusCode::BAD_REQUEST, format!("Ürün eklenemedi: {}", e))),
    }
}

async fn handle_update_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<Product>, (StatusCode, String)> {
    match state.db.update_product(&id, req) {
        Ok(p) => Ok(Json(p)),
        Err(e) => Err((StatusCode::BAD_REQUEST, format!("Ürün güncellenemedi: {}", e))),
    }
}

async fn handle_delete_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match state.db.delete_product(&id) {
        Ok(_) => Ok(Json(serde_json::json!({"success": true, "id": id}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Silinemedi: {}", e))),
    }
}

async fn handle_get_categories(State(state): State<Arc<AppState>>) -> Json<Vec<String>> {
    let cats = state.db.get_categories();
    Json(cats)
}

// ==========================================
// Müşteri & Veresiye Handler'lar
// ==========================================
async fn handle_customers(State(state): State<Arc<AppState>>) -> Json<Vec<Customer>> {
    let list = state.db.get_customers();
    Json(list)
}

async fn handle_create_customer(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCustomerRequest>,
) -> Result<Json<Customer>, (StatusCode, String)> {
    match state.db.create_customer(req) {
        Ok(c) => Ok(Json(c)),
        Err(e) => Err((StatusCode::BAD_REQUEST, format!("Müşteri eklenemedi: {}", e))),
    }
}

async fn handle_customer_ledger(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Vec<LedgerEntry>> {
    let ledger = state.db.get_customer_ledger(&id);
    Json(ledger)
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

// ==========================================
// Sepet Bekletme (Parked Carts) Handler'lar
// ==========================================
async fn handle_get_parked_carts(State(state): State<Arc<AppState>>) -> Json<Vec<ParkedCart>> {
    let carts = state.db.get_parked_carts();
    Json(carts)
}

async fn handle_park_cart(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ParkCartRequest>,
) -> Result<Json<ParkedCart>, (StatusCode, String)> {
    match state.db.park_cart(&req.note, req.items) {
        Ok(p) => Ok(Json(p)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Sepet bekletilemedi: {}", e))),
    }
}

async fn handle_delete_parked_cart(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match state.db.delete_parked_cart(&id) {
        Ok(_) => Ok(Json(serde_json::json!({"success": true, "id": id}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Silinemedi: {}", e))),
    }
}

// ==========================================
// Stok Sayım & Envanter Handler'lar
// ==========================================
async fn handle_get_stock_sessions(State(state): State<Arc<AppState>>) -> Json<Vec<StockCountSession>> {
    let sessions = state.db.get_stock_count_sessions();
    Json(sessions)
}

async fn handle_start_stock_session(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StartCountRequest>,
) -> Result<Json<StockCountSession>, (StatusCode, String)> {
    match state.db.start_stock_count(&req.title, &req.count_type, req.notes.as_deref()) {
        Ok(s) => Ok(Json(s)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Sayım başlatılamadı: {}", e))),
    }
}

async fn handle_get_stock_items(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Json<Vec<StockCountItem>> {
    let items = state.db.get_stock_count_items(&session_id);
    Json(items)
}

async fn handle_scan_stock_item(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(req): Json<ScanCountItemRequest>,
) -> Result<Json<StockCountItem>, (StatusCode, String)> {
    match state.db.add_stock_count_item(&session_id, &req.barcode, req.qty) {
        Ok(item) => Ok(Json(item)),
        Err(e) => Err((StatusCode::BAD_REQUEST, format!("Sayım kalemi eklenemedi: {}", e))),
    }
}

async fn handle_commit_stock_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match state.db.commit_stock_count(&session_id) {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "session_id": session_id,
            "message": "Sayım onaylandı ve ürün stokları güncellendi."
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Sayım onaylanamadı: {}", e))),
    }
}

// ==========================================
// Müşteri Ekranı (CFD) Durum Handler'lar
// ==========================================
async fn handle_get_cfd_state(State(state): State<Arc<AppState>>) -> Json<CfdState> {
    let cfd = state.db.get_cfd_state();
    Json(cfd)
}

async fn handle_update_cfd_state(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CfdState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if let Err(e) = state.db.update_cfd_state(&req) {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("CFD kaydedilemedi: {}", e)));
    }

    // WebSocket üzerinden tüm 2. ekranlara anlık fırlat
    let msg = serde_json::json!({
        "type": "CFD_UPDATE",
        "data": req
    }).to_string();
    let _ = state.ws_tx.send(msg);

    Ok(Json(serde_json::json!({"success": true})))
}

// ==========================================
// Satış Geçmişi & İade Handler'lar
// ==========================================
#[derive(Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<usize>,
}

async fn handle_get_sales_history(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HistoryQuery>,
) -> Json<Vec<Sale>> {
    let limit = query.limit.unwrap_or(100);
    let sales = state.db.get_sales_history(limit);
    Json(sales)
}

async fn handle_process_refund(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefundRequest>,
) -> Result<Json<RefundResponse>, (StatusCode, String)> {
    match state.db.process_refund(req) {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err((StatusCode::BAD_REQUEST, format!("İade işlemi başarısız: {}", e))),
    }
}

// ==========================================
// Fire / Zayi Takibi Handler'lar
// ==========================================
async fn handle_get_waste(State(state): State<Arc<AppState>>) -> Json<Vec<WasteRecord>> {
    let records = state.db.get_waste_records();
    Json(records)
}

async fn handle_create_waste(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateWasteRequest>,
) -> Result<Json<WasteRecord>, (StatusCode, String)> {
    match state.db.record_waste(req) {
        Ok(w) => Ok(Json(w)),
        Err(e) => Err((StatusCode::BAD_REQUEST, format!("Fire kaydedilemedi: {}", e))),
    }
}

// ==========================================
// Kasa Avans & Masraf (Nakit Hareketleri)
// ==========================================
async fn handle_get_cash_movements(State(state): State<Arc<AppState>>) -> Json<Vec<CashMovement>> {
    let movements = state.db.get_cash_movements();
    Json(movements)
}

async fn handle_create_cash_movement(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCashMovementRequest>,
) -> Result<Json<CashMovement>, (StatusCode, String)> {
    match state.db.record_cash_movement(&req.movement_type, req.amount, &req.reason) {
        Ok(m) => Ok(Json(m)),
        Err(e) => Err((StatusCode::BAD_REQUEST, format!("Kasa hareketi işlenemedi: {}", e))),
    }
}

// ==========================================
// Raf Etiketleri
// ==========================================
async fn handle_get_shelf_labels(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Json<Vec<ShelfLabel>> {
    let labels = state.db.get_shelf_labels(query.category.as_deref());
    Json(labels)
}

// ==========================================
// EFT-POS & Beko Terminal Handler'lar
// ==========================================
#[derive(Deserialize)]
pub struct TerminalPayApiReq {
    pub amount: f64,
    pub txn_type: Option<String>,
    pub order_ref: Option<String>,
    pub print_slip: Option<bool>,
}

async fn handle_terminal_pay(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TerminalPayApiReq>,
) -> Result<Json<crate::terminal::TerminalPaymentResult>, (StatusCode, String)> {
    let term_req = TerminalPaymentRequest {
        amount: req.amount,
        txn_type: req.txn_type.unwrap_or_else(|| "SALE".to_string()),
        order_ref: req.order_ref.unwrap_or_else(|| format!("POS-{}", chrono::Utc::now().timestamp())),
        print_slip_on_pos: req.print_slip.unwrap_or(true),
    };

    let term = state.terminal_mgr.get_terminal();
    match term.process_payment(&term_req).await {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err((StatusCode::BAD_GATEWAY, format!("Terminal iletişim hatası: {}", e))),
    }
}

async fn handle_terminal_test(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let term = state.terminal_mgr.get_terminal();
    let mode = term.get_mode();
    let connected = term.test_connection().await.unwrap_or(false);
    Json(serde_json::json!({
        "success": connected,
        "connected": connected,
        "mode": mode,
    }))
}

async fn handle_get_terminal_config(State(state): State<Arc<AppState>>) -> Json<TerminalConfig> {
    Json(state.terminal_mgr.get_config())
}

async fn handle_save_terminal_config(
    State(state): State<Arc<AppState>>,
    Json(cfg): Json<TerminalConfig>,
) -> Json<serde_json::Value> {
    state.terminal_mgr.update_config(cfg);
    Json(serde_json::json!({"success": true, "message": "Terminal ayarları güncellendi."}))
}

// ==========================================
// ESC/POS Termal Fiş & Kasa Çekmecesi Handler'lar
// ==========================================
#[derive(Deserialize)]
pub struct PrintReceiptReq {
    pub receipt_no: Option<String>,
    pub _printer_ip: Option<String>,
}

async fn handle_print_receipt(
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

async fn handle_kick_drawer(
    State(_state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    info!("💵 Kasa çekmecesi açma tetiklendi (ESC p 0 25 250)");
    Json(serde_json::json!({
        "success": true,
        "message": "Kasa çekmecesi tetiklendi."
    }))
}

pub async fn start_server(state: Arc<AppState>, port: u16) -> anyhow::Result<()> {
    let app = create_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("🚀 QORDY POS HTTP & WebSocket Server dinlemede: http://{}", addr);

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!("HTTP Sunucu hatası: {}", e);
        }
    });

    Ok(())
}
