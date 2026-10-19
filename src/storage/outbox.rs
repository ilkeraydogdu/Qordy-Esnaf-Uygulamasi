//! QORDY POS - Cloud Sync Outbox Pattern Repository

use crate::storage::DatabaseManager;
use anyhow::Result;
use rusqlite::params;

impl DatabaseManager {
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
            "SELECT id, entity_type, payload_json FROM outbox_sync WHERE status = 'PENDING' ORDER BY created_at ASC LIMIT ?1",
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
}
