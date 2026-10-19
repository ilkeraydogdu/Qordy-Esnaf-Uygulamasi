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

    #[test]
    fn test_product_crud_and_categories() {
        let temp_file = NamedTempFile::new().expect("Temp file failed");
        let db = DatabaseManager::new(temp_file.path()).expect("DB init failed");

        use qordy_pos_rs::models::CreateProductRequest;

        // 1. Yeni ürün ekle
        let req = CreateProductRequest {
            barcode: "8690001112223".to_string(),
            name: "Organik Bal 500g".to_string(),
            category: "Kahvaltılık".to_string(),
            price: 185.0,
            vat_rate: 1.0,
            stock_qty: 25.0,
            unit: "adet".to_string(),
            is_weighed: false,
            quick_order: Some(99),
        };

        let created = db.create_product(req).expect("Ürün oluşturulmalı");
        assert_eq!(created.name, "Organik Bal 500g");

        // 2. Barkod ile bul
        let found = db.find_product_by_barcode("8690001112223").expect("Bulunmalı");
        assert_eq!(found.price, 185.0);

        // 3. Fiyat güncelle
        let update_req = CreateProductRequest {
            barcode: "8690001112223".to_string(),
            name: "Organik Bal 500g (İndirimli)".to_string(),
            category: "Kahvaltılık".to_string(),
            price: 165.0,
            vat_rate: 1.0,
            stock_qty: 30.0,
            unit: "adet".to_string(),
            is_weighed: false,
            quick_order: Some(99),
        };
        let updated = db.update_product(&created.id, update_req).expect("Güncellenmeli");
        assert_eq!(updated.price, 165.0);
        assert_eq!(updated.name, "Organik Bal 500g (İndirimli)");

        // 4. Sil
        db.delete_product(&created.id).expect("Silinmeli");
        assert!(db.find_product_by_barcode("8690001112223").is_none());
    }

    #[test]
    fn test_cart_parking_and_recall() {
        let temp_file = NamedTempFile::new().expect("Temp file failed");
        let db = DatabaseManager::new(temp_file.path()).expect("DB init failed");

        let items = vec![CartItem {
            product_id: "PRD-001".to_string(),
            barcode: "8690504018087".to_string(),
            name: "Somun Ekmek".to_string(),
            unit_price: 10.0,
            quantity: 3.0,
            unit: "adet".to_string(),
            vat_rate: 1.0,
            discount: 0.0,
            total: 30.0,
            is_weighed: false,
            notes: None,
        }];

        // Sepeti beklet
        let parked = db.park_cart("Masa 4 - Bekleyen", items).expect("Sepet park edilmeli");
        assert_eq!(parked.grand_total, 30.0);
        assert_eq!(parked.item_count, 1);

        // Listele
        let parked_list = db.get_parked_carts();
        assert_eq!(parked_list.len(), 1);
        assert_eq!(parked_list[0].note, "Masa 4 - Bekleyen");

        // Sil / Çağır
        db.delete_parked_cart(&parked.id).expect("Silinmeli");
        assert_eq!(db.get_parked_carts().len(), 0);
    }

    #[test]
    fn test_stock_count_session_and_commit() {
        let temp_file = NamedTempFile::new().expect("Temp file failed");
        let db = DatabaseManager::new(temp_file.path()).expect("DB init failed");

        // Başlangıç ekmek stoğu: 150 adet
        let initial_bread = db.find_product_by_barcode("8690504018087").unwrap();
        assert_eq!(initial_bread.stock_qty, 150.0);

        // Sayım oturumu başlat
        let session = db.start_stock_count("Akşam Kapanış Sayımı", "FULL", None).expect("Sayım başlamalı");
        assert_eq!(session.status, "OPEN");

        // Sayımda rafta 142 ekmek sayıldı (8 adet eksik / satılmış / fire)
        let count_item = db.add_stock_count_item(&session.id, "8690504018087", 142.0).expect("Kalem eklenmeli");
        assert_eq!(count_item.system_qty, 150.0);
        assert_eq!(count_item.counted_qty, 142.0);
        assert_eq!(count_item.variance_qty, -8.0);

        // Sayımı onayla ve stokları güncelle
        db.commit_stock_count(&session.id).expect("Sayım onaylanmalı");

        // Ekmek stoğu artık 142 olmalı
        let updated_bread = db.find_product_by_barcode("8690504018087").unwrap();
        assert_eq!(updated_bread.stock_qty, 142.0);
    }

    #[test]
    fn test_customer_facing_display_cfd_state() {
        let temp_file = NamedTempFile::new().expect("Temp file failed");
        let db = DatabaseManager::new(temp_file.path()).expect("DB init failed");

        use qordy_pos_rs::models::CfdState;

        let initial_cfd = db.get_cfd_state();
        assert_eq!(initial_cfd.state, "IDLE");

        let new_state = CfdState {
            state: "ACTIVE_SALE".to_string(),
            items: Vec::new(),
            subtotal: 120.0,
            discount: 0.0,
            grand_total: 120.0,
            tender: 0.0,
            change: 0.0,
            payment_method: None,
            customer_name: Some("Ahmet Bey".to_string()),
            updated_at: "2026-10-20T00:30:00Z".to_string(),
        };

        db.update_cfd_state(&new_state).expect("CFD güncellenmeli");
        let retrieved = db.get_cfd_state();
        assert_eq!(retrieved.state, "ACTIVE_SALE");
        assert_eq!(retrieved.grand_total, 120.0);
    }
}
