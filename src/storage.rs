use crate::models::{
    CheckoutRequest, Customer, LedgerEntry, PosStatus, Product, Sale, VatSummary, ZReport,
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
}
