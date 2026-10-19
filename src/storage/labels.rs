//! QORDY POS - Shelf Labels & Barcode Printing Repository

use crate::models::ShelfLabel;
use crate::storage::DatabaseManager;
use rusqlite::params;

impl DatabaseManager {
    /// Raf & Barkod Etiketleri Verisi
    pub fn get_shelf_labels(&self, category: Option<&str>) -> Vec<ShelfLabel> {
        let conn = self.conn.lock();
        let (sql, filter_param) = if let Some(cat) = category {
            (
                "SELECT id, barcode, name, price, unit, vat_rate, category FROM products WHERE category = ?1 ORDER BY name ASC",
                Some(cat),
            )
        } else {
            (
                "SELECT id, barcode, name, price, unit, vat_rate, category FROM products ORDER BY name ASC",
                None,
            )
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
