//! QORDY POS - Enterprise SQLite Storage Architecture
//! WAL-mode, offline-first transactional storage engine with outbox pattern.

pub mod campaigns;
pub mod cash;
pub mod catalog;
pub mod cfd;
pub mod customers;
pub mod labels;
pub mod outbox;
pub mod products;
pub mod receiving;
pub mod registers;
pub mod reports;
pub mod sales;
pub mod stock_count;
pub mod transfers;
pub mod waste;

use crate::config::AppConfig;
use crate::models::{CartItem, Customer, LedgerEntry, Product, Sale};
use anyhow::Result;
use chrono::Local;
use parking_lot::Mutex;
use rusqlite::{params, Connection, Row};
use std::path::Path;
use std::sync::Arc;
use tracing::info;

// ---------------------------------------------------------------------------
// Shared row-mapping helpers — eliminates duplicated field-by-field mapping
// across products.rs, customers.rs, sales.rs, etc.
// ---------------------------------------------------------------------------

pub(crate) fn row_to_product(row: &Row) -> rusqlite::Result<Product> {
    Ok(Product {
        id: row.get(0)?,
        tenant_id: row.get(1)?,
        barcode: row.get(2)?,
        name: row.get(3)?,
        category: row.get(4)?,
        price: row.get(5)?,
        vat_rate: row.get(6)?,
        stock_qty: row.get(7)?,
        unit: row.get(8)?,
        is_weighed: row.get::<_, i32>(9)? == 1,
        tare_grams: row.get(10)?,
        quick_order: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

#[allow(dead_code)]
pub(crate) fn row_to_customer(row: &Row) -> rusqlite::Result<Customer> {
    Ok(Customer {
        id: row.get(0)?,
        tenant_id: row.get(1)?,
        name: row.get(2)?,
        phone: row.get(3)?,
        balance: row.get(4)?,
        credit_limit: row.get(5)?,
        note: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

#[allow(dead_code)]
pub(crate) fn row_to_ledger_entry(row: &Row) -> rusqlite::Result<LedgerEntry> {
    Ok(LedgerEntry {
        id: row.get(0)?,
        tenant_id: row.get(1)?,
        customer_id: row.get(2)?,
        customer_name: row.get(3)?,
        entry_type: row.get(4)?,
        amount: row.get(5)?,
        balance_after: row.get(6)?,
        note: row.get(7)?,
        created_at: row.get(8)?,
        sync_status: row.get(9)?,
    })
}

#[allow(dead_code)]
pub(crate) fn row_to_sale(row: &Row) -> rusqlite::Result<Sale> {
    Ok(Sale {
        id: row.get(0)?,
        receipt_no: row.get(1)?,
        tenant_id: row.get(2)?,
        branch_id: row.get(3)?,
        branch_name: row.get(4)?,
        register_id: row.get(5)?,
        cashier_id: row.get(6)?,
        cashier_name: row.get(7)?,
        subtotal: row.get(8)?,
        vat_amount: row.get(9)?,
        discount_amount: row.get(10)?,
        grand_total: row.get(11)?,
        payment_method: row.get(12)?,
        customer_id: row.get(13)?,
        customer_name: row.get(14)?,
        tender_amount: row.get(15)?,
        change_amount: row.get(16)?,
        items: Vec::new(),
        created_at: row.get(17)?,
        sync_status: row.get(18)?,
    })
}

#[allow(dead_code)]
pub(crate) fn row_to_cart_item(row: &Row) -> rusqlite::Result<CartItem> {
    Ok(CartItem {
        product_id: row.get(0)?,
        barcode: row.get(1)?,
        name: row.get(2)?,
        unit_price: row.get(3)?,
        quantity: row.get(4)?,
        unit: row.get(5)?,
        vat_rate: row.get(6)?,
        discount: row.get(7)?,
        total: row.get(8)?,
        is_weighed: row.get::<_, i32>(9)? == 1,
        notes: row.get(10)?,
    })
}

pub struct DatabaseManager {
    pub(crate) conn: Arc<Mutex<Connection>>,
    pub(crate) config: AppConfig,
}

impl DatabaseManager {
    #[allow(dead_code)]
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Arc<Self>> {
        Self::new_with_config(path, &AppConfig::default())
    }

    pub fn new_with_config<P: AsRef<Path>>(path: P, config: &AppConfig) -> Result<Arc<Self>> {
        let conn = Connection::open(path)?;

        // WAL (Write-Ahead Logging) modu: eşzamanlı okuma/yazma performansı ve veri kaybı koruması
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;",
        )?;

        let db = Arc::new(Self {
            conn: Arc::new(Mutex::new(conn)),
            config: config.clone(),
        });

        db.init_schema()?;
        db.seed_initial_data_if_empty()?;

        Ok(db)
    }

    #[allow(dead_code)]
    pub fn new_in_memory() -> Result<Arc<Self>> {
        Self::new_in_memory_with_config(&AppConfig::default())
    }

    #[allow(dead_code)]
    pub fn new_in_memory_with_config(config: &AppConfig) -> Result<Arc<Self>> {
        let conn = Connection::open_in_memory()?;
        let db = Arc::new(Self {
            conn: Arc::new(Mutex::new(conn)),
            config: config.clone(),
        });
        db.init_schema()?;
        db.seed_initial_data_if_empty()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock();

        conn.execute_batch(
            "
            -- Ürünler Tablosu (Lokal Katalog)
            CREATE TABLE IF NOT EXISTS products (
                id TEXT PRIMARY KEY,
                tenant_id TEXT NOT NULL,
                barcode TEXT NOT NULL,
                name TEXT NOT NULL,
                category TEXT NOT NULL,
                price REAL NOT NULL,
                vat_rate REAL NOT NULL DEFAULT 10.0,
                stock_qty REAL NOT NULL DEFAULT 100.0,
                unit TEXT NOT NULL DEFAULT 'adet',
                is_weighed INTEGER NOT NULL DEFAULT 0,
                tare_grams REAL NOT NULL DEFAULT 0.0,
                quick_order INTEGER,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_products_barcode ON products(barcode);
            CREATE INDEX IF NOT EXISTS idx_products_name ON products(name);
            CREATE INDEX IF NOT EXISTS idx_products_quick ON products(quick_order);

            -- Çoklu Barkodlar (Koli, Adet, Tedarikçi)
            CREATE TABLE IF NOT EXISTS barcodes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                product_id TEXT NOT NULL,
                barcode TEXT NOT NULL UNIQUE,
                multiplier REAL NOT NULL DEFAULT 1.0,
                FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_barcodes_code ON barcodes(barcode);

            -- Veresiye Müşterileri
            CREATE TABLE IF NOT EXISTS customers (
                id TEXT PRIMARY KEY,
                tenant_id TEXT NOT NULL,
                name TEXT NOT NULL,
                phone TEXT NOT NULL,
                balance REAL NOT NULL DEFAULT 0.0,
                credit_limit REAL NOT NULL DEFAULT 2000.0,
                note TEXT,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_customers_name ON customers(name);
            CREATE INDEX IF NOT EXISTS idx_customers_phone ON customers(phone);

            -- Veresiye Cari Hareketleri
            CREATE TABLE IF NOT EXISTS ledger_entries (
                id TEXT PRIMARY KEY,
                tenant_id TEXT NOT NULL,
                customer_id TEXT NOT NULL,
                customer_name TEXT NOT NULL,
                entry_type TEXT NOT NULL,
                amount REAL NOT NULL,
                balance_after REAL NOT NULL,
                note TEXT,
                created_at TEXT NOT NULL,
                sync_status TEXT NOT NULL DEFAULT 'PENDING'
            );
            CREATE INDEX IF NOT EXISTS idx_ledger_cust ON ledger_entries(customer_id);

            -- Satışlar (Kasa Fişleri)
            CREATE TABLE IF NOT EXISTS sales (
                id TEXT PRIMARY KEY,
                receipt_no TEXT NOT NULL UNIQUE,
                tenant_id TEXT NOT NULL,
                branch_id TEXT NOT NULL,
                branch_name TEXT NOT NULL,
                register_id TEXT NOT NULL,
                cashier_id TEXT NOT NULL,
                cashier_name TEXT NOT NULL,
                subtotal REAL NOT NULL,
                vat_amount REAL NOT NULL,
                discount_amount REAL NOT NULL,
                grand_total REAL NOT NULL,
                payment_method TEXT NOT NULL,
                customer_id TEXT,
                customer_name TEXT,
                tender_amount REAL NOT NULL DEFAULT 0.0,
                change_amount REAL NOT NULL DEFAULT 0.0,
                created_at TEXT NOT NULL,
                sync_status TEXT NOT NULL DEFAULT 'PENDING'
            );
            CREATE INDEX IF NOT EXISTS idx_sales_sync ON sales(sync_status);
            CREATE INDEX IF NOT EXISTS idx_sales_created ON sales(created_at);

            -- Satış Kalemleri
            CREATE TABLE IF NOT EXISTS sale_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                sale_id TEXT NOT NULL,
                product_id TEXT NOT NULL,
                barcode TEXT NOT NULL,
                name TEXT NOT NULL,
                unit_price REAL NOT NULL,
                quantity REAL NOT NULL,
                unit TEXT NOT NULL,
                vat_rate REAL NOT NULL,
                discount REAL NOT NULL DEFAULT 0.0,
                total REAL NOT NULL,
                is_weighed INTEGER NOT NULL DEFAULT 0,
                notes TEXT,
                FOREIGN KEY (sale_id) REFERENCES sales(id) ON DELETE CASCADE
            );

            -- Outbox Senkronizasyon Kuyruğu (Çevrimdışı Hareketler)
            CREATE TABLE IF NOT EXISTS outbox_sync (
                id TEXT PRIMARY KEY,
                entity_type TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                idempotency_key TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                retry_count INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'PENDING'
            );

            -- Kasa Vardiyaları & Gün Sonu
            CREATE TABLE IF NOT EXISTS shifts (
                id TEXT PRIMARY KEY,
                register_id TEXT NOT NULL,
                opened_at TEXT NOT NULL,
                closed_at TEXT,
                opening_cash REAL NOT NULL DEFAULT 0.0,
                status TEXT NOT NULL DEFAULT 'OPEN'
            );

            -- Sistem Konfigürasyonu
            CREATE TABLE IF NOT EXISTS app_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            -- Bekletilen / Park Edilen Sepetler
            CREATE TABLE IF NOT EXISTS parked_carts (
                id TEXT PRIMARY KEY,
                note TEXT NOT NULL,
                items_json TEXT NOT NULL,
                grand_total REAL NOT NULL,
                item_count INTEGER NOT NULL,
                parked_at TEXT NOT NULL
            );

            -- Stok Sayım Oturumları
            CREATE TABLE IF NOT EXISTS stock_count_sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                count_type TEXT NOT NULL DEFAULT 'FULL',
                status TEXT NOT NULL DEFAULT 'OPEN',
                started_at TEXT NOT NULL,
                completed_at TEXT,
                notes TEXT
            );

            -- Stok Sayım Kalemleri
            CREATE TABLE IF NOT EXISTS stock_count_items (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                product_id TEXT NOT NULL,
                barcode TEXT NOT NULL,
                product_name TEXT NOT NULL,
                system_qty REAL NOT NULL,
                counted_qty REAL NOT NULL,
                variance_qty REAL NOT NULL,
                unit TEXT NOT NULL,
                counted_at TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES stock_count_sessions(id) ON DELETE CASCADE
            );

            -- Müşteri Ekranı (CFD - Customer Facing Display) Canlı Durumu
            CREATE TABLE IF NOT EXISTS cfd_state (
                id TEXT PRIMARY KEY,
                state TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Fire / Zayi Takibi
            CREATE TABLE IF NOT EXISTS waste_records (
                id TEXT PRIMARY KEY,
                product_id TEXT NOT NULL,
                product_name TEXT NOT NULL,
                barcode TEXT NOT NULL,
                quantity REAL NOT NULL,
                unit TEXT NOT NULL,
                cost_loss REAL NOT NULL,
                reason TEXT NOT NULL,
                recorded_at TEXT NOT NULL
            );

            -- İade İşlemleri (Kısmi & Tam İade)
            CREATE TABLE IF NOT EXISTS refunds (
                id TEXT PRIMARY KEY,
                sale_id TEXT NOT NULL,
                refund_receipt_no TEXT NOT NULL UNIQUE,
                items_json TEXT NOT NULL,
                refund_amount REAL NOT NULL,
                reason TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            -- Kasa Para Giriş / Çıkış Hareketleri (Masraf / Avans)
            CREATE TABLE IF NOT EXISTS cash_movements (
                id TEXT PRIMARY KEY,
                movement_type TEXT NOT NULL,
                amount REAL NOT NULL,
                reason TEXT NOT NULL,
                register_id TEXT NOT NULL,
                cashier_name TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            -- Tedarikçiler
            CREATE TABLE IF NOT EXISTS suppliers (
                id TEXT PRIMARY KEY,
                tenant_id TEXT NOT NULL,
                name TEXT NOT NULL,
                contact_name TEXT,
                phone TEXT,
                email TEXT,
                address TEXT,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_suppliers_name ON suppliers(name);

            -- Mal Kabul & Alış İrsaliyeleri
            CREATE TABLE IF NOT EXISTS goods_receipts (
                id TEXT PRIMARY KEY,
                tenant_id TEXT NOT NULL,
                branch_id TEXT NOT NULL,
                supplier_id TEXT NOT NULL,
                supplier_name TEXT NOT NULL,
                invoice_no TEXT NOT NULL,
                receipt_date TEXT NOT NULL,
                total_amount REAL NOT NULL,
                notes TEXT,
                created_at TEXT NOT NULL
            );

            -- Mal Kabul Kalemleri
            CREATE TABLE IF NOT EXISTS goods_receipt_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                receipt_id TEXT NOT NULL,
                product_id TEXT NOT NULL,
                barcode TEXT NOT NULL,
                name TEXT NOT NULL,
                quantity REAL NOT NULL,
                unit TEXT NOT NULL,
                cost_price REAL NOT NULL,
                sale_price REAL NOT NULL,
                total_cost REAL NOT NULL,
                FOREIGN KEY (receipt_id) REFERENCES goods_receipts(id) ON DELETE CASCADE
            );

            -- Kampanyalar & İndirimler
            CREATE TABLE IF NOT EXISTS campaigns (
                id TEXT PRIMARY KEY,
                tenant_id TEXT NOT NULL,
                name TEXT NOT NULL,
                campaign_type TEXT NOT NULL,
                discount_percent REAL,
                discount_amount REAL,
                min_cart_amount REAL,
                buy_qty INTEGER,
                pay_qty INTEGER,
                target_product_id TEXT,
                target_category TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            );

            -- Şubeler
            CREATE TABLE IF NOT EXISTS branches (
                id TEXT PRIMARY KEY,
                tenant_id TEXT NOT NULL,
                name TEXT NOT NULL,
                code TEXT,
                address TEXT,
                phone TEXT,
                is_active INTEGER NOT NULL DEFAULT 1
            );

            -- Şubeler Arası Mal Transferleri
            CREATE TABLE IF NOT EXISTS stock_transfers (
                id TEXT PRIMARY KEY,
                tenant_id TEXT NOT NULL,
                from_branch_id TEXT NOT NULL,
                from_branch_name TEXT NOT NULL,
                to_branch_id TEXT NOT NULL,
                to_branch_name TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'DRAFT',
                notes TEXT,
                created_by TEXT NOT NULL,
                created_at TEXT NOT NULL,
                received_at TEXT
            );

            -- Transfer Kalemleri
            CREATE TABLE IF NOT EXISTS stock_transfer_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                transfer_id TEXT NOT NULL,
                product_id TEXT NOT NULL,
                barcode TEXT NOT NULL,
                name TEXT NOT NULL,
                quantity REAL NOT NULL,
                unit TEXT NOT NULL,
                received_quantity REAL NOT NULL DEFAULT 0.0,
                FOREIGN KEY (transfer_id) REFERENCES stock_transfers(id) ON DELETE CASCADE
            );

            -- Kasa Vardiyaları & Çekmece Devri
            CREATE TABLE IF NOT EXISTS register_shifts (
                id TEXT PRIMARY KEY,
                tenant_id TEXT NOT NULL,
                branch_id TEXT NOT NULL,
                register_name TEXT NOT NULL,
                cashier_name TEXT NOT NULL,
                opening_cash REAL NOT NULL DEFAULT 0.0,
                closing_cash REAL,
                expected_cash REAL,
                cash_difference REAL,
                notes TEXT,
                status TEXT NOT NULL DEFAULT 'OPEN',
                opened_at TEXT NOT NULL,
                closed_at TEXT
            );

            -- 4.13 Milyon Ulusal Barkod Kataloğu Önbelleği
            CREATE TABLE IF NOT EXISTS national_catalog_cache (
                barcode TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                brand TEXT,
                category TEXT,
                vat_rate REAL,
                unit TEXT,
                is_weighed INTEGER DEFAULT 0,
                plu_code TEXT,
                suggested_price REAL
            );
            CREATE INDEX IF NOT EXISTS idx_national_name ON national_catalog_cache(name);
            ",
        )?;

        Ok(())
    }

    fn seed_initial_data_if_empty(&self) -> Result<()> {
        let conn = self.conn.lock();

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM products", [], |r| r.get(0))?;
        if count > 0 {
            return Ok(());
        }

        info!("Lokal SQLite veritabanı ilk kez oluşturuldu, örnek esnaf ürünleri ve cariler yükleniyor...");

        let sample_products = vec![
            ("PRD-001", "8690504018087", "Somun Ekmek (200g)", "Temel Gıda", 10.0, 1.0, 150.0, "adet", 0, 0.0, Some(1)),
            ("PRD-002", "8690504018094", "Erikli Doğal Kaynak Suyu 0.5L", "İçecek", 7.5, 10.0, 240.0, "adet", 0, 0.0, Some(2)),
            ("PRD-003", "8690504018100", "Susamlı Simit", "Unlu Mamüller", 15.0, 1.0, 80.0, "adet", 0, 0.0, Some(3)),
            ("PRD-004", "8690504018117", "Köy Yumurtası (30'lu Koli)", "Temel Gıda", 145.0, 1.0, 40.0, "adet", 0, 0.0, Some(4)),
            ("PRD-005", "8690504018124", "Çaykur Rize Turist Çayı 1000g", "Gıda", 210.0, 1.0, 50.0, "adet", 0, 0.0, Some(5)),
            ("PRD-006", "8690504018131", "Sütaş Tam Yağlı Süt 1L", "Süt Ürünleri", 38.0, 1.0, 60.0, "adet", 0, 0.0, Some(6)),
            ("PRD-007", "8690504018148", "Standart Alışveriş Poşeti", "Ambalaj", 0.50, 20.0, 1000.0, "adet", 0, 0.0, Some(7)),
            ("PRD-008", "2700124", "Salkım Domates (Yerli)", "Manav", 42.0, 1.0, 250.0, "kg", 1, 10.0, Some(8)),
            ("PRD-009", "2700125", "Çengelköy Salatalık", "Manav", 36.0, 1.0, 180.0, "kg", 1, 10.0, Some(9)),
            ("PRD-010", "2700126", "Amasya Elması", "Manav", 34.0, 1.0, 200.0, "kg", 1, 10.0, Some(10)),
            ("PRD-011", "8690504018155", "Coca Cola 1L Şişe", "İçecek", 35.0, 20.0, 90.0, "adet", 0, 0.0, None),
            ("PRD-012", "8690504018162", "Ülker Çikolatalı Gofret", "Atıştırmalık", 12.0, 10.0, 120.0, "adet", 0, 0.0, None),
        ];

        let now = Local::now().to_rfc3339();
        let tenant_id = &self.config.tenant_id;
        for (id, barcode, name, cat, price, vat, stock, unit, weighed, tare, quick) in sample_products {
            conn.execute(
                "INSERT INTO products (id, tenant_id, barcode, name, category, price, vat_rate, stock_qty, unit, is_weighed, tare_grams, quick_order, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![id, tenant_id, barcode, name, cat, price, vat, stock, unit, weighed, tare, quick, now],
            )?;
        }

        let sample_customers = vec![
            ("CUST-001", "Komşu Ahmet Bey", "0532 111 22 33", 450.0, 2500.0, "A Blok D:4"),
            ("CUST-002", "Mehmet Usta (Oto Tamir)", "0544 222 33 44", 1250.0, 5000.0, "Köşe Dükkan"),
            ("CUST-003", "Ayşe Teyze", "0505 333 44 55", 85.0, 1000.0, "Emekli Öğretmen"),
        ];

        for (id, name, phone, balance, limit, note) in sample_customers {
            conn.execute(
                "INSERT INTO customers (id, tenant_id, name, phone, balance, credit_limit, note, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![id, tenant_id, name, phone, balance, limit, note, now],
            )?;
        }

        let default_configs = vec![
            ("tenant_id", self.config.tenant_id.as_str()),
            ("business_name", self.config.business_name.as_str()),
            ("branch_id", self.config.branch_id.as_str()),
            ("branch_name", self.config.branch_name.as_str()),
            ("register_id", self.config.register_id.as_str()),
            ("cashier_name", self.config.cashier_name.as_str()),
            ("cloud_api_url", self.config.cloud_url.as_str()),
            ("api_key", self.config.api_key.as_str()),
        ];

        for (k, v) in default_configs {
            conn.execute(
                "INSERT OR REPLACE INTO app_config (key, value) VALUES (?1, ?2)",
                params![k, v],
            )?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_app_config(&self, key: &str) -> Option<String> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT value FROM app_config WHERE key = ?1",
            params![key],
            |r| r.get(0),
        )
        .ok()
    }

    #[allow(dead_code)]
    pub fn set_app_config(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO app_config (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
}
