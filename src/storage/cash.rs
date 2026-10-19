//! QORDY POS - Cash Movements (Avans / Masraf) Repository

use crate::models::CashMovement;
use crate::storage::DatabaseManager;
use anyhow::Result;
use chrono::Local;
use rusqlite::params;
use uuid::Uuid;

impl DatabaseManager {
    /// Kasa Para Giriş / Çıkış Hareketi (Avans / Masraf)
    pub fn record_cash_movement(
        &self,
        movement_type: &str,
        amount: f64,
        reason: &str,
    ) -> Result<CashMovement> {
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
             FROM cash_movements ORDER BY created_at DESC LIMIT 50",
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
}
