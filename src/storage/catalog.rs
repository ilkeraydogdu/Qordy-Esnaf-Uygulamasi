//! QORDY POS - 4.13 Milyon Ulusal Barkod Kataloğu Yerel Arama & Önbellek Deposu

use crate::models::NationalProductInfo;
use crate::storage::DatabaseManager;
use anyhow::Result;
use rusqlite::params;

impl DatabaseManager {
    /// Ulusal Barkod Önbelleğinden Barkod ile Sorgula
    pub fn lookup_national_catalog(&self, barcode: &str) -> Option<NationalProductInfo> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT barcode, name, brand, category, vat_rate, unit, is_weighed, plu_code, suggested_price
             FROM national_catalog_cache WHERE barcode = ?1",
            params![barcode],
            |row| {
                let is_w: i32 = row.get(6)?;

                Ok(NationalProductInfo {
                    barcode: row.get(0)?,
                    name: row.get(1)?,
                    brand: row.get(2)?,
                    category: row.get(3)?,
                    vat_rate: row.get(4)?,
                    unit: row.get(5)?,
                    is_weighed: is_w == 1,
                    plu_code: row.get(7)?,
                    suggested_price: row.get(8)?,
                    source: "NATIONAL_CATALOG_4.13M".to_string(),
                })
            },
        )
        .ok()
    }

    /// Ulusal Katalog Verisini Önbelleğe Kaydet
    pub fn save_national_catalog_item(&self, item: &NationalProductInfo) -> Result<()> {
        let conn = self.conn.lock();

        conn.execute(
            "INSERT OR REPLACE INTO national_catalog_cache 
             (barcode, name, brand, category, vat_rate, unit, is_weighed, plu_code, suggested_price)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                item.barcode,
                item.name,
                item.brand,
                item.category,
                item.vat_rate,
                item.unit,
                if item.is_weighed { 1 } else { 0 },
                item.plu_code,
                item.suggested_price
            ],
        )?;

        Ok(())
    }

    /// Ulusal Katalogda İsim / Marka ile Arama Yap
    pub fn search_national_catalog(&self, query: &str) -> Vec<NationalProductInfo> {
        let conn = self.conn.lock();
        let pattern = format!("%{}%", query.trim());
        let mut stmt = match conn.prepare(
            "SELECT barcode, name, brand, category, vat_rate, unit, is_weighed, plu_code, suggested_price
             FROM national_catalog_cache 
             WHERE name LIKE ?1 OR barcode LIKE ?1 OR brand LIKE ?1
             ORDER BY name ASC LIMIT 20",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let iter = stmt.query_map(params![pattern], |row| {
            let is_w: i32 = row.get(6)?;

            Ok(NationalProductInfo {
                barcode: row.get(0)?,
                name: row.get(1)?,
                brand: row.get(2)?,
                category: row.get(3)?,
                vat_rate: row.get(4)?,
                unit: row.get(5)?,
                is_weighed: is_w == 1,
                plu_code: row.get(7)?,
                suggested_price: row.get(8)?,
                source: "NATIONAL_CATALOG_4.13M".to_string(),
            })
        });

        match iter {
            Ok(rows) => rows.filter_map(Result::ok).collect(),
            Err(_) => Vec::new(),
        }
    }
}
