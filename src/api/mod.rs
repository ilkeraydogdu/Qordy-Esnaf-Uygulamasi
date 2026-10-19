//! QORDY POS - Modular API Layer
//! Decoupled enterprise REST & WebSocket routing architecture.

pub mod campaigns;
pub mod cash;
pub mod catalog;
pub mod cfd;
pub mod customers;
pub mod hardware;
pub mod inventory;
pub mod pos;
pub mod receiving;
pub mod registers;
pub mod static_files;
pub mod terminal;
pub mod transfers;

use crate::config::AppConfig;
use crate::storage::DatabaseManager;
use crate::sync::SyncEngine;
use crate::terminal::TerminalManager;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
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
    pub config: AppConfig,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // HTML & Static Assets (Apple HIG UI & Decoupled Webview)
        .route("/", get(static_files::handle_index))
        .route("/customer", get(static_files::handle_customer_display))
        .route("/assets/*path", get(static_files::handle_static_asset))
        // WebSocket Realtime Channel
        .route("/ws/pos", get(cfd::handle_ws_pos))
        // POS Core & Catalog
        .route("/api/status", get(pos::handle_status))
        .route("/api/products/quick", get(pos::handle_quick_products))
        .route("/api/products/search", get(pos::handle_search_products))
        .route("/api/barcode/scan", post(pos::handle_barcode_scan))
        .route("/api/checkout", post(pos::handle_checkout))
        .route("/api/sync/trigger", post(pos::handle_sync_trigger))
        .route("/api/sales/history", get(pos::handle_get_sales_history))
        .route("/api/sales/refund", post(pos::handle_process_refund))
        // Parked Carts
        .route("/api/cart/parked", get(pos::handle_get_parked_carts))
        .route("/api/cart/park", post(pos::handle_park_cart))
        .route("/api/cart/parked/:id", delete(pos::handle_delete_parked_cart))
        // Products CRUD & Catalog
        .route("/api/products", get(inventory::handle_get_products))
        .route("/api/products", post(inventory::handle_create_product))
        .route("/api/products/:id", put(inventory::handle_update_product))
        .route("/api/products/:id", delete(inventory::handle_delete_product))
        .route("/api/categories", get(inventory::handle_get_categories))
        // Stock Count Audits
        .route("/api/stock/sessions", get(inventory::handle_get_stock_sessions))
        .route("/api/stock/sessions", post(inventory::handle_start_stock_session))
        .route("/api/stock/sessions/:id/items", get(inventory::handle_get_stock_items))
        .route("/api/stock/sessions/:id/scan", post(inventory::handle_scan_stock_item))
        .route("/api/stock/sessions/:id/commit", post(inventory::handle_commit_stock_session))
        // Waste & Fire Records
        .route("/api/waste", get(inventory::handle_get_waste))
        .route("/api/waste", post(inventory::handle_create_waste))
        // Shelf Labels
        .route("/api/labels", get(inventory::handle_get_shelf_labels))
        // Customers & Veresiye Ledger
        .route("/api/customers", get(customers::handle_customers))
        .route("/api/customers", post(customers::handle_create_customer))
        .route("/api/customers/:id/ledger", get(customers::handle_customer_ledger))
        .route("/api/customers/payment", post(customers::handle_customer_payment))
        // Cash Movements & Z-Report
        .route("/api/reports/z", get(cash::handle_z_report))
        .route("/api/cash/movements", get(cash::handle_get_cash_movements))
        .route("/api/cash/movements", post(cash::handle_create_cash_movement))
        // Customer Facing Display State
        .route("/api/cfd/state", get(cfd::handle_get_cfd_state))
        .route("/api/cfd/update", post(cfd::handle_update_cfd_state))
        // Terminal Payments & Hardware
        .route("/api/terminal/pay", post(terminal::handle_terminal_pay))
        .route("/api/terminal/test", post(terminal::handle_terminal_test))
        .route("/api/terminal/config", get(terminal::handle_get_terminal_config))
        .route("/api/terminal/config", post(terminal::handle_save_terminal_config))
        .route("/api/print/receipt", post(hardware::handle_print_receipt))
        .route("/api/print/drawer", post(hardware::handle_kick_drawer))
        // ---------------------------------------------------------------------
        // Kurumsal Perakende Ek Modülleri (Server ile %100 Uyumlu)
        // ---------------------------------------------------------------------
        // 1. Mal Kabul & Tedarikçiler
        .route("/api/pos/suppliers", get(receiving::handle_get_suppliers).post(receiving::handle_create_supplier))
        .route("/api/pos/receiving", post(receiving::handle_process_receiving))
        .route("/api/pos/receiving/history", get(receiving::handle_receiving_history))
        .route("/api/pos/receiving/:id", get(receiving::handle_receiving_detail))
        // 2. Kampanyalar & İndirimler
        .route("/api/pos/campaigns", get(campaigns::handle_get_campaigns).post(campaigns::handle_create_campaign))
        .route("/api/pos/campaigns/:id/toggle", post(campaigns::handle_toggle_campaign))
        .route("/api/pos/campaigns/:id", delete(campaigns::handle_delete_campaign))
        .route("/api/pos/campaigns/calculate", post(campaigns::handle_calculate_campaign))
        // 3. Şubeler & Mal Transferleri
        .route("/api/pos/branches", get(transfers::handle_get_branches))
        .route("/api/pos/transfers", get(transfers::handle_get_transfers).post(transfers::handle_create_transfer))
        .route("/api/pos/transfers/:id", get(transfers::handle_get_transfer_detail))
        .route("/api/pos/transfers/:id/receive", post(transfers::handle_receive_transfer))
        // 4. Kasa & Vardiya Yönetimi (Açılış, Kapanış, X-Raporu)
        .route("/api/pos/registers/shift/active", get(registers::handle_get_active_shift))
        .route("/api/pos/registers/shift/open", post(registers::handle_open_shift))
        .route("/api/pos/registers/shift/close", post(registers::handle_close_shift))
        .route("/api/pos/registers/x-report", get(registers::handle_x_report))
        .route("/api/pos/registers/shifts", get(registers::handle_shift_history))
        // 5. 4.13 Milyon Ulusal Barkod Kataloğu Arama
        .route("/api/pos/national-catalog/:barcode", get(catalog::handle_national_catalog_lookup))
        .route("/api/pos/national-catalog/search", get(catalog::handle_national_catalog_search))
        .layer(cors)
        .with_state(state)
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
