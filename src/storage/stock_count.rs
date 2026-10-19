//! QORDY POS - Stock Count Audits & Inventory Sessions Repository

use crate::models::{StockCountItem, StockCountSession};
use crate::storage::DatabaseManager;
use anyhow::Result;
use chrono::Local;
use rusqlite::params;
use uuid::Uuid;

impl DatabaseManager {
    /// Yeni Stok Sayım Oturumu Başlat
    pub fn start_stock_count(
        &self,
        title: &str,
        count_type: &str,
        notes: Option<&str>,
    ) -> Result<StockCountSession> {
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
    pub fn add_stock_count_item(
        &self,
        session_id: &str,
        barcode: &str,
        counted_qty: f64,
    ) -> Result<StockCountItem> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;

        // Önce ürünü bul
        let (product_id, name, unit, system_qty): (String, String, String, f64) = tx.query_row(
            "SELECT id, name, unit, stock_qty FROM products WHERE barcode = ?1 LIMIT 1",
            params![barcode],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )?;

        // Bu oturumda daha önce bu ürün sayılmış mı kontrol et
        let existing: Option<(String, f64)> = tx
            .query_row(
                "SELECT id, counted_qty FROM stock_count_items WHERE session_id = ?1 AND product_id = ?2",
                params![session_id, product_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();

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
             FROM stock_count_items WHERE session_id = ?1 ORDER BY counted_at DESC",
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
                "SELECT product_id, counted_qty FROM stock_count_items WHERE session_id = ?1",
            )?;
            let rows = stmt.query_map(params![session_id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))
            })?;
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
             GROUP BY s.id ORDER BY s.started_at DESC",
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
}
