//! QORDY POS - Mal Kabul, Tedarikçi & Alış İrsaliyesi Deposu

use crate::models::{
    CreateGoodsReceiptRequest, CreateSupplierRequest, GoodsReceipt, GoodsReceiptItem, Supplier,
};
use crate::storage::DatabaseManager;
use anyhow::Result;
use chrono::Local;
use rusqlite::params;
use uuid::Uuid;

impl DatabaseManager {
    /// Tüm Tedarikçileri Listele
    pub fn get_suppliers(&self) -> Vec<Supplier> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, tenant_id, name, contact_name, phone, email, address, created_at
             FROM suppliers ORDER BY name ASC",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let iter = stmt.query_map([], |row| {
            Ok(Supplier {
                id: row.get(0)?,
                tenant_id: row.get(1)?,
                name: row.get(2)?,
                contact_name: row.get(3)?,
                phone: row.get(4)?,
                email: row.get(5)?,
                address: row.get(6)?,
                created_at: row.get(7)?,
            })
        });

        match iter {
            Ok(rows) => rows.filter_map(Result::ok).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Yeni Tedarikçi Oluştur
    pub fn create_supplier(&self, req: CreateSupplierRequest) -> Result<Supplier> {
        let conn = self.conn.lock();
        let id = format!("SUP-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        let tenant_id = self.config.tenant_id.clone();
        let now = Local::now().to_rfc3339();

        conn.execute(
            "INSERT INTO suppliers (id, tenant_id, name, contact_name, phone, email, address, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id,
                tenant_id,
                req.name,
                req.contact_name,
                req.phone,
                req.email,
                req.address,
                now
            ],
        )?;

        Ok(Supplier {
            id,
            tenant_id,
            name: req.name,
            contact_name: req.contact_name,
            phone: req.phone,
            email: req.email,
            address: req.address,
            created_at: now,
        })
    }

    /// Mal Kabul / Alış İrsaliyesi Kaydetme (Atomik İşlem & Stok/Fiyat Güncelleme)
    pub fn process_goods_receipt(&self, req: CreateGoodsReceiptRequest) -> Result<GoodsReceipt> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;

        let receipt_id = format!("GR-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        let tenant_id = self.config.tenant_id.clone();
        let branch_id = self.config.branch_id.clone();
        let now = Local::now().to_rfc3339();

        let mut total_amount = 0.0;
        for item in &req.items {
            total_amount += item.total_cost;
        }

        tx.execute(
            "INSERT INTO goods_receipts (id, tenant_id, branch_id, supplier_id, supplier_name, invoice_no, receipt_date, total_amount, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                receipt_id,
                tenant_id,
                branch_id,
                req.supplier_id,
                req.supplier_name,
                req.invoice_no,
                req.receipt_date,
                total_amount,
                req.notes,
                now
            ],
        )?;

        for item in &req.items {
            // 1. Kalem kaydı
            tx.execute(
                "INSERT INTO goods_receipt_items (receipt_id, product_id, barcode, name, quantity, unit, cost_price, sale_price, total_cost)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    receipt_id,
                    item.product_id,
                    item.barcode,
                    item.name,
                    item.quantity,
                    item.unit,
                    item.cost_price,
                    item.sale_price,
                    item.total_cost
                ],
            )?;

            // 2. Ürün stoğunu artır ve gerekiyorsa satış fiyatını güncelle
            let updated = tx.execute(
                "UPDATE products 
                 SET stock_qty = stock_qty + ?1,
                     price = CASE WHEN ?2 > 0 THEN ?2 ELSE price END,
                     updated_at = ?3
                 WHERE barcode = ?4 OR id = ?5",
                params![item.quantity, item.sale_price, now, item.barcode, item.product_id],
            )?;

            // Ürün yoksa yeni ürün olarak ekle
            if updated == 0 {
                let new_prod_id = format!("PRD-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
                tx.execute(
                    "INSERT INTO products (id, tenant_id, barcode, name, category, price, vat_rate, stock_qty, unit, is_weighed, tare_grams, updated_at)
                     VALUES (?1, ?2, ?3, ?4, 'Genel', ?5, 10.0, ?6, ?7, 0, 0.0, ?8)",
                    params![new_prod_id, tenant_id, item.barcode, item.name, item.sale_price, item.quantity, item.unit, now],
                )?;
            }
        }

        // 3. Outbox Senkronizasyon kaydı
        let outbox_id = Uuid::new_v4().to_string();
        let payload = serde_json::json!({
            "receipt_id": receipt_id,
            "invoice_no": req.invoice_no,
            "supplier_id": req.supplier_id,
            "total_amount": total_amount,
            "items_count": req.items.len(),
            "created_at": now
        });

        tx.execute(
            "INSERT INTO outbox_sync (id, entity_type, payload_json, idempotency_key, created_at, status)
             VALUES (?1, 'GOODS_RECEIPT', ?2, ?3, ?4, 'PENDING')",
            params![outbox_id, payload.to_string(), receipt_id, now],
        )?;

        tx.commit()?;

        Ok(GoodsReceipt {
            id: receipt_id,
            tenant_id,
            branch_id,
            supplier_id: req.supplier_id,
            supplier_name: req.supplier_name,
            invoice_no: req.invoice_no,
            receipt_date: req.receipt_date,
            total_amount,
            notes: req.notes,
            items: req.items,
            created_at: now,
        })
    }

    /// Mal Kabul Geçmişi
    pub fn get_goods_receipts(&self) -> Vec<GoodsReceipt> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, tenant_id, branch_id, supplier_id, supplier_name, invoice_no, receipt_date, total_amount, notes, created_at
             FROM goods_receipts ORDER BY created_at DESC LIMIT 50",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let iter = stmt.query_map([], |row| {
            Ok(GoodsReceipt {
                id: row.get(0)?,
                tenant_id: row.get(1)?,
                branch_id: row.get(2)?,
                supplier_id: row.get(3)?,
                supplier_name: row.get(4)?,
                invoice_no: row.get(5)?,
                receipt_date: row.get(6)?,
                total_amount: row.get(7)?,
                notes: row.get(8)?,
                items: Vec::new(),
                created_at: row.get(9)?,
            })
        });

        match iter {
            Ok(rows) => rows.filter_map(Result::ok).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Mal Kabul Detay ve Kalemleri
    pub fn get_goods_receipt_detail(&self, id: &str) -> Option<GoodsReceipt> {
        let conn = self.conn.lock();
        let mut receipt = conn
            .query_row(
                "SELECT id, tenant_id, branch_id, supplier_id, supplier_name, invoice_no, receipt_date, total_amount, notes, created_at
                 FROM goods_receipts WHERE id = ?1",
                params![id],
                |row| {
                    Ok(GoodsReceipt {
                        id: row.get(0)?,
                        tenant_id: row.get(1)?,
                        branch_id: row.get(2)?,
                        supplier_id: row.get(3)?,
                        supplier_name: row.get(4)?,
                        invoice_no: row.get(5)?,
                        receipt_date: row.get(6)?,
                        total_amount: row.get(7)?,
                        notes: row.get(8)?,
                        items: Vec::new(),
                        created_at: row.get(9)?,
                    })
                },
            )
            .ok()?;

        let mut item_stmt = conn
            .prepare(
                "SELECT product_id, barcode, name, quantity, unit, cost_price, sale_price, total_cost
                 FROM goods_receipt_items WHERE receipt_id = ?1",
            )
            .ok()?;

        let items = item_stmt
            .query_map(params![id], |row| {
                Ok(GoodsReceiptItem {
                    product_id: row.get(0)?,
                    barcode: row.get(1)?,
                    name: row.get(2)?,
                    quantity: row.get(3)?,
                    unit: row.get(4)?,
                    cost_price: row.get(5)?,
                    sale_price: row.get(6)?,
                    total_cost: row.get(7)?,
                })
            })
            .ok()?
            .filter_map(Result::ok)
            .collect();

        receipt.items = items;
        Some(receipt)
    }
}
