//! QORDY POS - Customer & Veresiye Ledger Repository

use crate::models::{CreateCustomerRequest, Customer, LedgerEntry};
use crate::storage::DatabaseManager;
use anyhow::Result;
use chrono::Local;
use rusqlite::params;
use uuid::Uuid;

impl DatabaseManager {
    /// Veresiye Müşteri Listesi
    pub fn get_customers(&self) -> Vec<Customer> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, tenant_id, name, phone, balance, credit_limit, note, updated_at
             FROM customers ORDER BY name ASC",
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

    /// Yeni Müşteri (Cari) Ekle
    pub fn create_customer(&self, req: CreateCustomerRequest) -> Result<Customer> {
        let conn = self.conn.lock();
        let now = Local::now().to_rfc3339();
        let id = format!("CUST-{}", &Uuid::new_v4().to_string()[0..8].to_uppercase());
        let tenant_id = "CUST_69653cfccb839";

        conn.execute(
            "INSERT INTO customers (id, tenant_id, name, phone, balance, credit_limit, note, updated_at)
             VALUES (?1, ?2, ?3, ?4, 0.0, ?5, ?6, ?7)",
            params![id, tenant_id, req.name, req.phone, req.credit_limit, req.note, now],
        )?;

        Ok(Customer {
            id,
            tenant_id: tenant_id.to_string(),
            name: req.name,
            phone: req.phone,
            balance: 0.0,
            credit_limit: req.credit_limit,
            note: req.note,
            updated_at: now,
        })
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
        let tenant_id = "CUST_69653cfccb839";

        // Müşteri bilgisini çek
        let (name, current_balance): (String, f64) = tx.query_row(
            "SELECT name, balance FROM customers WHERE id = ?1",
            params![customer_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;

        let new_balance = current_balance - amount;

        // Bakiyeyi güncelle
        tx.execute(
            "UPDATE customers SET balance = ?1, updated_at = ?2 WHERE id = ?3",
            params![new_balance, now.to_rfc3339(), customer_id],
        )?;

        let ledger_id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO ledger_entries (id, tenant_id, customer_id, customer_name, entry_type, amount, balance_after, note, created_at, sync_status)
             VALUES (?1, ?2, ?3, ?4, 'PAYMENT', ?5, ?6, ?7, ?8, 'PENDING')",
            params![ledger_id, tenant_id, customer_id, name, amount, new_balance, note, now.to_rfc3339()],
        )?;

        let entry = LedgerEntry {
            id: ledger_id.clone(),
            tenant_id: tenant_id.to_string(),
            customer_id: customer_id.to_string(),
            customer_name: name,
            entry_type: "PAYMENT".to_string(),
            amount,
            balance_after: new_balance,
            note: Some(note.to_string()),
            created_at: now.to_rfc3339(),
            sync_status: "PENDING".to_string(),
        };

        // Outbox'a kuyrukla
        let outbox_id = Uuid::new_v4().to_string();
        let payload = serde_json::to_string(&entry)?;
        tx.execute(
            "INSERT INTO outbox_sync (id, entity_type, payload_json, idempotency_key, created_at, status)
             VALUES (?1, 'LEDGER', ?2, ?3, ?4, 'PENDING')",
            params![outbox_id, payload, ledger_id, now.to_rfc3339()],
        )?;

        tx.commit()?;

        Ok(entry)
    }

    /// Müşterinin Cari Hesap Ekstresi (Hareket Geçmişi)
    pub fn get_customer_ledger(&self, customer_id: &str) -> Vec<LedgerEntry> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, tenant_id, customer_id, customer_name, entry_type, amount, balance_after, note, created_at, sync_status
             FROM ledger_entries WHERE customer_id = ?1 ORDER BY created_at DESC LIMIT 50",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map(params![customer_id], |r| {
            Ok(LedgerEntry {
                id: r.get(0)?,
                tenant_id: r.get(1)?,
                customer_id: r.get(2)?,
                customer_name: r.get(3)?,
                entry_type: r.get(4)?,
                amount: r.get(5)?,
                balance_after: r.get(6)?,
                note: r.get(7)?,
                created_at: r.get(8)?,
                sync_status: r.get(9)?,
            })
        });

        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }
}
