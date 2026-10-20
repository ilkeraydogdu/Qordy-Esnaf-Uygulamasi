//! QORDY POS - Kasa & Vardiya Yönetimi Deposu (Açılış, Kapanış, X-Raporu, Kasa Farkı)

use crate::models::{CloseShiftRequest, OpenShiftRequest, RegisterShift, ZReport};
use crate::storage::DatabaseManager;
use anyhow::{anyhow, Result};
use chrono::Local;
use rusqlite::params;
use uuid::Uuid;

impl DatabaseManager {
    /// Aktif Açık Vardiyayı Getir
    pub fn get_active_shift(&self) -> Option<RegisterShift> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT id, tenant_id, branch_id, register_name, cashier_name, opening_cash, closing_cash, expected_cash, cash_difference, notes, status, opened_at, closed_at
             FROM register_shifts WHERE status = 'OPEN' ORDER BY opened_at DESC LIMIT 1",
            [],
            |row| {
                Ok(RegisterShift {
                    id: row.get(0)?,
                    tenant_id: row.get(1)?,
                    branch_id: row.get(2)?,
                    register_name: row.get(3)?,
                    cashier_name: row.get(4)?,
                    opening_cash: row.get(5)?,
                    closing_cash: row.get(6)?,
                    expected_cash: row.get(7)?,
                    cash_difference: row.get(8)?,
                    notes: row.get(9)?,
                    status: row.get(10)?,
                    opened_at: row.get(11)?,
                    closed_at: row.get(12)?,
                })
            },
        )
        .ok()
    }

    /// Kasa Açılışı Yap (Gün Başı Nakit Devri - Float)
    pub fn open_shift(&self, req: OpenShiftRequest) -> Result<RegisterShift> {
        if let Some(active) = self.get_active_shift() {
            return Err(anyhow!(
                "Halen açık bir kasa vardiyası mevcut (Vardiya ID: {}). Lütfen önce mevcut kasayı kapatın.",
                active.id
            ));
        }

        let conn = self.conn.lock();
        let shift_id = format!("SHF-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        let tenant_id = self.config.tenant_id.clone();
        let branch_id = self.config.branch_id.clone();
        let now = Local::now().to_rfc3339();

        conn.execute(
            "INSERT INTO register_shifts (id, tenant_id, branch_id, register_name, cashier_name, opening_cash, status, opened_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'OPEN', ?7)",
            params![
                shift_id,
                tenant_id,
                branch_id,
                req.register_name,
                req.cashier_name,
                req.opening_cash,
                now
            ],
        )?;

        // Ayrıca cash_movements tablosuna AÇILIŞ DEVRİ olarak işle
        let mv_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO cash_movements (id, movement_type, amount, reason, register_id, cashier_name, created_at)
             VALUES (?1, 'IN', ?2, 'GÜN BAŞI KASA DEVRİ (FLOAT)', ?3, ?4, ?5)",
            params![mv_id, req.opening_cash, req.register_name, req.cashier_name, now],
        )?;

        Ok(RegisterShift {
            id: shift_id,
            tenant_id,
            branch_id,
            register_name: req.register_name,
            cashier_name: req.cashier_name,
            opening_cash: req.opening_cash,
            closing_cash: None,
            expected_cash: None,
            cash_difference: None,
            notes: None,
            status: "OPEN".to_string(),
            opened_at: now,
            closed_at: None,
        })
    }

    /// Kasa Kapanışı Yap (Gün Sonu Sayımı ve Kasa Farkı Hesaplama)
    pub fn close_shift(&self, req: CloseShiftRequest) -> Result<RegisterShift> {
        let active = self
            .get_active_shift()
            .ok_or_else(|| anyhow!("Kapatılacak açık bir kasa vardiyası bulunamadı."))?;

        let conn = self.conn.lock();
        let now = Local::now().to_rfc3339();

        // Vardiya başlangıcından bu yana yapılan nakit satışlar
        let cash_sales_f64: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(grand_total), 0.0) FROM sales 
                 WHERE payment_method = 'CASH' AND created_at >= ?1",
                params![active.opened_at],
                |r| r.get(0),
            )
            .unwrap_or(0.0);

        // Kasa nakit girişleri (avans)
        let cash_in_f64: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(amount), 0.0) FROM cash_movements 
                 WHERE movement_type = 'IN' AND reason NOT LIKE '%FLOAT%' AND created_at >= ?1",
                params![active.opened_at],
                |r| r.get(0),
            )
            .unwrap_or(0.0);

        // Kasa nakit çıkışları (masraf)
        let cash_out_f64: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(amount), 0.0) FROM cash_movements 
                 WHERE movement_type = 'OUT' AND created_at >= ?1",
                params![active.opened_at],
                |r| r.get(0),
            )
            .unwrap_or(0.0);

        let expected_f64 = active.opening_cash + cash_sales_f64 + cash_in_f64 - cash_out_f64;
        let diff_f64 = req.counted_cash - expected_f64;

        conn.execute(
            "UPDATE register_shifts 
             SET closing_cash = ?1, expected_cash = ?2, cash_difference = ?3, notes = ?4, status = 'CLOSED', closed_at = ?5
             WHERE id = ?6",
            params![req.counted_cash, expected_f64, diff_f64, req.notes, now, active.id],
        )?;

        Ok(RegisterShift {
            id: active.id,
            tenant_id: active.tenant_id,
            branch_id: active.branch_id,
            register_name: active.register_name,
            cashier_name: active.cashier_name,
            opening_cash: active.opening_cash,
            closing_cash: Some(req.counted_cash),
            expected_cash: Some(expected_f64),
            cash_difference: Some(diff_f64),
            notes: req.notes,
            status: "CLOSED".to_string(),
            opened_at: active.opened_at,
            closed_at: Some(now),
        })
    }

    /// X-Raporu Al (Ara Mali Rapor - Sayacı Sıfırlamaz)
    pub fn get_x_report(&self) -> ZReport {
        let mut report = self.generate_z_report().unwrap_or_else(|_| ZReport {
            z_number: format!("X-{}", Local::now().format("%Y%m%d%H%M")),
            branch_name: self.config.branch_name.clone(),
            register_id: self.config.register_id.clone(),
            cashier_name: self.config.cashier_name.clone(),
            date: Local::now().format("%d.%m.%Y %H:%M").to_string(),
            total_sales_count: 0,
            total_gross: 0.0,
            total_discount: 0.0,
            total_net: 0.0,
            cash_total: 0.0,
            card_total: 0.0,
            veresiye_total: 0.0,
            vat_breakdown: Vec::new(),
        });
        report.z_number = format!("X-{}", &report.z_number[2..]);
        report
    }

    /// Vardiya Geçmişi
    pub fn get_shift_history(&self) -> Vec<RegisterShift> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, tenant_id, branch_id, register_name, cashier_name, opening_cash, closing_cash, expected_cash, cash_difference, notes, status, opened_at, closed_at
             FROM register_shifts ORDER BY opened_at DESC LIMIT 30",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let iter = stmt.query_map([], |row| {
            Ok(RegisterShift {
                id: row.get(0)?,
                tenant_id: row.get(1)?,
                branch_id: row.get(2)?,
                register_name: row.get(3)?,
                cashier_name: row.get(4)?,
                opening_cash: row.get(5)?,
                closing_cash: row.get(6)?,
                expected_cash: row.get(7)?,
                cash_difference: row.get(8)?,
                notes: row.get(9)?,
                status: row.get(10)?,
                opened_at: row.get(11)?,
                closed_at: row.get(12)?,
            })
        });

        match iter {
            Ok(rows) => rows.filter_map(Result::ok).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Aktif vardiyadaki beklenen kasa nakdini hesapla
    pub fn calculate_expected_cash(&self, shift_id: &str) -> f64 {
        let conn = self.conn.lock();
        let query_res: rusqlite::Result<(String, f64)> = conn.query_row(
            "SELECT opened_at, opening_cash FROM register_shifts WHERE id = ?1",
            params![shift_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        );

        match query_res {
            Ok((opened_at, opening_cash)) => {
                let cash_sales: f64 = conn
                    .query_row(
                        "SELECT COALESCE(SUM(grand_total), 0.0) FROM sales WHERE payment_method = 'CASH' AND created_at >= ?1",
                        params![opened_at],
                        |r| r.get(0),
                    )
                    .unwrap_or(0.0);
                let cash_in: f64 = conn
                    .query_row(
                        "SELECT COALESCE(SUM(amount), 0.0) FROM cash_movements WHERE movement_type = 'IN' AND reason NOT LIKE '%FLOAT%' AND created_at >= ?1",
                        params![opened_at],
                        |r| r.get(0),
                    )
                    .unwrap_or(0.0);
                let cash_out: f64 = conn
                    .query_row(
                        "SELECT COALESCE(SUM(amount), 0.0) FROM cash_movements WHERE movement_type = 'OUT' AND created_at >= ?1",
                        params![opened_at],
                        |r| r.get(0),
                    )
                    .unwrap_or(0.0);
                opening_cash + cash_sales + cash_in - cash_out
            }
            Err(_) => 0.0,
        }
    }
}
