use crate::models::{
    CartItem, CashMovement, CfdState, CheckoutRequest, CreateCustomerRequest, CreateProductRequest,
    CreateWasteRequest, Customer, LedgerEntry, ParkedCart, PosStatus, Product,
    RefundRequest, RefundResponse, Sale, ShelfLabel, StockCountItem, StockCountSession,
    VatSummary, WasteRecord, ZReport,
};
use anyhow::Result;
use chrono::Local;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub struct DatabaseManager {
    conn: Mutex<Connection>,
}

impl DatabaseManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Arc<Self>> {
        let conn = Connection::open(path)?;

        // WAL (Write-Ahead Logging) modu: eşzamanlı okuma/yazma performansı ve veri kaybı koruması
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;",
        )?;

        let db = Arc::new(Self {
            conn: Mutex::new(conn),
        });

        db.init_schema()?;
        db.seed_initial_data_if_empty()?;

        Ok(db)
    }

    #[allow(dead_code)]
    pub fn new_in_memory() -> Result<Arc<Self>> {
        let conn = Connection::open_in_memory()?;
        let db = Arc::new(Self {
            conn: Mutex::new(conn),
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
                entry_type TEXT NOT NULL, -- 'DEBT' veya 'PAYMENT'
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
                entity_type TEXT NOT NULL, -- 'SALE' veya 'LEDGER'
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

        // Örnek Esnaf & Bakkal & Manav Ürünleri
        let sample_products = vec![
            ("PRD-001", "8690504018087", "Somun Ekmek (200g)", "Temel Gıda", 10.0, 1.0, 150.0, "adet", 0, 0.0, Some(1)),
            ("PRD-002", "8690504018094", "Erikli Doğal Kaynak Suyu 0.5L", "İçecek", 7.5, 10.0, 240.0, "adet", 0, 0.0, Some(2)),
            ("PRD-003", "8690504018100", "Susamlı Simit", "Unlu Mamüller", 15.0, 1.0, 80.0, "adet", 0, 0.0, Some(3)),
            ("PRD-004", "8690504018117", "Köy Yumurtası (30'lu Koli)", "Temel Gıda", 145.0, 1.0, 40.0, "adet", 0, 0.0, Some(4)),
            ("PRD-005", "8690504018124", "Çaykur Rize Turist Çayı 1000g", "Gıda", 210.0, 1.0, 50.0, "adet", 0, 0.0, Some(5)),
            ("PRD-006", "8690504018131", "Sütaş Tam Yağlı Süt 1L", "Süt Ürünleri", 38.0, 1.0, 60.0, "adet", 0, 0.0, Some(6)),
            ("PRD-007", "8690504018148", "Standart Alışveriş Poşeti", "Ambalaj", 0.50, 20.0, 1000.0, "adet", 0, 0.0, Some(7)),
            // Terazi / Manav Ürünleri (2700124 = Domates, 2700125 = Salatalık, 2700126 = Elma)
            ("PRD-008", "2700124", "Salkım Domates (Yerli)", "Manav", 42.0, 1.0, 250.0, "kg", 1, 10.0, Some(8)),
            ("PRD-009", "2700125", "Çengelköy Salatalık", "Manav", 36.0, 1.0, 180.0, "kg", 1, 10.0, Some(9)),
            ("PRD-010", "2700126", "Amasya Elması", "Manav", 34.0, 1.0, 200.0, "kg", 1, 10.0, Some(10)),
            ("PRD-011", "8690504018155", "Coca Cola 1L Şişe", "İçecek", 35.0, 20.0, 90.0, "adet", 0, 0.0, None),
            ("PRD-012", "8690504018162", "Ülker Çikolatalı Gofret", "Atıştırmalık", 12.0, 10.0, 120.0, "adet", 0, 0.0, None),
        ];

        let now = Local::now().to_rfc3339();
        for (id, barcode, name, cat, price, vat, stock, unit, weighed, tare, quick) in sample_products {
            conn.execute(
                "INSERT INTO products (id, tenant_id, barcode, name, category, price, vat_rate, stock_qty, unit, is_weighed, tare_grams, quick_order, updated_at)
                 VALUES (?1, 'CUST_69653cfccb839', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![id, barcode, name, cat, price, vat, stock, unit, weighed, tare, quick, now],
            )?;
        }

        // Örnek Veresiye Müşterileri
        let sample_customers = vec![
            ("CUST-001", "Komşu Ahmet Bey", "0532 111 22 33", 450.0, 2500.0, "A Blok D:4"),
            ("CUST-002", "Mehmet Usta (Oto Tamir)", "0544 222 33 44", 1250.0, 5000.0, "Köşe Dükkan"),
            ("CUST-003", "Ayşe Teyze", "0505 333 44 55", 85.0, 1000.0, "Emekli Öğretmen"),
        ];

        for (id, name, phone, balance, limit, note) in sample_customers {
            conn.execute(
                "INSERT INTO customers (id, tenant_id, name, phone, balance, credit_limit, note, updated_at)
                 VALUES (?1, 'CUST_69653cfccb839', ?2, ?3, ?4, ?5, ?6, ?7)",
                params![id, name, phone, balance, limit, note, now],
            )?;
        }

        // Başlangıç Konfigürasyonu
        let default_configs = vec![
            ("tenant_id", "CUST_69653cfccb839"),
            ("business_name", "QORDY Cadde Esnaf"),
            ("branch_id", "BR_01"),
            ("branch_name", "Kadıköy Çarşı Şubesi"),
            ("register_id", "KASA-01"),
            ("cashier_name", "Ahmet Kasiyer"),
            ("cloud_api_url", "https://qordy.com/api/retail"),
            ("api_key", "42ac390b9f93aa624d7ed72b6ca603e408a9de3d8f859218c157019db3ff6700"),
        ];

        for (k, v) in default_configs {
            conn.execute(
                "INSERT OR REPLACE INTO app_config (key, value) VALUES (?1, ?2)",
                params![k, v],
            )?;
        }

        Ok(())
    }

    /// Barkod ile Ürün Bul (Önce tam barkod, yoksa terazi lookup_code)
    pub fn find_product_by_barcode(&self, barcode_or_lookup: &str) -> Option<Product> {
        let conn = self.conn.lock();

        let sql = "SELECT id, tenant_id, barcode, name, category, price, vat_rate, stock_qty, unit, is_weighed, tare_grams, quick_order, updated_at
                   FROM products WHERE barcode = ?1 OR barcode = ?2 LIMIT 1";

        let mut stmt = conn.prepare(sql).ok()?;
        let clean = barcode_or_lookup.trim();

        let product = stmt.query_row(params![clean, clean.trim_start_matches('0')], |row| {
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
        }).ok();

        if product.is_some() {
            return product;
        }

        // İkincil barkodlar tablosundan ara
        let alt_sql = "SELECT p.id, p.tenant_id, p.barcode, p.name, p.category, (p.price * b.multiplier), p.vat_rate, p.stock_qty, p.unit, p.is_weighed, p.tare_grams, p.quick_order, p.updated_at
                       FROM barcodes b JOIN products p ON b.product_id = p.id WHERE b.barcode = ?1 LIMIT 1";
        let mut alt_stmt = conn.prepare(alt_sql).ok()?;
        alt_stmt.query_row(params![clean], |row| {
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
        }).ok()
    }

    /// Sık Satılan Hızlı Buton Ürünleri
    pub fn get_quick_products(&self) -> Vec<Product> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, tenant_id, barcode, name, category, price, vat_rate, stock_qty, unit, is_weighed, tare_grams, quick_order, updated_at
             FROM products WHERE quick_order IS NOT NULL ORDER BY quick_order ASC"
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map([], |row| {
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
        });

        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Ürün Arama (İsim veya Barkod)
    pub fn search_products(&self, query: &str) -> Vec<Product> {
        let conn = self.conn.lock();
        let pattern = format!("%{}%", query.trim());
        let mut stmt = match conn.prepare(
            "SELECT id, tenant_id, barcode, name, category, price, vat_rate, stock_qty, unit, is_weighed, tare_grams, quick_order, updated_at
             FROM products WHERE name LIKE ?1 OR barcode LIKE ?1 ORDER BY name ASC LIMIT 30"
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map(params![pattern], |row| {
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
        });

        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Veresiye Müşteri Listesi
    pub fn get_customers(&self) -> Vec<Customer> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, tenant_id, name, phone, balance, credit_limit, note, updated_at
             FROM customers ORDER BY name ASC"
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map([], |row| {
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
        });

        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Çevrimdışı Satışı Kaydet (Atomik Transaction: Satış + Stok Düşümü + Veresiye + Outbox Kuyruğu)
    pub fn process_checkout(&self, req: CheckoutRequest) -> Result<Sale> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;

        let now = Local::now();
        let sale_id = Uuid::new_v4().to_string();
        let receipt_no = format!(
            "ESN-{}-{:06}",
            now.format("%Y%m%d%H%M%S"),
            now.timestamp_subsec_micros()
        );

        let tenant_id = "CUST_69653cfccb839".to_string();
        let branch_id = "BR_01".to_string();
        let branch_name = "Kadıköy Çarşı Şubesi".to_string();
        let register_id = "KASA-01".to_string();
        let cashier_id = "USR_01".to_string();
        let cashier_name = "Ahmet Kasiyer".to_string();

        let mut subtotal = 0.0;
        let mut vat_total = 0.0;

        for item in &req.items {
            subtotal += item.total;
            // KDV hesapla
            let item_vat = item.total - (item.total / (1.0 + (item.vat_rate / 100.0)));
            vat_total += item_vat;

            // Stok düşümü yap
            tx.execute(
                "UPDATE products SET stock_qty = stock_qty - ?1 WHERE id = ?2",
                params![item.quantity, item.product_id],
            )?;
        }

        let discount = req.discount_amount.unwrap_or(0.0);
        let grand_total = (subtotal - discount).max(0.0);

        let tender = req.tender_amount.unwrap_or(grand_total);
        let change = (tender - grand_total).max(0.0);

        let customer_name = if let Some(ref cid) = req.customer_id {
            let cname: Option<String> = tx
                .query_row(
                    "SELECT name FROM customers WHERE id = ?1",
                    params![cid],
                    |r| r.get(0),
                )
                .ok();
            cname
        } else {
            None
        };

        // Veresiye ise müşterinin borç bakiyesini artır ve deftere yaz
        if req.payment_method == "VERESIYE" {
            if let Some(ref cid) = req.customer_id {
                tx.execute(
                    "UPDATE customers SET balance = balance + ?1, updated_at = ?2 WHERE id = ?3",
                    params![grand_total, now.to_rfc3339(), cid],
                )?;

                let new_bal: f64 = tx.query_row(
                    "SELECT balance FROM customers WHERE id = ?1",
                    params![cid],
                    |r| r.get(0),
                )?;

                let ledger_id = Uuid::new_v4().to_string();
                tx.execute(
                    "INSERT INTO ledger_entries (id, tenant_id, customer_id, customer_name, entry_type, amount, balance_after, note, created_at, sync_status)
                     VALUES (?1, ?2, ?3, ?4, 'DEBT', ?5, ?6, ?7, ?8, 'PENDING')",
                    params![
                        ledger_id,
                        tenant_id,
                        cid,
                        customer_name.as_deref().unwrap_or("Müşteri"),
                        grand_total,
                        new_bal,
                        format!("Fiş: #{}", receipt_no),
                        now.to_rfc3339()
                    ],
                )?;
            }
        }

        // Satış başlığını kaydet
        tx.execute(
            "INSERT INTO sales (id, receipt_no, tenant_id, branch_id, branch_name, register_id, cashier_id, cashier_name, subtotal, vat_amount, discount_amount, grand_total, payment_method, customer_id, customer_name, tender_amount, change_amount, created_at, sync_status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, 'PENDING')",
            params![
                sale_id,
                receipt_no,
                tenant_id,
                branch_id,
                branch_name,
                register_id,
                cashier_id,
                cashier_name,
                subtotal,
                vat_total,
                discount,
                grand_total,
                req.payment_method,
                req.customer_id,
                customer_name,
                tender,
                change,
                now.to_rfc3339()
            ],
        )?;

        // Kalemleri kaydet
        for item in &req.items {
            tx.execute(
                "INSERT INTO sale_items (sale_id, product_id, barcode, name, unit_price, quantity, unit, vat_rate, discount, total, is_weighed, notes)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    sale_id,
                    item.product_id,
                    item.barcode,
                    item.name,
                    item.unit_price,
                    item.quantity,
                    item.unit,
                    item.vat_rate,
                    item.discount,
                    item.total,
                    if item.is_weighed { 1 } else { 0 },
                    item.notes
                ],
            )?;
        }

        let sale = Sale {
            id: sale_id.clone(),
            receipt_no: receipt_no.clone(),
            tenant_id: tenant_id.clone(),
            branch_id,
            branch_name,
            register_id,
            cashier_id,
            cashier_name,
            subtotal,
            vat_amount: vat_total,
            discount_amount: discount,
            grand_total,
            payment_method: req.payment_method,
            customer_id: req.customer_id,
            customer_name,
            tender_amount: tender,
            change_amount: change,
            items: req.items,
            created_at: now.to_rfc3339(),
            sync_status: "PENDING".to_string(),
        };

        // Outbox tablosuna yaz (İnternet geldiğinde buluta basılacak)
        let outbox_id = Uuid::new_v4().to_string();
        let payload = serde_json::to_string(&sale)?;
        tx.execute(
            "INSERT INTO outbox_sync (id, entity_type, payload_json, idempotency_key, created_at, status)
             VALUES (?1, 'SALE', ?2, ?3, ?4, 'PENDING')",
            params![outbox_id, payload, sale_id, now.to_rfc3339()],
        )?;

        tx.commit()?;

        Ok(sale)
    }

    /// Veresiye Tahsilatı Kaydet (Borç Ödemesi)
    pub fn record_customer_payment(
        &self,
        customer_id: &str,
        amount: f64,
        note: &str,
    ) -> Result<LedgerEntry> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;

        let now = Local::now();
        let tenant_id = "CUST_69653cfccb839".to_string();

        let cname: String = tx.query_row(
            "SELECT name FROM customers WHERE id = ?1",
            params![customer_id],
            |r| r.get(0),
        )?;

        // Borcu düş
        tx.execute(
            "UPDATE customers SET balance = balance - ?1, updated_at = ?2 WHERE id = ?3",
            params![amount, now.to_rfc3339(), customer_id],
        )?;

        let balance_after: f64 = tx.query_row(
            "SELECT balance FROM customers WHERE id = ?1",
            params![customer_id],
            |r| r.get(0),
        )?;

        let entry_id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO ledger_entries (id, tenant_id, customer_id, customer_name, entry_type, amount, balance_after, note, created_at, sync_status)
             VALUES (?1, ?2, ?3, ?4, 'PAYMENT', ?5, ?6, ?7, ?8, 'PENDING')",
            params![
                entry_id,
                tenant_id,
                customer_id,
                cname,
                amount,
                balance_after,
                note,
                now.to_rfc3339()
            ],
        )?;

        let entry = LedgerEntry {
            id: entry_id.clone(),
            tenant_id,
            customer_id: customer_id.to_string(),
            customer_name: cname,
            entry_type: "PAYMENT".to_string(),
            amount,
            balance_after,
            note: Some(note.to_string()),
            created_at: now.to_rfc3339(),
            sync_status: "PENDING".to_string(),
        };

        // Outbox'a ekle
        let payload = serde_json::to_string(&entry)?;
        tx.execute(
            "INSERT INTO outbox_sync (id, entity_type, payload_json, idempotency_key, created_at, status)
             VALUES (?1, 'LEDGER', ?2, ?3, ?4, 'PENDING')",
            params![Uuid::new_v4().to_string(), payload, entry_id, now.to_rfc3339()],
        )?;

        tx.commit()?;

        Ok(entry)
    }

    /// Bekleyen Senkronizasyon Sayısı
    pub fn get_pending_sync_count(&self) -> usize {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT COUNT(*) FROM outbox_sync WHERE status = 'PENDING'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0)
    }

    /// Outbox'tan bekleyen kayıtları çek
    pub fn get_pending_outbox_items(&self, limit: usize) -> Vec<(String, String, String)> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, entity_type, payload_json FROM outbox_sync WHERE status = 'PENDING' ORDER BY created_at ASC LIMIT ?1"
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map(params![limit], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)));
        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Outbox kaydını 'SYNCED' olarak işaretle
    pub fn mark_outbox_synced(&self, outbox_id: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE outbox_sync SET status = 'SYNCED' WHERE id = ?1",
            params![outbox_id],
        )?;
        Ok(())
    }

    /// Gün Sonu Z Raporu Özeti
    pub fn generate_z_report(&self) -> Result<ZReport> {
        let conn = self.conn.lock();
        let today = Local::now().format("%Y-%m-%d").to_string();

        let total_count: usize = conn.query_row(
            "SELECT COUNT(*) FROM sales WHERE created_at LIKE ?1",
            params![format!("{}%", today)],
            |r| r.get(0),
        )?;

        let total_gross: f64 = conn.query_row(
            "SELECT COALESCE(SUM(subtotal), 0.0) FROM sales WHERE created_at LIKE ?1",
            params![format!("{}%", today)],
            |r| r.get(0),
        )?;

        let total_disc: f64 = conn.query_row(
            "SELECT COALESCE(SUM(discount_amount), 0.0) FROM sales WHERE created_at LIKE ?1",
            params![format!("{}%", today)],
            |r| r.get(0),
        )?;

        let total_net: f64 = conn.query_row(
            "SELECT COALESCE(SUM(grand_total), 0.0) FROM sales WHERE created_at LIKE ?1",
            params![format!("{}%", today)],
            |r| r.get(0),
        )?;

        let cash_total: f64 = conn.query_row(
            "SELECT COALESCE(SUM(grand_total), 0.0) FROM sales WHERE payment_method = 'CASH' AND created_at LIKE ?1",
            params![format!("{}%", today)],
            |r| r.get(0),
        )?;

        let card_total: f64 = conn.query_row(
            "SELECT COALESCE(SUM(grand_total), 0.0) FROM sales WHERE payment_method = 'CREDIT_CARD' AND created_at LIKE ?1",
            params![format!("{}%", today)],
            |r| r.get(0),
        )?;

        let veresiye_total: f64 = conn.query_row(
            "SELECT COALESCE(SUM(grand_total), 0.0) FROM sales WHERE payment_method = 'VERESIYE' AND created_at LIKE ?1",
            params![format!("{}%", today)],
            |r| r.get(0),
        )?;

        // KDV dökümü
        let mut vat_stmt = conn.prepare(
            "SELECT vat_rate, SUM(total) FROM sale_items si JOIN sales s ON si.sale_id = s.id WHERE s.created_at LIKE ?1 GROUP BY vat_rate"
        )?;

        let vat_rows = vat_stmt.query_map(params![format!("{}%", today)], |row| {
            let rate: f64 = row.get(0)?;
            let total_with_vat: f64 = row.get(1)?;
            let vat_amount = total_with_vat - (total_with_vat / (1.0 + (rate / 100.0)));
            let taxable = total_with_vat - vat_amount;
            Ok(VatSummary {
                rate,
                taxable_amount: taxable,
                vat_amount,
            })
        })?;

        let vat_breakdown: Vec<VatSummary> = vat_rows.filter_map(|r| r.ok()).collect();

        Ok(ZReport {
            z_number: format!("Z-{}", Local::now().format("%Y%m%d%H%M")),
            branch_name: "Kadıköy Çarşı Şubesi".to_string(),
            register_id: "KASA-01".to_string(),
            cashier_name: "Ahmet Kasiyer".to_string(),
            date: Local::now().format("%d.%m.%Y %H:%M").to_string(),
            total_sales_count: total_count,
            total_gross,
            total_discount: total_disc,
            total_net,
            cash_total,
            card_total,
            veresiye_total,
            vat_breakdown,
        })
    }

    /// Kasa Durumu
    pub fn get_pos_status(&self, is_online: bool) -> PosStatus {
        let conn = self.conn.lock();
        let today = Local::now().format("%Y-%m-%d").to_string();

        let count: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM sales WHERE created_at LIKE ?1",
                params![format!("{}%", today)],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let revenue: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(grand_total), 0.0) FROM sales WHERE created_at LIKE ?1",
                params![format!("{}%", today)],
                |r| r.get(0),
            )
            .unwrap_or(0.0);

        let pending: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM outbox_sync WHERE status = 'PENDING'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let local_ip = local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        PosStatus {
            version: "3.0.0".to_string(),
            tenant_id: "CUST_69653cfccb839".to_string(),
            business_name: "QORDY Cadde Esnaf".to_string(),
            branch_id: "BR_01".to_string(),
            branch_name: "Kadıköy Çarşı Şubesi".to_string(),
            register_id: "KASA-01".to_string(),
            cashier_name: "Ahmet Kasiyer".to_string(),
            is_online,
            pending_sync_count: pending,
            today_sales_count: count,
            today_revenue: revenue,
            local_ip,
            port: 8090,
        }
    }

    /// Yeni Ürün Ekle
    pub fn create_product(&self, req: CreateProductRequest) -> Result<Product> {
        let conn = self.conn.lock();
        let now = Local::now().to_rfc3339();
        let id = format!("PRD-{}", &Uuid::new_v4().to_string()[0..8].to_uppercase());
        let tenant_id = "CUST_69653cfccb839";

        conn.execute(
            "INSERT INTO products (id, tenant_id, barcode, name, category, price, vat_rate, stock_qty, unit, is_weighed, tare_grams, quick_order, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0.0, ?11, ?12)",
            params![
                id,
                tenant_id,
                req.barcode,
                req.name,
                req.category,
                req.price,
                req.vat_rate,
                req.stock_qty,
                req.unit,
                if req.is_weighed { 1 } else { 0 },
                req.quick_order,
                now
            ],
        )?;

        Ok(Product {
            id,
            tenant_id: tenant_id.to_string(),
            barcode: req.barcode,
            name: req.name,
            category: req.category,
            price: req.price,
            vat_rate: req.vat_rate,
            stock_qty: req.stock_qty,
            unit: req.unit,
            is_weighed: req.is_weighed,
            tare_grams: 0.0,
            quick_order: req.quick_order,
            updated_at: now,
        })
    }

    /// Ürün Güncelle
    pub fn update_product(&self, id: &str, req: CreateProductRequest) -> Result<Product> {
        let conn = self.conn.lock();
        let now = Local::now().to_rfc3339();

        conn.execute(
            "UPDATE products SET barcode = ?1, name = ?2, category = ?3, price = ?4, vat_rate = ?5, stock_qty = ?6, unit = ?7, is_weighed = ?8, quick_order = ?9, updated_at = ?10
             WHERE id = ?11",
            params![
                req.barcode,
                req.name,
                req.category,
                req.price,
                req.vat_rate,
                req.stock_qty,
                req.unit,
                if req.is_weighed { 1 } else { 0 },
                req.quick_order,
                now,
                id
            ],
        )?;

        Ok(Product {
            id: id.to_string(),
            tenant_id: "CUST_69653cfccb839".to_string(),
            barcode: req.barcode,
            name: req.name,
            category: req.category,
            price: req.price,
            vat_rate: req.vat_rate,
            stock_qty: req.stock_qty,
            unit: req.unit,
            is_weighed: req.is_weighed,
            tare_grams: 0.0,
            quick_order: req.quick_order,
            updated_at: now,
        })
    }

    /// Ürün Sil
    pub fn delete_product(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM products WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Tüm Ürünler Listesi (Opsiyonel kategori filtreli)
    pub fn get_all_products(&self, category: Option<&str>) -> Vec<Product> {
        let conn = self.conn.lock();

        let (sql, filter_param) = if let Some(cat) = category {
            ("SELECT id, tenant_id, barcode, name, category, price, vat_rate, stock_qty, unit, is_weighed, tare_grams, quick_order, updated_at FROM products WHERE category = ?1 ORDER BY name ASC", Some(cat))
        } else {
            ("SELECT id, tenant_id, barcode, name, category, price, vat_rate, stock_qty, unit, is_weighed, tare_grams, quick_order, updated_at FROM products ORDER BY name ASC", None)
        };

        let mut stmt = match conn.prepare(sql) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let map_row = |row: &rusqlite::Row| {
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
        };

        let rows = if let Some(cat) = filter_param {
            stmt.query_map(params![cat], map_row)
        } else {
            stmt.query_map([], map_row)
        };

        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Mevcut Kategoriler
    pub fn get_categories(&self) -> Vec<String> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare("SELECT DISTINCT category FROM products ORDER BY category ASC") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map([], |r| r.get(0));
        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Yeni Müşteri (Cari) Ekle
    pub fn create_customer(&self, req: CreateCustomerRequest) -> Result<Customer> {
        let conn = self.conn.lock();
        let now = Local::now().to_rfc3339();
        let id = format!("CUST-{}", &Uuid::new_v4().to_string()[0..8].to_uppercase());
        let tenant_id = "CUST_69653cfccb839";

        conn.execute(
            "INSERT INTO customers (id, tenant_id, name, phone, balance, credit_limit, note, updated_at)
             VALUES (?1, ?2, ?3, ?4, 0.0, ?5, ?6, ?7)",
            params![id, tenant_id, req.name, req.phone, req.credit_limit, req.note, now],
        )?;

        Ok(Customer {
            id,
            tenant_id: tenant_id.to_string(),
            name: req.name,
            phone: req.phone,
            balance: 0.0,
            credit_limit: req.credit_limit,
            note: req.note,
            updated_at: now,
        })
    }

    /// Müşterinin Cari Hesap Ekstresi (Hareket Geçmişi)
    pub fn get_customer_ledger(&self, customer_id: &str) -> Vec<LedgerEntry> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, tenant_id, customer_id, customer_name, entry_type, amount, balance_after, note, created_at, sync_status
             FROM ledger_entries WHERE customer_id = ?1 ORDER BY created_at DESC LIMIT 50"
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map(params![customer_id], |r| {
            Ok(LedgerEntry {
                id: r.get(0)?,
                tenant_id: r.get(1)?,
                customer_id: r.get(2)?,
                customer_name: r.get(3)?,
                entry_type: r.get(4)?,
                amount: r.get(5)?,
                balance_after: r.get(6)?,
                note: r.get(7)?,
                created_at: r.get(8)?,
                sync_status: r.get(9)?,
            })
        });

        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Sepeti Beklet / Park Et (F7)
    pub fn park_cart(&self, note: &str, items: Vec<CartItem>) -> Result<ParkedCart> {
        let conn = self.conn.lock();
        let id = Uuid::new_v4().to_string();
        let now = Local::now().to_rfc3339();
        let grand_total: f64 = items.iter().map(|i| i.total).sum();
        let item_count = items.len();
        let items_json = serde_json::to_string(&items)?;

        conn.execute(
            "INSERT INTO parked_carts (id, note, items_json, grand_total, item_count, parked_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, note, items_json, grand_total, item_count, now],
        )?;

        Ok(ParkedCart {
            id,
            note: note.to_string(),
            items,
            grand_total,
            item_count,
            parked_at: now,
        })
    }

    /// Bekletilen Sepetleri Getir
    pub fn get_parked_carts(&self) -> Vec<ParkedCart> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, note, items_json, grand_total, item_count, parked_at FROM parked_carts ORDER BY parked_at DESC"
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map([], |r| {
            let json_str: String = r.get(2)?;
            let items: Vec<CartItem> = serde_json::from_str(&json_str).unwrap_or_default();
            Ok(ParkedCart {
                id: r.get(0)?,
                note: r.get(1)?,
                items,
                grand_total: r.get(3)?,
                item_count: r.get(4)?,
                parked_at: r.get(5)?,
            })
        });

        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Bekletilen Sepeti Sil (Kasaya geri çağrıldığında veya iptal edildiğinde)
    pub fn delete_parked_cart(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM parked_carts WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Yeni Stok Sayım Oturumu Başlat
    pub fn start_stock_count(&self, title: &str, count_type: &str, notes: Option<&str>) -> Result<StockCountSession> {
        let conn = self.conn.lock();
        let id = Uuid::new_v4().to_string();
        let now = Local::now().to_rfc3339();

        conn.execute(
            "INSERT INTO stock_count_sessions (id, title, count_type, status, started_at, notes)
             VALUES (?1, ?2, ?3, 'OPEN', ?4, ?5)",
            params![id, title, count_type, now, notes],
        )?;

        Ok(StockCountSession {
            id,
            title: title.to_string(),
            count_type: count_type.to_string(),
            status: "OPEN".to_string(),
            started_at: now,
            completed_at: None,
            total_items_counted: 0,
            notes: notes.map(|s| s.to_string()),
        })
    }

    /// Sayıma Barkodlu Ürün Ekle / Miktar Artır
    pub fn add_stock_count_item(&self, session_id: &str, barcode: &str, counted_qty: f64) -> Result<StockCountItem> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;

        // Önce ürünü bul
        let (product_id, name, unit, system_qty): (String, String, String, f64) = tx.query_row(
            "SELECT id, name, unit, stock_qty FROM products WHERE barcode = ?1 LIMIT 1",
            params![barcode],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )?;

        // Bu oturumda daha önce bu ürün sayılmış mı kontrol et
        let existing: Option<(String, f64)> = tx.query_row(
            "SELECT id, counted_qty FROM stock_count_items WHERE session_id = ?1 AND product_id = ?2",
            params![session_id, product_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ).ok();

        let now = Local::now().to_rfc3339();
        let (item_id, final_qty) = if let Some((old_id, old_qty)) = existing {
            let new_qty = old_qty + counted_qty;
            let variance = new_qty - system_qty;
            tx.execute(
                "UPDATE stock_count_items SET counted_qty = ?1, variance_qty = ?2, counted_at = ?3 WHERE id = ?4",
                params![new_qty, variance, now, old_id],
            )?;
            (old_id, new_qty)
        } else {
            let new_id = Uuid::new_v4().to_string();
            let variance = counted_qty - system_qty;
            tx.execute(
                "INSERT INTO stock_count_items (id, session_id, product_id, barcode, product_name, system_qty, counted_qty, variance_qty, unit, counted_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![new_id, session_id, product_id, barcode, name, system_qty, counted_qty, variance, unit, now],
            )?;
            (new_id, counted_qty)
        };

        tx.commit()?;

        Ok(StockCountItem {
            id: item_id,
            session_id: session_id.to_string(),
            product_id,
            barcode: barcode.to_string(),
            product_name: name,
            system_qty,
            counted_qty: final_qty,
            variance_qty: final_qty - system_qty,
            unit,
            counted_at: now,
        })
    }

    /// Sayım Kalemlerini Listele
    pub fn get_stock_count_items(&self, session_id: &str) -> Vec<StockCountItem> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, session_id, product_id, barcode, product_name, system_qty, counted_qty, variance_qty, unit, counted_at
             FROM stock_count_items WHERE session_id = ?1 ORDER BY counted_at DESC"
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map(params![session_id], |r| {
            Ok(StockCountItem {
                id: r.get(0)?,
                session_id: r.get(1)?,
                product_id: r.get(2)?,
                barcode: r.get(3)?,
                product_name: r.get(4)?,
                system_qty: r.get(5)?,
                counted_qty: r.get(6)?,
                variance_qty: r.get(7)?,
                unit: r.get(8)?,
                counted_at: r.get(9)?,
            })
        });

        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Sayımı Tamamla ve Stokları Güncelle (Commit Stock Count)
    pub fn commit_stock_count(&self, session_id: &str) -> Result<()> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;

        let now = Local::now().to_rfc3339();

        // 1. Sayılan kalemleri çek
        let mut items = Vec::new();
        {
            let mut stmt = tx.prepare(
                "SELECT product_id, counted_qty FROM stock_count_items WHERE session_id = ?1"
            )?;
            let rows = stmt.query_map(params![session_id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?)))?;
            for item in rows.flatten() {
                items.push(item);
            }
        }

        // 2. Her ürünün stok miktarını güncellenen sayım miktarıyla eşitle
        for (prod_id, counted_qty) in items {
            tx.execute(
                "UPDATE products SET stock_qty = ?1, updated_at = ?2 WHERE id = ?3",
                params![counted_qty, now, prod_id],
            )?;
        }

        // 3. Oturumu 'COMMITTED' yap
        tx.execute(
            "UPDATE stock_count_sessions SET status = 'COMMITTED', completed_at = ?1 WHERE id = ?2",
            params![now, session_id],
        )?;

        tx.commit()?;
        Ok(())
    }

    /// Stok Sayım Oturumlarını Listele
    pub fn get_stock_count_sessions(&self) -> Vec<StockCountSession> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT s.id, s.title, s.count_type, s.status, s.started_at, s.completed_at, COUNT(i.id), s.notes
             FROM stock_count_sessions s
             LEFT JOIN stock_count_items i ON s.id = i.session_id
             GROUP BY s.id ORDER BY s.started_at DESC"
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map([], |r| {
            Ok(StockCountSession {
                id: r.get(0)?,
                title: r.get(1)?,
                count_type: r.get(2)?,
                status: r.get(3)?,
                started_at: r.get(4)?,
                completed_at: r.get(5)?,
                total_items_counted: r.get(6)?,
                notes: r.get(7)?,
            })
        });

        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Müşteri Ekranı (CFD) Durumunu Güncelle
    pub fn update_cfd_state(&self, state: &CfdState) -> Result<()> {
        let conn = self.conn.lock();
        let payload = serde_json::to_string(state)?;
        conn.execute(
            "INSERT OR REPLACE INTO cfd_state (id, state, payload_json, updated_at) VALUES ('ACTIVE', ?1, ?2, ?3)",
            params![state.state, payload, state.updated_at],
        )?;
        Ok(())
    }

    /// Müşteri Ekranı (CFD) Durumunu Al
    pub fn get_cfd_state(&self) -> CfdState {
        let conn = self.conn.lock();
        let row_opt: Option<(String, String)> = conn.query_row(
            "SELECT payload_json, updated_at FROM cfd_state WHERE id = 'ACTIVE'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ).ok();

        if let Some((json, updated)) = row_opt {
            if let Ok(mut cfd) = serde_json::from_str::<CfdState>(&json) {
                cfd.updated_at = updated;
                return cfd;
            }
        }

        CfdState {
            state: "IDLE".to_string(),
            items: Vec::new(),
            subtotal: 0.0,
            discount: 0.0,
            grand_total: 0.0,
            tender: 0.0,
            change: 0.0,
            payment_method: None,
            customer_name: None,
            updated_at: Local::now().to_rfc3339(),
        }
    }

    /// Fire / Zayi Kaydı Ekle (Stoktan Düşülür)
    pub fn record_waste(&self, req: CreateWasteRequest) -> Result<WasteRecord> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;

        // Ürünü bul
        let (prod_id, prod_name, unit, price): (String, String, String, f64) = tx.query_row(
            "SELECT id, name, unit, price FROM products WHERE barcode = ?1 LIMIT 1",
            params![req.barcode],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )?;

        let cost_loss = req.quantity * price;
        let now = Local::now().to_rfc3339();
        let id = Uuid::new_v4().to_string();

        // Stoktan düş
        tx.execute(
            "UPDATE products SET stock_qty = stock_qty - ?1, updated_at = ?2 WHERE id = ?3",
            params![req.quantity, now, prod_id],
        )?;

        // Fire tablosuna ekle
        tx.execute(
            "INSERT INTO waste_records (id, product_id, product_name, barcode, quantity, unit, cost_loss, reason, recorded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![id, prod_id, prod_name, req.barcode, req.quantity, unit, cost_loss, req.reason, now],
        )?;

        tx.commit()?;

        Ok(WasteRecord {
            id,
            product_id: prod_id,
            product_name: prod_name,
            barcode: req.barcode,
            quantity: req.quantity,
            unit,
            cost_loss,
            reason: req.reason,
            recorded_at: now,
        })
    }

    /// Fire Kayıtları Listesi
    pub fn get_waste_records(&self) -> Vec<WasteRecord> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, product_id, product_name, barcode, quantity, unit, cost_loss, reason, recorded_at
             FROM waste_records ORDER BY recorded_at DESC LIMIT 100"
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map([], |r| {
            Ok(WasteRecord {
                id: r.get(0)?,
                product_id: r.get(1)?,
                product_name: r.get(2)?,
                barcode: r.get(3)?,
                quantity: r.get(4)?,
                unit: r.get(5)?,
                cost_loss: r.get(6)?,
                reason: r.get(7)?,
                recorded_at: r.get(8)?,
            })
        });

        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Kısmi veya Tam İade İşlemi
    pub fn process_refund(&self, req: RefundRequest) -> Result<RefundResponse> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;

        let now = Local::now();
        let refund_id = Uuid::new_v4().to_string();
        let refund_receipt_no = format!("IAD-{}-{:06}", now.format("%Y%m%d%H%M%S"), now.timestamp_subsec_micros());

        // İade edilen ürünlerin stoklarını geri yükle
        for item in &req.items {
            tx.execute(
                "UPDATE products SET stock_qty = stock_qty + ?1 WHERE id = ?2",
                params![item.quantity, item.product_id],
            )?;
        }

        let items_json = serde_json::to_string(&req.items)?;

        tx.execute(
            "INSERT INTO refunds (id, sale_id, refund_receipt_no, items_json, refund_amount, reason, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![refund_id, req.sale_id, refund_receipt_no, items_json, req.refund_amount, req.reason, now.to_rfc3339()],
        )?;

        tx.commit()?;

        Ok(RefundResponse {
            success: true,
            refund_id,
            refund_receipt_no,
            refunded_amount: req.refund_amount,
            message: "İade işlemi başarıyla tamamlandı ve stoklar iade edildi.".to_string(),
        })
    }

    /// Son İşlem Geçmişi (Fiş Listesi)
    pub fn get_sales_history(&self, limit: usize) -> Vec<Sale> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, receipt_no, tenant_id, branch_id, branch_name, register_id, cashier_id, cashier_name,
                    subtotal, vat_amount, discount_amount, grand_total, payment_method, customer_id, customer_name,
                    tender_amount, change_amount, created_at, sync_status
             FROM sales ORDER BY created_at DESC LIMIT ?1"
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map(params![limit], |r| {
            let sale_id: String = r.get(0)?;
            Ok(Sale {
                id: sale_id.clone(),
                receipt_no: r.get(1)?,
                tenant_id: r.get(2)?,
                branch_id: r.get(3)?,
                branch_name: r.get(4)?,
                register_id: r.get(5)?,
                cashier_id: r.get(6)?,
                cashier_name: r.get(7)?,
                subtotal: r.get(8)?,
                vat_amount: r.get(9)?,
                discount_amount: r.get(10)?,
                grand_total: r.get(11)?,
                payment_method: r.get(12)?,
                customer_id: r.get(13)?,
                customer_name: r.get(14)?,
                tender_amount: r.get(15)?,
                change_amount: r.get(16)?,
                items: Vec::new(),
                created_at: r.get(17)?,
                sync_status: r.get(18)?,
            })
        });

        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Kasa Para Giriş / Çıkış Hareketi (Avans / Masraf)
    pub fn record_cash_movement(&self, movement_type: &str, amount: f64, reason: &str) -> Result<CashMovement> {
        let conn = self.conn.lock();
        let id = Uuid::new_v4().to_string();
        let now = Local::now().to_rfc3339();
        let register_id = "KASA-01";
        let cashier_name = "Ahmet Kasiyer";

        conn.execute(
            "INSERT INTO cash_movements (id, movement_type, amount, reason, register_id, cashier_name, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, movement_type, amount, reason, register_id, cashier_name, now],
        )?;

        Ok(CashMovement {
            id,
            movement_type: movement_type.to_string(),
            amount,
            reason: reason.to_string(),
            register_id: register_id.to_string(),
            cashier_name: cashier_name.to_string(),
            created_at: now,
        })
    }

    /// Kasa Hareketleri Listesi
    pub fn get_cash_movements(&self) -> Vec<CashMovement> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, movement_type, amount, reason, register_id, cashier_name, created_at
             FROM cash_movements ORDER BY created_at DESC LIMIT 50"
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map([], |r| {
            Ok(CashMovement {
                id: r.get(0)?,
                movement_type: r.get(1)?,
                amount: r.get(2)?,
                reason: r.get(3)?,
                register_id: r.get(4)?,
                cashier_name: r.get(5)?,
                created_at: r.get(6)?,
            })
        });

        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Raf & Barkod Etiketleri Verisi
    pub fn get_shelf_labels(&self, category: Option<&str>) -> Vec<ShelfLabel> {
        let conn = self.conn.lock();
        let (sql, filter_param) = if let Some(cat) = category {
            ("SELECT id, barcode, name, price, unit, vat_rate, category FROM products WHERE category = ?1 ORDER BY name ASC", Some(cat))
        } else {
            ("SELECT id, barcode, name, price, unit, vat_rate, category FROM products ORDER BY name ASC", None)
        };

        let mut stmt = match conn.prepare(sql) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let map_row = |r: &rusqlite::Row| {
            Ok(ShelfLabel {
                product_id: r.get(0)?,
                barcode: r.get(1)?,
                name: r.get(2)?,
                price: r.get(3)?,
                unit: r.get(4)?,
                vat_rate: r.get(5)?,
                category: r.get(6)?,
            })
        };

        let rows = if let Some(cat) = filter_param {
            stmt.query_map(params![cat], map_row)
        } else {
            stmt.query_map([], map_row)
        };

        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }
}
