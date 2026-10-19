//! QORDY POS - Waste / Fire Tracking Repository

use crate::models::{CreateWasteRequest, WasteRecord};
use crate::storage::DatabaseManager;
use anyhow::Result;
use chrono::Local;
use rusqlite::params;
use uuid::Uuid;

impl DatabaseManager {
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
            params![
                id,
                prod_id,
                prod_name,
                req.barcode,
                req.quantity,
                unit,
                cost_loss,
                req.reason,
                now
            ],
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
             FROM waste_records ORDER BY recorded_at DESC LIMIT 100",
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
}
