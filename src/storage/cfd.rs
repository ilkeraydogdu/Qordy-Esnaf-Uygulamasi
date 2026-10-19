//! QORDY POS - Customer Facing Display (CFD) State Repository

use crate::models::CfdState;
use crate::storage::DatabaseManager;
use anyhow::Result;
use chrono::Local;
use rusqlite::params;

impl DatabaseManager {
    /// Müşteri Ekranı (CFD) Durumunu Güncelle
    pub fn update_cfd_state(&self, state: &CfdState) -> Result<()> {
        let conn = self.conn.lock();
        let payload = serde_json::to_string(state)?;
        let state_name = &state.state;
        let updated_at = &state.updated_at;
        conn.execute(
            "INSERT OR REPLACE INTO cfd_state (id, state, payload_json, updated_at) VALUES ('ACTIVE', ?1, ?2, ?3)",
            params![state_name, payload, updated_at],
        )?;
        Ok(())
    }

    /// Müşteri Ekranı (CFD) Durumunu Al
    pub fn get_cfd_state(&self) -> CfdState {
        let conn = self.conn.lock();
        let row_opt: Option<(String, String)> = conn
            .query_row(
                "SELECT payload_json, updated_at FROM cfd_state WHERE id = 'ACTIVE'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();

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
}