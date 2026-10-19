#[cfg(test)]
mod tests {
    use qordy_pos_rs::barcode::BarcodeParser;
    use qordy_pos_rs::escpos::ReceiptPrinter;
    use qordy_pos_rs::models::{CartItem, CheckoutRequest};
    use qordy_pos_rs::storage::DatabaseManager;
    use tempfile::NamedTempFile;

    #[test]
    fn test_scale_barcode_grams_parsing() {
        // 27 (prefix) + 00124 (domates) + 01450 (1.450 kg) + 3 (checksum)
        let parsed = BarcodeParser::parse("2700124014503");
        assert!(parsed.is_weighed);
        assert_eq!(parsed.item_code.as_deref(), Some("00124"));
        assert_eq!(parsed.lookup_code, "2700124");
        assert_eq!(parsed.weight_kg, Some(1.450));
        assert_eq!(parsed.embedded_price, None);
    }

    #[test]
    fn test_scale_barcode_price_parsing() {
        // 28 (prefix) + 00124 (kıyma) + 08550 (85.50 TL) + 3 (checksum)
        let parsed = BarcodeParser::parse("2800124085503");
        assert!(parsed.is_weighed);
        assert_eq!(parsed.lookup_code, "2800124");
        assert_eq!(parsed.embedded_price, Some(85.50));
        assert_eq!(parsed.weight_kg, None);
    }

    #[test]
    fn test_offline_database_lifecycle_and_checkout() {
        let temp_file = NamedTempFile::new().expect("Temp file failed");
        let db = DatabaseManager::new(temp_file.path()).expect("DB init failed");

        // 1. Ürün sorgulama
        let bread = db.find_product_by_barcode("8690504018087").expect("Ekmek bulunmalı");
        assert_eq!(bread.name, "Somun Ekmek (200g)");
        assert_eq!(bread.price, 10.0);

        // 2. Terazi ürünü sorgulama (Domates - 2700124)
        let tomato = db.find_product_by_barcode("2700124").expect("Domates bulunmalı");
        assert!(tomato.is_weighed);
        assert_eq!(tomato.price, 42.0);

        // 3. Nakit Satış Simülasyonu
        let cart_items = vec![
            CartItem {
                product_id: bread.id.clone(),
                barcode: bread.barcode.clone(),
                name: bread.name.clone(),
                unit_price: bread.price,
                quantity: 2.0,
                unit: "adet".to_string(),
                vat_rate: bread.vat_rate,
                discount: 0.0,
                total: 20.0,
                is_weighed: false,
                notes: None,
            },
            CartItem {
                product_id: tomato.id.clone(),
                barcode: "2700124014503".to_string(),
                name: tomato.name.clone(),
                unit_price: tomato.price,
                quantity: 1.5,
                unit: "kg".to_string(),
                vat_rate: tomato.vat_rate,
                discount: 0.0,
                total: 63.0,
                is_weighed: true,
                notes: None,
            },
        ];

        let req = CheckoutRequest {
            items: cart_items,
            payment_method: "CASH".to_string(),
            customer_id: None,
            tender_amount: Some(100.0),
            discount_amount: Some(0.0),
            notes: None,
        };

        let sale = db.process_checkout(req).expect("Satış işlenmeli");
        assert_eq!(sale.grand_total, 83.0);
        assert_eq!(sale.change_amount, 17.0);
        assert_eq!(sale.sync_status, "PENDING");

        // Outbox kuyruğunda 1 bekleyen işlem olmalı
        assert_eq!(db.get_pending_sync_count(), 1);

        // Termal fiş çıktısı doğrulaması
        let receipt_bytes = ReceiptPrinter::build_sale_receipt(&sale);
        assert!(!receipt_bytes.is_empty());
        let receipt_str = String::from_utf8_lossy(&receipt_bytes);
        assert!(receipt_str.contains("QORDY PERAKENDE"));
        assert!(receipt_str.contains("Somun Ekmek"));
        assert!(receipt_str.contains("GENEL TOPLAM"));

        // 4. Veresiye Satış Simülasyonu (Komşu Ahmet Bey: CUST-001)
        let customers = db.get_customers();
        let ahmet = customers.iter().find(|c| c.id == "CUST-001").expect("Ahmet bey var");
        let initial_debt = ahmet.balance;

        let veresiye_req = CheckoutRequest {
            items: vec![CartItem {
                product_id: bread.id.clone(),
                barcode: bread.barcode.clone(),
                name: bread.name.clone(),
                unit_price: bread.price,
                quantity: 3.0,
                unit: "adet".to_string(),
                vat_rate: bread.vat_rate,
                discount: 0.0,
                total: 30.0,
                is_weighed: false,
                notes: None,
            }],
            payment_method: "VERESIYE".to_string(),
            customer_id: Some("CUST-001".to_string()),
            tender_amount: None,
            discount_amount: None,
            notes: None,
        };

        let veresiye_sale = db.process_checkout(veresiye_req).expect("Veresiye satışı işlenmeli");
        assert_eq!(veresiye_sale.grand_total, 30.0);

        // Müşteri borcu güncellenmiş olmalı
        let updated_customers = db.get_customers();
        let updated_ahmet = updated_customers.iter().find(|c| c.id == "CUST-001").unwrap();
        assert_eq!(updated_ahmet.balance, initial_debt + 30.0);

        // 5. Tahsilat Kaydı (Ahmet bey borcunun 50 TL'sini ödedi)
        let payment_entry = db.record_customer_payment("CUST-001", 50.0, "Elden nakit ödeme").expect("Tahsilat yapılmalı");
        assert_eq!(payment_entry.amount, 50.0);
        assert_eq!(payment_entry.balance_after, updated_ahmet.balance - 50.0);

        // 6. Gün Sonu Z-Raporu
        let z_report = db.generate_z_report().expect("Z raporu üretilmeli");
        assert_eq!(z_report.total_sales_count, 2);
        assert_eq!(z_report.cash_total, 83.0);
        assert_eq!(z_report.veresiye_total, 30.0);
        assert_eq!(z_report.total_net, 113.0);

        // Outbox'ta 3 işlem bekliyor olmalı (Nakit satış, Veresiye satış, Tahsilat)
        assert_eq!(db.get_pending_sync_count(), 3);

        // Eşitleme işaretleme
        let pending = db.get_pending_outbox_items(10);
        for (outbox_id, _, _) in pending {
            db.mark_outbox_synced(&outbox_id).unwrap();
        }
        assert_eq!(db.get_pending_sync_count(), 0);
    }
}
