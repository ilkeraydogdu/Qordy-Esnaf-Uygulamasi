//! QORDY POS - Kampanya ve İndirim Motoru Deposu

use crate::models::{Campaign, CartItem, CreateCampaignRequest};
use crate::storage::DatabaseManager;
use anyhow::{anyhow, Result};
use chrono::Local;
use rusqlite::params;
use uuid::Uuid;

impl DatabaseManager {
    /// Tüm Kampanyaları Listele
    pub fn get_campaigns(&self) -> Vec<Campaign> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, tenant_id, name, campaign_type, discount_percent, discount_amount, min_cart_amount, buy_qty, pay_qty, target_product_id, target_category, is_active, created_at
             FROM campaigns ORDER BY created_at DESC",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let iter = stmt.query_map([], |row| {
            let is_act: i32 = row.get(11)?;

            Ok(Campaign {
                id: row.get(0)?,
                tenant_id: row.get(1)?,
                name: row.get(2)?,
                campaign_type: row.get(3)?,
                discount_percent: row.get(4)?,
                discount_amount: row.get(5)?,
                min_cart_amount: row.get(6)?,
                buy_qty: row.get(7)?,
                pay_qty: row.get(8)?,
                target_product_id: row.get(9)?,
                target_category: row.get(10)?,
                is_active: is_act == 1,
                created_at: row.get(12)?,
            })
        });

        match iter {
            Ok(rows) => rows.filter_map(Result::ok).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Yeni Kampanya Tanımla
    pub fn create_campaign(&self, req: CreateCampaignRequest) -> Result<Campaign> {
        let conn = self.conn.lock();
        let id = format!("CMP-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        let tenant_id = self.config.tenant_id.clone();
        let now = Local::now().to_rfc3339();

        conn.execute(
            "INSERT INTO campaigns (id, tenant_id, name, campaign_type, discount_percent, discount_amount, min_cart_amount, buy_qty, pay_qty, target_product_id, target_category, is_active, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 1, ?12)",
            params![
                id,
                tenant_id,
                req.name,
                req.campaign_type,
                req.discount_percent,
                req.discount_amount,
                req.min_cart_amount,
                req.buy_qty,
                req.pay_qty,
                req.target_product_id,
                req.target_category,
                now
            ],
        )?;

        Ok(Campaign {
            id,
            tenant_id,
            name: req.name,
            campaign_type: req.campaign_type,
            discount_percent: req.discount_percent,
            discount_amount: req.discount_amount,
            min_cart_amount: req.min_cart_amount,
            buy_qty: req.buy_qty,
            pay_qty: req.pay_qty,
            target_product_id: req.target_product_id,
            target_category: req.target_category,
            is_active: true,
            created_at: now,
        })
    }

    /// Kampanya Durumunu Aç / Kapat
    pub fn toggle_campaign(&self, id: &str) -> Result<bool> {
        let conn = self.conn.lock();
        let current: i32 = conn.query_row(
            "SELECT is_active FROM campaigns WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )?;
        let new_status = if current == 1 { 0 } else { 1 };
        conn.execute(
            "UPDATE campaigns SET is_active = ?1 WHERE id = ?2",
            params![new_status, id],
        )?;
        Ok(new_status == 1)
    }

    /// Kampanyayı Sil
    pub fn delete_campaign(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock();
        let rows = conn.execute("DELETE FROM campaigns WHERE id = ?1", params![id])?;
        if rows == 0 {
            return Err(anyhow!("Kampanya bulunamadı"));
        }
        Ok(())
    }

    /// Sepet için Otomatik En Avantajlı Kampanya İndirimini Hesapla
    pub fn calculate_campaign_discount(&self, subtotal: f64, items: &[CartItem]) -> (f64, Option<String>) {
        let campaigns = self.get_campaigns();
        let mut max_discount = 0.0;
        let mut applied_campaign_name = None;

        for camp in campaigns {
            if !camp.is_active {
                continue;
            }

            // Sepet alt limit kontrolü
            if let Some(min_amt) = camp.min_cart_amount {
                if subtotal < min_amt {
                    continue;
                }
            }

            let discount = match camp.campaign_type.as_str() {
                "PERCENT" => {
                    if let Some(pct) = camp.discount_percent {
                        (subtotal * pct) / 100.0
                    } else {
                        0.0
                    }
                }
                "AMOUNT" => camp.discount_amount.unwrap_or(0.0),
                "BUY_X_PAY_Y" => {
                    let buy = camp.buy_qty.unwrap_or(2) as f64;
                    let pay = camp.pay_qty.unwrap_or(1) as f64;
                    if buy > pay {
                        let free_count = buy - pay;
                        let mut eligible_discount = 0.0;
                        for item in items {
                            let match_product = camp.target_product_id.as_deref().map_or(true, |pid| pid == item.product_id);
                            if match_product && item.quantity >= buy {
                                let sets = (item.quantity / buy).floor();
                                eligible_discount += sets * free_count * item.unit_price;
                            }
                        }
                        eligible_discount
                    } else {
                        0.0
                    }
                }
                _ => 0.0,
            };

            if discount > max_discount {
                max_discount = discount;
                applied_campaign_name = Some(camp.name.clone());
            }
        }

        // İndirim sepet toplamından fazla olamaz
        if max_discount > subtotal {
            max_discount = subtotal;
        }

        (max_discount, applied_campaign_name)
    }
}
