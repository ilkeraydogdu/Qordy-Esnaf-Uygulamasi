//! QORDY POS - Sales, Checkout, Cart Parking & Refund Repository

use crate::models::{
    CartItem, CheckoutRequest, ParkedCart, RefundRequest, RefundResponse, Sale,
};
use crate::storage::DatabaseManager;
use anyhow::Result;
use chrono::Local;
use rusqlite::params;
use uuid::Uuid;

impl DatabaseManager {
    /// Çevrimdışı Satışı Kaydet (Atomik Transaction: Satış + Stok Düşümü + Veresiye + Outbox Kuyruğu)
    pub fn process_checkout(&self, req: CheckoutRequest) -> Result<Sale> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;

        let now = Local::now();
        let sale_id = Uuid::new_v4().to_string();
        let receipt_no = format!(
            "ESN-{}-{:06}",
            now.format("%Y%m%d%H%M%S"),
            now.timestamp_subsec_micros()
        );

        let tenant_id = "CUST_69653cfccb839".to_string();
        let branch_id = "BR_01".to_string();
        let branch_name = "Kadıköy Çarşı Şubesi".to_string();
        let register_id = "KASA-01".to_string();
        let cashier_id = "USR_01".to_string();
        let cashier_name = "Ahmet Kasiyer".to_string();

        let mut subtotal = 0.0;
        let mut vat_total = 0.0;

        for item in &req.items {
            subtotal += item.total;
            let item_vat = item.total - (item.total / (1.0 + (item.vat_rate / 100.0)));
            vat_total += item_vat;

            // Stok düşümü yap
            tx.execute(
                "UPDATE products SET stock_qty = stock_qty - ?1 WHERE id = ?2",
                params![item.quantity, item.product_id],
            )?;
        }

        let discount = req.discount_amount.unwrap_or(0.0);
        let grand_total = (subtotal - discount).max(0.0);

        let tender = req.tender_amount.unwrap_or(grand_total);
        let change = (tender - grand_total).max(0.0);

        let customer_name = if let Some(ref cid) = req.customer_id {
            let cname: Option<String> = tx
                .query_row(
                    "SELECT name FROM customers WHERE id = ?1",
                    params![cid],
                    |r| r.get(0),
                )
                .ok();
            cname
        } else {
            None
        };

        // Veresiye ise müşterinin borç bakiyesini artır ve deftere yaz
        if req.payment_method == "VERESIYE" {
            if let Some(ref cid) = req.customer_id {
                tx.execute(
                    "UPDATE customers SET balance = balance + ?1, updated_at = ?2 WHERE id = ?3",
                    params![grand_total, now.to_rfc3339(), cid],
                )?;

                let new_bal: f64 = tx.query_row(
                    "SELECT balance FROM customers WHERE id = ?1",
                    params![cid],
                    |r| r.get(0),
                )?;

                let ledger_id = Uuid::new_v4().to_string();
                tx.execute(
                    "INSERT INTO ledger_entries (id, tenant_id, customer_id, customer_name, entry_type, amount, balance_after, note, created_at, sync_status)
                     VALUES (?1, ?2, ?3, ?4, 'DEBT', ?5, ?6, ?7, ?8, 'PENDING')",
                    params![
                        ledger_id,
                        tenant_id,
                        cid,
                        customer_name.as_deref().unwrap_or("Müşteri"),
                        grand_total,
                        new_bal,
                        format!("Fiş: #{}", receipt_no),
                        now.to_rfc3339()
                    ],
                )?;
            }
        }

        // Satış başlığını kaydet
        tx.execute(
            "INSERT INTO sales (id, receipt_no, tenant_id, branch_id, branch_name, register_id, cashier_id, cashier_name, subtotal, vat_amount, discount_amount, grand_total, payment_method, customer_id, customer_name, tender_amount, change_amount, created_at, sync_status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, 'PENDING')",
            params![
                sale_id,
                receipt_no,
                tenant_id,
                branch_id,
                branch_name,
                register_id,
                cashier_id,
                cashier_name,
                subtotal,
                vat_total,
                discount,
                grand_total,
                req.payment_method,
                req.customer_id,
                customer_name,
                tender,
                change,
                now.to_rfc3339()
            ],
        )?;

