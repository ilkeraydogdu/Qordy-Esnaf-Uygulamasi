use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub tenant_id: String,
    pub barcode: String,
    pub name: String,
    pub category: String,
    pub price: f64,
    pub vat_rate: f64,       // e.g. 1.0, 10.0, 20.0
    pub stock_qty: f64,
    pub unit: String,         // "adet", "kg", "paket", "koli"
    pub is_weighed: bool,     // true if weighed at scale (manav/kasap)
    pub tare_grams: f64,
    pub quick_order: Option<i32>, // non-null for quick buttons
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: String,
    pub barcode: String,
    pub name: String,
    pub unit_price: f64,
    pub quantity: f64,
    pub unit: String,
    pub vat_rate: f64,
    pub discount: f64,
    pub total: f64,
    pub is_weighed: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub phone: String,
    pub balance: f64,         // Current debt amount (borç tutarı)
    pub credit_limit: f64,    // Maximum allowed debt limit
    pub note: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: String,
    pub tenant_id: String,
    pub customer_id: String,
    pub customer_name: String,
    pub entry_type: String,   // "DEBT" (Borç eklendi) or "PAYMENT" (Tahsilat yapıldı)
    pub amount: f64,
    pub balance_after: f64,
    pub note: Option<String>,
    pub created_at: String,
    pub sync_status: String,  // "PENDING", "SYNCED"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sale {
    pub id: String,           // UUIDv4 generated locally
    pub receipt_no: String,   // E.g. "FS-20261020-0001"
    pub tenant_id: String,
    pub branch_id: String,
    pub branch_name: String,
    pub register_id: String,  // E.g. "KASA-01"
    pub cashier_id: String,
    pub cashier_name: String,
    pub subtotal: f64,
    pub vat_amount: f64,
    pub discount_amount: f64,
    pub grand_total: f64,
    pub payment_method: String, // "CASH", "CREDIT_CARD", "VERESIYE", "SPLIT"
    pub customer_id: Option<String>,
    pub customer_name: Option<String>,
    pub tender_amount: f64,
    pub change_amount: f64,
    pub items: Vec<CartItem>,
    pub created_at: String,
    pub sync_status: String,  // "PENDING", "SYNCED"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitPayment {
    pub method: String, // "CASH", "CREDIT_CARD", "VERESIYE", "BANK_TRANSFER"
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CheckoutRequest {
    pub items: Vec<CartItem>,
    pub payment_method: String,
    pub customer_id: Option<String>,
    pub tender_amount: Option<f64>,
    pub discount_amount: Option<f64>,
    pub split_payments: Option<Vec<SplitPayment>>,
    pub campaign_code: Option<String>,
    pub is_efatura: Option<bool>,
    pub tax_id_or_tckn: Option<String>,
    pub company_title: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutResponse {
    pub success: bool,
    pub sale_id: String,
    pub receipt_no: String,
    pub grand_total: f64,
    pub change_amount: f64,
    pub sync_status: String,
    pub printed: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZReport {
    pub z_number: String,
    pub branch_name: String,
    pub register_id: String,
    pub cashier_name: String,
    pub date: String,
    pub total_sales_count: usize,
    pub total_gross: f64,
    pub total_discount: f64,
    pub total_net: f64,
    pub cash_total: f64,
    pub card_total: f64,
    pub veresiye_total: f64,
    pub vat_breakdown: Vec<VatSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VatSummary {
    pub rate: f64,
    pub taxable_amount: f64,
    pub vat_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosStatus {
    pub version: String,
    pub tenant_id: String,
    pub business_name: String,
    pub branch_id: String,
    pub branch_name: String,
    pub register_id: String,
    pub cashier_name: String,
    pub is_online: bool,
    pub pending_sync_count: usize,
    pub today_sales_count: usize,
    pub today_revenue: f64,
    pub local_ip: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfdState {
    pub state: String, // "IDLE", "ACTIVE_SALE", "PAYMENT_COMPLETED"
    pub items: Vec<CartItem>,
    pub subtotal: f64,
    pub discount: f64,
    pub grand_total: f64,
    pub tender: f64,
    pub change: f64,
    pub payment_method: Option<String>,
    pub customer_name: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockCountSession {
    pub id: String,
    pub title: String,
    pub count_type: String, // "FULL", "CATEGORY", "SPOT"
    pub status: String,     // "OPEN", "COMMITTED", "CANCELLED"
    pub started_at: String,
    pub completed_at: Option<String>,
    pub total_items_counted: usize,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockCountItem {
    pub id: String,
    pub session_id: String,
    pub product_id: String,
    pub barcode: String,
    pub product_name: String,
    pub system_qty: f64,
    pub counted_qty: f64,
    pub variance_qty: f64, // counted_qty - system_qty (negatif = eksik, pozitif = fazla)
    pub unit: String,
    pub counted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParkedCart {
    pub id: String,
    pub note: String,
    pub items: Vec<CartItem>,
    pub grand_total: f64,
    pub item_count: usize,
    pub parked_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub barcode: String,
    pub name: String,
    pub category: String,
    pub price: f64,
    pub vat_rate: f64,
    pub stock_qty: f64,
    pub unit: String,
    pub is_weighed: bool,
    pub quick_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCustomerRequest {
    pub name: String,
    pub phone: String,
    pub credit_limit: f64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasteRecord {
    pub id: String,
    pub product_id: String,
    pub product_name: String,
    pub barcode: String,
    pub quantity: f64,
    pub unit: String,
    pub cost_loss: f64,
    pub reason: String, // "BAYAT / SKT", "KIRILDI / HASAR", "ÇÜRÜME", "DİĞER"
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWasteRequest {
    pub barcode: String,
    pub quantity: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundItem {
    pub product_id: String,
    pub barcode: String,
    pub name: String,
    pub unit_price: f64,
    pub quantity: f64,
    pub unit: String,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRequest {
    pub sale_id: String,
    pub items: Vec<RefundItem>,
    pub refund_amount: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
    pub success: bool,
    pub refund_id: String,
    pub refund_receipt_no: String,
    pub refunded_amount: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashMovement {
    pub id: String,
    pub movement_type: String, // "IN" (Kasa Giriş / Avans) veya "OUT" (Kasa Çıkış / Masraf)
    pub amount: f64,
    pub reason: String,
    pub register_id: String,
    pub cashier_name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCashMovementRequest {
    pub movement_type: String,
    pub amount: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShelfLabel {
    pub product_id: String,
    pub barcode: String,
    pub name: String,
    pub price: f64,
    pub unit: String,
    pub vat_rate: f64,
    pub category: String,
}

// -------------------------------------------------------------
// 1. Tedarikçiler & Mal Kabul / Alış İrsaliyesi
// -------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supplier {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub contact_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSupplierRequest {
    pub name: String,
    pub contact_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodsReceipt {
    pub id: String,
    pub tenant_id: String,
    pub branch_id: String,
    pub supplier_id: String,
    pub supplier_name: String,
    pub invoice_no: String,
    pub receipt_date: String,
    pub total_amount: f64,
    pub notes: Option<String>,
    pub items: Vec<GoodsReceiptItem>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodsReceiptItem {
    pub product_id: String,
    pub barcode: String,
    pub name: String,
    pub quantity: f64,
    pub unit: String,
    pub cost_price: f64,
    pub sale_price: f64,
    pub total_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGoodsReceiptRequest {
    pub supplier_id: String,
    pub supplier_name: String,
    pub invoice_no: String,
    pub receipt_date: String,
    pub notes: Option<String>,
    pub items: Vec<GoodsReceiptItem>,
}

// -------------------------------------------------------------
// 2. Kampanyalar & İndirimler
// -------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub campaign_type: String, // "PERCENT", "AMOUNT", "BUY_X_PAY_Y"
    pub discount_percent: Option<f64>,
    pub discount_amount: Option<f64>,
    pub min_cart_amount: Option<f64>,
    pub buy_qty: Option<i32>,
    pub pay_qty: Option<i32>,
    pub target_product_id: Option<String>,
    pub target_category: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub campaign_type: String,
    pub discount_percent: Option<f64>,
    pub discount_amount: Option<f64>,
    pub min_cart_amount: Option<f64>,
    pub buy_qty: Option<i32>,
    pub pay_qty: Option<i32>,
    pub target_product_id: Option<String>,
    pub target_category: Option<String>,
}

// -------------------------------------------------------------
// 3. Şubeler & Şubeler Arası Mal Transferi
// -------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub code: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockTransfer {
    pub id: String,
    pub tenant_id: String,
    pub from_branch_id: String,
    pub from_branch_name: String,
    pub to_branch_id: String,
    pub to_branch_name: String,
    pub status: String, // "DRAFT", "DISPATCHED", "RECEIVED", "CANCELLED"
    pub notes: Option<String>,
    pub created_by: String,
    pub items: Vec<StockTransferItem>,
    pub created_at: String,
    pub received_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockTransferItem {
    pub product_id: String,
    pub barcode: String,
    pub name: String,
    pub quantity: f64,
    pub unit: String,
    pub received_quantity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStockTransferRequest {
    pub from_branch_id: String,
    pub from_branch_name: String,
    pub to_branch_id: String,
    pub to_branch_name: String,
    pub notes: Option<String>,
    pub items: Vec<StockTransferItem>,
}

// -------------------------------------------------------------
// 4. Kasa Vardiyaları & Çekmece Devri
// -------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterShift {
    pub id: String,
    pub tenant_id: String,
    pub branch_id: String,
    pub register_name: String,
    pub cashier_name: String,
    pub opening_cash: f64,
    pub closing_cash: Option<f64>,
    pub expected_cash: Option<f64>,
    pub cash_difference: Option<f64>,
    pub notes: Option<String>,
    pub status: String, // "OPEN", "CLOSED"
    pub opened_at: String,
    pub closed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenShiftRequest {
    pub register_name: String,
    pub cashier_name: String,
    pub opening_cash: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseShiftRequest {
    pub counted_cash: f64,
    pub notes: Option<String>,
}

// -------------------------------------------------------------
// 5. 4.13 Milyon Ulusal Barkod Kataloğu Arama Sonucu
// -------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NationalProductInfo {
    pub barcode: String,
    pub name: String,
    pub brand: Option<String>,
    pub category: Option<String>,
    pub vat_rate: Option<f64>,
    pub unit: String,
    pub is_weighed: bool,
    pub plu_code: Option<String>,
    pub suggested_price: Option<f64>,
    pub source: String, // "NATIONAL_CATALOG_4.13M"
}

