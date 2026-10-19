//! QORDY POS - Products & Inventory Catalog Repository

use crate::models::{CreateProductRequest, Product};
use crate::storage::DatabaseManager;
use anyhow::Result;
use chrono::Local;
use rusqlite::params;
use uuid::Uuid;

impl DatabaseManager {
    /// Barkod ile Ürün Bul (Önce tam barkod, yoksa terazi lookup_code)
    pub fn find_product_by_barcode(&self, barcode_or_lookup: &str) -> Option<Product> {
        let conn = self.conn.lock();

        let sql = "SELECT id, tenant_id, barcode, name, category, price, vat_rate, stock_qty, unit, is_weighed, tare_grams, quick_order, updated_at
                   FROM products WHERE barcode = ?1 OR barcode = ?2 LIMIT 1";

        let mut stmt = conn.prepare(sql).ok()?;
        let clean = barcode_or_lookup.trim();

        let product = stmt
            .query_row(params![clean, clean.trim_start_matches('0')], |row| {
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
            })
            .ok();

        if product.is_some() {
            return product;
        }

        // İkincil barkodlar tablosundan ara
        let alt_sql = "SELECT p.id, p.tenant_id, p.barcode, p.name, p.category, (p.price * b.multiplier), p.vat_rate, p.stock_qty, p.unit, p.is_weighed, p.tare_grams, p.quick_order, p.updated_at
                       FROM barcodes b JOIN products p ON b.product_id = p.id WHERE b.barcode = ?1 LIMIT 1";
        let mut alt_stmt = conn.prepare(alt_sql).ok()?;
        alt_stmt
            .query_row(params![clean], |row| {
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
            })
            .ok()
    }

    /// ID ile Ürün Bul
    #[allow(dead_code)]
    pub fn find_product_by_id(&self, id: &str) -> Option<Product> {
        let conn = self.conn.lock();
        let sql = "SELECT id, tenant_id, barcode, name, category, price, vat_rate, stock_qty, unit, is_weighed, tare_grams, quick_order, updated_at
                   FROM products WHERE id = ?1 LIMIT 1";
        let mut stmt = conn.prepare(sql).ok()?;
        stmt.query_row(params![id], |row| {
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
        })
        .ok()
    }

    /// Sık Satılan Hızlı Buton Ürünleri
    pub fn get_quick_products(&self) -> Vec<Product> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, tenant_id, barcode, name, category, price, vat_rate, stock_qty, unit, is_weighed, tare_grams, quick_order, updated_at
             FROM products WHERE quick_order IS NOT NULL ORDER BY quick_order ASC",
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
             FROM products WHERE name LIKE ?1 OR barcode LIKE ?1 ORDER BY name ASC LIMIT 30",
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

    /// Tüm Ürünler Listesi (Opsiyonel kategori filtreli)
    pub fn get_all_products(&self, category: Option<&str>) -> Vec<Product> {
        let conn = self.conn.lock();

        let (sql, filter_param) = if let Some(cat) = category {
            (
                "SELECT id, tenant_id, barcode, name, category, price, vat_rate, stock_qty, unit, is_weighed, tare_grams, quick_order, updated_at
                 FROM products WHERE category = ?1 ORDER BY name ASC",
                Some(cat),
            )
        } else {
            (
                "SELECT id, tenant_id, barcode, name, category, price, vat_rate, stock_qty, unit, is_weighed, tare_grams, quick_order, updated_at
                 FROM products ORDER BY name ASC",
                None,
            )
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
}
