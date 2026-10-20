//! QORDY POS - Z-Report & Cashier Financial Status Repository

use crate::models::{PosStatus, VatSummary, ZReport};
use crate::storage::DatabaseManager;
use anyhow::Result;
use chrono::Local;
use rusqlite::params;

impl DatabaseManager {
    /// Gün Sonu Z-Raporu Üretimi
    pub fn generate_z_report(&self) -> Result<ZReport> {
        let conn = self.conn.lock();
        let today = Local::now().format("%Y-%m-%d").to_string();

        let mut stmt = conn.prepare(
            "SELECT payment_method, grand_total, discount_amount FROM sales WHERE created_at LIKE ?1",
        )?;

        let mut total_count = 0;
        let mut total_gross = 0.0;
        let mut total_disc = 0.0;
        let mut cash_total = 0.0;
        let mut card_total = 0.0;
        let mut veresiye_total = 0.0;

        let rows = stmt.query_map(params![format!("{}%", today)], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, f64>(2)?,
            ))
        })?;

        for r in rows.flatten() {
            total_count += 1;
            let method = r.0;
            let total = r.1;
            let disc = r.2;

            total_gross += total + disc;
            total_disc += disc;

            match method.as_str() {
                "CASH" => cash_total += total,
                "CREDIT_CARD" => card_total += total,
                "VERESIYE" => veresiye_total += total,
                _ => cash_total += total,
            }
        }

        let total_net = total_gross - total_disc;

        // KDV dökümü
        let mut vat_stmt = conn.prepare(
            "SELECT vat_rate, SUM(total) FROM sale_items si JOIN sales s ON si.sale_id = s.id WHERE s.created_at LIKE ?1 GROUP BY vat_rate",
        )?;

        let vat_rows = vat_stmt.query_map(params![format!("{}%", today)], |row| {
            let rate: f64 = row.get(0)?;
            let total_with_vat: f64 = row.get(1)?;
            let vat_amount = total_with_vat - (total_with_vat / (1.0 + (rate / 100.0)));
            let taxable = total_with_vat - vat_amount;
            Ok(VatSummary {
                rate,
                taxable_amount: taxable,
                vat_amount,
            })
        })?;

        let vat_breakdown: Vec<VatSummary> = vat_rows.filter_map(|r| r.ok()).collect();

        let live_business_name: Option<String> = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'business_name'",
                [],
                |r| r.get(0),
            )
            .ok();

        let live_branch_name: Option<String> = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'branch_name'",
                [],
                |r| r.get(0),
            )
            .ok();

        let live_cashier_name: Option<String> = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'cashier_name'",
                [],
                |r| r.get(0),
            )
            .ok();

        Ok(ZReport {
            z_number: format!("Z-{}", Local::now().format("%Y%m%d%H%M")),
            business_name: live_business_name.unwrap_or_else(|| self.config.business_name.clone()),
            branch_name: live_branch_name.unwrap_or_else(|| self.config.branch_name.clone()),
            register_id: self.config.register_id.clone(),
            cashier_name: live_cashier_name.unwrap_or_else(|| self.config.cashier_name.clone()),
            date: Local::now().format("%d.%m.%Y %H:%M").to_string(),
            total_sales_count: total_count,
            total_gross,
            total_discount: total_disc,
            total_net,
            cash_total,
            card_total,
            veresiye_total,
            vat_breakdown,
        })
    }

    /// Kasa Durumu
    pub fn get_pos_status(&self, is_online: bool) -> PosStatus {
        let conn = self.conn.lock();
        let today = Local::now().format("%Y-%m-%d").to_string();

        let count: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM sales WHERE created_at LIKE ?1",
                params![format!("{}%", today)],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let revenue: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(grand_total), 0.0) FROM sales WHERE created_at LIKE ?1",
                params![format!("{}%", today)],
                |r| r.get(0),
            )
            .unwrap_or(0.0);

        let pending: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM outbox_sync WHERE status = 'PENDING'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let live_business_name: Option<String> = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'business_name'",
                [],
                |r| r.get(0),
            )
            .ok();

        let live_tenant_id: Option<String> = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'tenant_id'",
                [],
                |r| r.get(0),
            )
            .ok();

        let live_branch_name: Option<String> = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'branch_name'",
                [],
                |r| r.get(0),
            )
            .ok();

        let live_cashier_name: Option<String> = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'cashier_name'",
                [],
                |r| r.get(0),
            )
            .ok();

        let local_ip = local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        PosStatus {
            version: "3.0.0".to_string(),
            tenant_id: live_tenant_id.unwrap_or_else(|| self.config.tenant_id.clone()),
            business_name: live_business_name.unwrap_or_else(|| self.config.business_name.clone()),
            branch_id: self.config.branch_id.clone(),
            branch_name: live_branch_name.unwrap_or_else(|| self.config.branch_name.clone()),
            register_id: self.config.register_id.clone(),
            cashier_name: live_cashier_name.unwrap_or_else(|| self.config.cashier_name.clone()),
            is_online,
            pending_sync_count: pending,
            today_sales_count: count,
            today_revenue: revenue,
            local_ip,
            port: self.config.port,
        }
    }
}