        // Kalemleri kaydet
        for item in &req.items {
            tx.execute(
                "INSERT INTO sale_items (sale_id, product_id, barcode, name, unit_price, quantity, unit, vat_rate, discount, total, is_weighed, notes)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    sale_id,
                    item.product_id,
                    item.barcode,
                    item.name,
                    item.unit_price,
                    item.quantity,
                    item.unit,
                    item.vat_rate,
                    item.discount,
                    item.total,
                    if item.is_weighed { 1 } else { 0 },
                    item.notes
                ],
            )?;
        }

        let sale = Sale {
            id: sale_id.clone(),
            receipt_no: receipt_no.clone(),
            tenant_id: tenant_id.clone(),
            branch_id,
            branch_name,
            register_id,
            cashier_id,
            cashier_name,
            subtotal,
            vat_amount: vat_total,
            discount_amount: discount,
            grand_total,
            payment_method: req.payment_method,
            customer_id: req.customer_id,
            customer_name,
            tender_amount: tender,
            change_amount: change,
            items: req.items,
            created_at: now.to_rfc3339(),
            sync_status: "PENDING".to_string(),
        };

        // Outbox tablosuna yaz (İnternet geldiğinde buluta basılacak)
        let outbox_id = Uuid::new_v4().to_string();
        let payload = serde_json::to_string(&sale)?;
        tx.execute(
            "INSERT INTO outbox_sync (id, entity_type, payload_json, idempotency_key, created_at, status)
             VALUES (?1, 'SALE', ?2, ?3, ?4, 'PENDING')",
            params![outbox_id, payload, sale_id, now.to_rfc3339()],
        )?;

        tx.commit()?;

        Ok(sale)
    }

    /// Kısmi veya Tam İade İşlemi
    pub fn process_refund(&self, req: RefundRequest) -> Result<RefundResponse> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;

        let now = Local::now();
        let refund_id = Uuid::new_v4().to_string();
        let refund_receipt_no = format!(
            "IAD-{}-{:06}",
            now.format("%Y%m%d%H%M%S"),
            now.timestamp_subsec_micros()
        );

        // İade edilen ürünlerin stoklarını geri yükle
        for item in &req.items {
            tx.execute(
                "UPDATE products SET stock_qty = stock_qty + ?1 WHERE id = ?2",
                params![item.quantity, item.product_id],
            )?;
        }

        let items_json = serde_json::to_string(&req.items)?;

        tx.execute(
            "INSERT INTO refunds (id, sale_id, refund_receipt_no, items_json, refund_amount, reason, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                refund_id,
                req.sale_id,
                refund_receipt_no,
                items_json,
                req.refund_amount,
                req.reason,
                now.to_rfc3339()
            ],
        )?;

        tx.commit()?;

        Ok(RefundResponse {
            success: true,
            refund_id,
            refund_receipt_no,
            refunded_amount: req.refund_amount,
            message: "İade işlemi başarıyla tamamlandı ve stoklar iade edildi.".to_string(),
        })
    }

    /// Son İşlem Geçmişi (Fiş Listesi ve Kalemleri)
    pub fn get_sales_history(&self, limit: usize) -> Vec<Sale> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, receipt_no, tenant_id, branch_id, branch_name, register_id, cashier_id, cashier_name,
                    subtotal, vat_amount, discount_amount, grand_total, payment_method, customer_id, customer_name,
                    tender_amount, change_amount, created_at, sync_status
             FROM sales ORDER BY created_at DESC LIMIT ?1",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let mut sales: Vec<Sale> = stmt
            .query_map(params![limit], |r| {
                Ok(Sale {
                    id: r.get(0)?,
                    receipt_no: r.get(1)?,
                    tenant_id: r.get(2)?,
                    branch_id: r.get(3)?,
                    branch_name: r.get(4)?,
                    register_id: r.get(5)?,
                    cashier_id: r.get(6)?,
                    cashier_name: r.get(7)?,
                    subtotal: r.get(8)?,
                    vat_amount: r.get(9)?,
                    discount_amount: r.get(10)?,
                    grand_total: r.get(11)?,
                    payment_method: r.get(12)?,
                    customer_id: r.get(13)?,
                    customer_name: r.get(14)?,
                    tender_amount: r.get(15)?,
                    change_amount: r.get(16)?,
                    items: Vec::new(),
                    created_at: r.get(17)?,
                    sync_status: r.get(18)?,
                })
            })
            .map(|iter| iter.filter_map(|r| r.ok()).collect())
            .unwrap_or_default();

        // Her satışın kalemlerini doldur
        for sale in &mut sales {
            let mut item_stmt = match conn.prepare(
                "SELECT product_id, barcode, name, unit_price, quantity, unit, vat_rate, discount, total, is_weighed, notes
                 FROM sale_items WHERE sale_id = ?1",
            ) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let items = item_stmt
                .query_map(params![sale.id], |r| {
                    Ok(CartItem {
                        product_id: r.get(0)?,
                        barcode: r.get(1)?,
                        name: r.get(2)?,
                        unit_price: r.get(3)?,
                        quantity: r.get(4)?,
                        unit: r.get(5)?,
                        vat_rate: r.get(6)?,
                        discount: r.get(7)?,
                        total: r.get(8)?,
                        is_weighed: r.get::<_, i32>(9)? == 1,
                        notes: r.get(10)?,
                    })
                })
                .map(|iter| iter.filter_map(|r| r.ok()).collect())
                .unwrap_or_default();

            sale.items = items;
        }

        sales
    }

    /// Sepeti Beklet / Park Et (F7)
    pub fn park_cart(&self, note: &str, items: Vec<CartItem>) -> Result<ParkedCart> {
        let conn = self.conn.lock();
        let id = Uuid::new_v4().to_string();
        let now = Local::now().to_rfc3339();
        let grand_total: f64 = items.iter().map(|i| i.total).sum();
        let item_count = items.len();
        let items_json = serde_json::to_string(&items)?;

        conn.execute(
            "INSERT INTO parked_carts (id, note, items_json, grand_total, item_count, parked_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, note, items_json, grand_total, item_count, now],
        )?;

        Ok(ParkedCart {
            id,
            note: note.to_string(),
            items,
            grand_total,
            item_count,
            parked_at: now,
        })
    }

    /// Bekletilen Sepetleri Getir
    pub fn get_parked_carts(&self) -> Vec<ParkedCart> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, note, items_json, grand_total, item_count, parked_at FROM parked_carts ORDER BY parked_at DESC",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map([], |r| {
            let json_str: String = r.get(2)?;
            let items: Vec<CartItem> = serde_json::from_str(&json_str).unwrap_or_default();
            Ok(ParkedCart {
                id: r.get(0)?,
                note: r.get(1)?,
                items,
                grand_total: r.get(3)?,
                item_count: r.get(4)?,
                parked_at: r.get(5)?,
            })
        });

        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Bekletilen Sepeti Sil (Kasaya geri çağrıldığında veya iptal edildiğinde)
    pub fn delete_parked_cart(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM parked_carts WHERE id = ?1", params![id])?;
        Ok(())
    }
}
