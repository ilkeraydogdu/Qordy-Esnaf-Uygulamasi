//! QORDY POS - Şubeler & Şubeler Arası Mal Transferi Deposu

use crate::models::{
    Branch, CreateStockTransferRequest, StockTransfer, StockTransferItem,
};
use crate::storage::DatabaseManager;
use anyhow::{anyhow, Result};
use chrono::Local;
use rusqlite::params;
use uuid::Uuid;

impl DatabaseManager {
    /// Tüm Şubeleri Listele
    pub fn get_branches(&self) -> Vec<Branch> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, tenant_id, name, code, address, phone, is_active
             FROM branches ORDER BY name ASC",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let iter = stmt.query_map([], |row| {
            let is_act: i32 = row.get(6)?;
            Ok(Branch {
                id: row.get(0)?,
                tenant_id: row.get(1)?,
                name: row.get(2)?,
                code: row.get(3)?,
                address: row.get(4)?,
                phone: row.get(5)?,
                is_active: is_act == 1,
            })
        });

        let mut list: Vec<Branch> = match iter {
            Ok(rows) => rows.filter_map(Result::ok).collect(),
            Err(_) => Vec::new(),
        };

        if list.is_empty() {
            list.push(Branch {
                id: self.config.branch_id.clone(),
                tenant_id: self.config.tenant_id.clone(),
                name: self.config.branch_name.clone(),
                code: Some("MRK".to_string()),
                address: Some("Ritim Sokağı, Mimar Sinan Mahallesi, Sultanbeyli".to_string()),
                phone: Some("905510162050".to_string()),
                is_active: true,
            });
            list.push(Branch {
                id: "BRANCH-02".to_string(),
                tenant_id: self.config.tenant_id.clone(),
                name: "Sultanbeyli Çarşı Şubesi".to_string(),
                code: Some("SUL-2".to_string()),
                address: Some("Mimar Sinan Cad. No:18 Sultanbeyli".to_string()),
                phone: Some("905510162050".to_string()),
                is_active: true,
            });
            list.push(Branch {
                id: "BRANCH-WAREHOUSE".to_string(),
                tenant_id: self.config.tenant_id.clone(),
                name: "Merkez Ana Depo".to_string(),
                code: Some("DEPO".to_string()),
                address: Some("Organize Sanayi Bölgesi 4. Cad. No:12".to_string()),
                phone: Some("0216 555 00 11".to_string()),
                is_active: true,
            });
        }

        list
    }

    /// Yeni Şubeler Arası Transfer Talebi Oluştur
    pub fn create_stock_transfer(&self, req: CreateStockTransferRequest) -> Result<StockTransfer> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;

        let transfer_id = format!("TRF-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        let tenant_id = self.config.tenant_id.clone();
        let cashier = self.config.cashier_name.clone();
        let now = Local::now().to_rfc3339();

        tx.execute(
            "INSERT INTO stock_transfers (id, tenant_id, from_branch_id, from_branch_name, to_branch_id, to_branch_name, status, notes, created_by, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'DISPATCHED', ?7, ?8, ?9)",
            params![
                transfer_id,
                tenant_id,
                req.from_branch_id,
                req.from_branch_name,
                req.to_branch_id,
                req.to_branch_name,
                req.notes,
                cashier,
                now
            ],
        )?;

        for item in &req.items {
            tx.execute(
                "INSERT INTO stock_transfer_items (transfer_id, product_id, barcode, name, quantity, unit, received_quantity)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0.0)",
                params![
                    transfer_id,
                    item.product_id,
                    item.barcode,
                    item.name,
                    item.quantity,
                    item.unit
                ],
            )?;

            // Eğer çıkış yapılan şube mevcut şube ise stok düş
            if req.from_branch_id == self.config.branch_id {
                tx.execute(
                    "UPDATE products SET stock_qty = stock_qty - ?1, updated_at = ?2 WHERE barcode = ?3 OR id = ?4",
                    params![item.quantity, now, item.barcode, item.product_id],
                )?;
            }
        }

        tx.commit()?;

        Ok(StockTransfer {
            id: transfer_id,
            tenant_id,
            from_branch_id: req.from_branch_id,
            from_branch_name: req.from_branch_name,
            to_branch_id: req.to_branch_id,
            to_branch_name: req.to_branch_name,
            status: "DISPATCHED".to_string(),
            notes: req.notes,
            created_by: cashier,
            items: req.items,
            created_at: now,
            received_at: None,
        })
    }

    /// Transfer Geçmişi
    pub fn get_stock_transfers(&self) -> Vec<StockTransfer> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, tenant_id, from_branch_id, from_branch_name, to_branch_id, to_branch_name, status, notes, created_by, created_at, received_at
             FROM stock_transfers ORDER BY created_at DESC LIMIT 50",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let iter = stmt.query_map([], |row| {
            Ok(StockTransfer {
                id: row.get(0)?,
                tenant_id: row.get(1)?,
                from_branch_id: row.get(2)?,
                from_branch_name: row.get(3)?,
                to_branch_id: row.get(4)?,
                to_branch_name: row.get(5)?,
                status: row.get(6)?,
                notes: row.get(7)?,
                created_by: row.get(8)?,
                items: Vec::new(),
                created_at: row.get(9)?,
                received_at: row.get(10)?,
            })
        });

        match iter {
            Ok(rows) => rows.filter_map(Result::ok).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Transfer Detayı ve Kalemleri
    pub fn get_stock_transfer_detail(&self, id: &str) -> Option<StockTransfer> {
        let conn = self.conn.lock();
        let mut transfer = conn
            .query_row(
                "SELECT id, tenant_id, from_branch_id, from_branch_name, to_branch_id, to_branch_name, status, notes, created_by, created_at, received_at
                 FROM stock_transfers WHERE id = ?1",
                params![id],
                |row| {
                    Ok(StockTransfer {
                        id: row.get(0)?,
                        tenant_id: row.get(1)?,
                        from_branch_id: row.get(2)?,
                        from_branch_name: row.get(3)?,
                        to_branch_id: row.get(4)?,
                        to_branch_name: row.get(5)?,
                        status: row.get(6)?,
                        notes: row.get(7)?,
                        created_by: row.get(8)?,
                        items: Vec::new(),
                        created_at: row.get(9)?,
                        received_at: row.get(10)?,
                    })
                },
            )
            .ok()?;

        let mut item_stmt = conn
            .prepare(
                "SELECT product_id, barcode, name, quantity, unit, received_quantity
                 FROM stock_transfer_items WHERE transfer_id = ?1",
            )
            .ok()?;

        let items = item_stmt
            .query_map(params![id], |row| {
                Ok(StockTransferItem {
                    product_id: row.get(0)?,
                    barcode: row.get(1)?,
                    name: row.get(2)?,
                    quantity: row.get(3)?,
                    unit: row.get(4)?,
                    received_quantity: row.get(5)?,
                })
            })
            .ok()?
            .filter_map(Result::ok)
            .collect();

        transfer.items = items;
        Some(transfer)
    }

    /// Gelen Transferi Teslim Al / Stoğa Kabul Et (Received)
    pub fn receive_stock_transfer(&self, transfer_id: &str) -> Result<StockTransfer> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        let now = Local::now().to_rfc3339();

        // 1. Durumu RECEIVED yap
        let rows = tx.execute(
            "UPDATE stock_transfers SET status = 'RECEIVED', received_at = ?1 WHERE id = ?2 AND status = 'DISPATCHED'",
            params![now, transfer_id],
        )?;

        if rows == 0 {
            return Err(anyhow!("Transfer zaten kabul edilmiş veya bulunamadı"));
        }

        // 2. Kalemleri oku ve stoğu artır
        let mut items = Vec::new();
        {
            let mut stmt = tx.prepare(
                "SELECT product_id, barcode, name, quantity, unit
                 FROM stock_transfer_items WHERE transfer_id = ?1",
            )?;
            let rows = stmt.query_map(params![transfer_id], |row| {
                Ok(StockTransferItem {
                    product_id: row.get(0)?,
                    barcode: row.get(1)?,
                    name: row.get(2)?,
                    quantity: row.get(3)?,
                    unit: row.get(4)?,
                    received_quantity: row.get(3)?,
                })
            })?;

            for r in rows {
                items.push(r?);
            }
        }

        for item in &items {
            tx.execute(
                "UPDATE stock_transfer_items SET received_quantity = ?1 WHERE transfer_id = ?2 AND barcode = ?3",
                params![item.quantity, transfer_id, item.barcode],
            )?;

            tx.execute(
                "UPDATE products SET stock_qty = stock_qty + ?1, updated_at = ?2 WHERE barcode = ?3 OR id = ?4",
                params![item.quantity, now, item.barcode, item.product_id],
            )?;
        }

        tx.commit()?;
        drop(conn);

        self.get_stock_transfer_detail(transfer_id)
            .ok_or_else(|| anyhow!("Detay alınamadı"))
    }
}
