#[cfg(test)]
mod enterprise_tests {
    use qordy_pos_rs::models::{
        CartItem, CheckoutRequest, CloseShiftRequest, CreateCampaignRequest,
        CreateGoodsReceiptRequest, CreateStockTransferRequest, CreateSupplierRequest,
        GoodsReceiptItem, NationalProductInfo, OpenShiftRequest, SplitPayment, StockTransferItem,
    };
    use qordy_pos_rs::storage::DatabaseManager;
    use tempfile::NamedTempFile;

    #[test]
    fn test_supplier_crud_and_goods_receipt_stock_increment() {
        let temp_file = NamedTempFile::new().expect("Temp file failed");
        let db = DatabaseManager::new(temp_file.path()).expect("DB init failed");

        // 1. Tedarikçi Ekle
        let supplier = db
            .create_supplier(CreateSupplierRequest {
                name: "Anadolu Gıda Toptan A.Ş.".to_string(),
                contact_name: Some("Ahmet Yılmaz".to_string()),
                phone: Some("0216 555 1234".to_string()),
                email: Some("ahmet@anadolugida.com".to_string()),
                address: Some("Kadıköy Hal İçi No:12".to_string()),
            })
            .expect("Tedarikçi eklenmeli");

        assert_eq!(supplier.name, "Anadolu Gıda Toptan A.Ş.");

        let suppliers = db.get_suppliers();
        assert!(!suppliers.is_empty());
        assert_eq!(suppliers[0].id, supplier.id);

        // 2. Mal Kabul Öncesi Ekmek Stoğunu Kontrol Et (150 adet seed edilmişti)
        let bread_before = db
            .find_product_by_barcode("8690504018087")
            .expect("Ekmek var");
        assert_eq!(bread_before.stock_qty, 150.0);

        // 3. 50 Adet Ekmek Alış İrsaliyesi İle Mal Kabul Yap (Alış Maliyeti: 7.50 TL)
        let receipt = db
            .process_goods_receipt(CreateGoodsReceiptRequest {
                supplier_id: supplier.id.clone(),
                supplier_name: supplier.name.clone(),
                invoice_no: "IRS-2026-0099".to_string(),
                receipt_date: "2026-10-20".to_string(),
                notes: Some("Haftalık ekmek sevkıyatı".to_string()),
                items: vec![GoodsReceiptItem {
                    product_id: bread_before.id.clone(),
                    barcode: bread_before.barcode.clone(),
                    name: bread_before.name.clone(),
                    quantity: 50.0,
                    unit: "adet".to_string(),
                    cost_price: 7.50,
                    sale_price: 10.0,
                    total_cost: 375.0,
                }],
            })
            .expect("Mal kabul fişi işlenmeli");

        assert_eq!(receipt.invoice_no, "IRS-2026-0099");
        assert_eq!(receipt.total_amount, 375.0); // 50 * 7.50

        // 4. Ekmek Stoğu 150 + 50 = 200 Olmalı
        let bread_after = db
            .find_product_by_barcode("8690504018087")
            .expect("Ekmek var");
        assert_eq!(bread_after.stock_qty, 200.0);

        // 5. Mal Kabul Detayını Çek
        let detail = db
            .get_goods_receipt_detail(&receipt.id)
            .expect("Detay bulunmalı");
        assert_eq!(detail.items.len(), 1);
        assert_eq!(detail.items[0].quantity, 50.0);
        assert_eq!(detail.items[0].cost_price, 7.50);
    }

    #[test]
    fn test_campaign_discount_calculations_and_checkout() {
        let temp_file = NamedTempFile::new().expect("Temp file failed");
        let db = DatabaseManager::new(temp_file.path()).expect("DB init failed");

        // 1. Kampanyalar Tanımla
        // Yüzde 10 indirim
        let c_percent = db
            .create_campaign(CreateCampaignRequest {
                name: "Yaz İndirimi %10".to_string(),
                campaign_type: "PERCENT".to_string(),
                discount_percent: Some(10.0),
                discount_amount: None,
                min_cart_amount: Some(50.0),
                buy_qty: None,
                pay_qty: None,
                target_product_id: None,
                target_category: None,
            })
            .expect("Yüzde indirim oluşturulmalı");

        let campaigns = db.get_campaigns();
        assert_eq!(campaigns.len(), 1);
        assert_eq!(campaigns[0].id, c_percent.id);

        // 2. İndirim Hesaplama Testi (10 Adet Ekmek = 100 TL -> %10 İndirim = 10 TL)
        let bread = db.find_product_by_barcode("8690504018087").unwrap();
        let items = vec![CartItem {
            product_id: bread.id.clone(),
            barcode: bread.barcode.clone(),
            name: bread.name.clone(),
            unit_price: 10.0,
            quantity: 10.0, // 10 adet = 100 TL
            unit: "adet".to_string(),
            vat_rate: bread.vat_rate,
            discount: 0.0,
            total: 100.0,
            is_weighed: false,
            notes: None,
        }];

        let (disc, camp_name) = db.calculate_campaign_discount(100.0, &items);
        assert_eq!(disc, 10.0);
        assert_eq!(camp_name.as_deref(), Some("Yaz İndirimi %10"));

        // 3. Kampanyalı Satış İşleme
        let sale = db
            .process_checkout(CheckoutRequest {
                items,
                payment_method: "CASH".to_string(),
                customer_id: None,
                tender_amount: Some(100.0),
                discount_amount: Some(10.0),
                notes: Some("Kampanya uygulandı".to_string()),
                campaign_code: Some("YAZ10".to_string()),
                ..Default::default()
            })
            .expect("Satış yapılmalı");

        assert_eq!(sale.subtotal, 100.0);
        assert_eq!(sale.discount_amount, 10.0);
        assert_eq!(sale.grand_total, 90.0);
        assert_eq!(sale.change_amount, 10.0); // 100 - 90
    }

    #[test]
    fn test_inter_branch_stock_transfers() {
        let temp_file = NamedTempFile::new().expect("Temp file failed");
        let db = DatabaseManager::new(temp_file.path()).expect("DB init failed");

        // 1. Şubeleri Doğrula
        let branches = db.get_branches();
        assert!(branches.len() >= 2);
        let from_branch = &branches[0];
        let to_branch = &branches[1];

        // 2. İlk Şubeden İkinci Şubeye 25 Somun Ekmek Sevk Et
        let bread_before = db.find_product_by_barcode("8690504018087").unwrap();
        assert_eq!(bread_before.stock_qty, 150.0);

        let transfer = db
            .create_stock_transfer(CreateStockTransferRequest {
                from_branch_id: from_branch.id.clone(),
                from_branch_name: from_branch.name.clone(),
                to_branch_id: to_branch.id.clone(),
                to_branch_name: to_branch.name.clone(),
                notes: Some("Şube stok ikmali".to_string()),
                items: vec![StockTransferItem {
                    product_id: bread_before.id.clone(),
                    barcode: bread_before.barcode.clone(),
                    name: bread_before.name.clone(),
                    unit: "adet".to_string(),
                    quantity: 25.0,
                    received_quantity: 0.0,
                }],
            })
            .expect("Transfer başlatılmalı");

        assert_eq!(transfer.status, "DISPATCHED");

        // 3. Çıkış Şubesi Stoğundan Düşmeli (150 - 25 = 125)
        let bread_after_dispatch = db.find_product_by_barcode("8690504018087").unwrap();
        assert_eq!(bread_after_dispatch.stock_qty, 125.0);

        // 4. Karşı Şube Transferi Teslim Alır (Stok Yeniden Artar veya Şube Tarafına Girer)
        let received = db
            .receive_stock_transfer(&transfer.id)
            .expect("Transfer teslim alınmalı");
        assert_eq!(received.status, "RECEIVED");

        let transfers = db.get_stock_transfers();
        assert!(!transfers.is_empty());
        assert_eq!(transfers[0].status, "RECEIVED");
    }

    #[test]
    fn test_register_shift_lifecycle_and_x_report() {
        let temp_file = NamedTempFile::new().expect("Temp file failed");
        let db = DatabaseManager::new(temp_file.path()).expect("DB init failed");

        // 1. Başlangıçta aktif vardiya olmamalı
        assert!(db.get_active_shift().is_none());

        // 2. 500 TL Çekmece Açılış Devir Nakdi (Float) ile Vardiya Başlat
        let shift = db
            .open_shift(OpenShiftRequest {
                register_name: "KASA-01".to_string(),
                cashier_name: "Fatma Kasiyer".to_string(),
                opening_cash: 500.0,
            })
            .expect("Vardiya açılmalı");

        assert_eq!(shift.status, "OPEN");
        assert_eq!(shift.opening_cash, 500.0);

        // 3. Nakit Satış Yap (83 TL)
        let bread = db.find_product_by_barcode("8690504018087").unwrap();
        let _sale = db
            .process_checkout(CheckoutRequest {
                items: vec![CartItem {
                    product_id: bread.id.clone(),
                    barcode: bread.barcode.clone(),
                    name: bread.name.clone(),
                    unit_price: 10.0,
                    quantity: 8.3,
                    unit: "adet".to_string(),
                    vat_rate: bread.vat_rate,
                    discount: 0.0,
                    total: 83.0,
                    is_weighed: false,
                    notes: None,
                }],
                payment_method: "CASH".to_string(),
                tender_amount: Some(100.0),
                ..Default::default()
            })
            .expect("Satış yapılmalı");

        // 4. Kasa Avans Girişi (100 TL)
        db.record_cash_movement("IN", 100.0, "Bozuk para takviyesi")
            .expect("Kasa girişi işlenmeli");

        // 5. Gün İçi X-Raporu Al (Ara Kasa Dökümü)
        let x_report = db.get_x_report();
        assert_eq!(x_report.cash_total, 83.0);

        // 6. Vardiyayı Kapat (Kasiyer çekmecede 680 TL saydı -> 3 TL Kasa Açığı)
        // Açılış (500) + Nakit Satış (83) + Kasa Girişi (100) = 683 beklenen
        // Kasiyer 680 sayarsa fark = 680 - 683 = -3.0
        let closed = db
            .close_shift(CloseShiftRequest {
                counted_cash: 680.0,
                notes: Some("3 TL bozuk para yuvarlama açığı".to_string()),
            })
            .expect("Vardiya kapatılmalı");

        assert_eq!(closed.status, "CLOSED");
        assert_eq!(closed.closing_cash, Some(680.0));
        assert_eq!(closed.expected_cash, Some(683.0));
        assert_eq!(closed.cash_difference, Some(-3.0));

        // 7. Artık aktif vardiya olmamalı
        assert!(db.get_active_shift().is_none());
    }

    #[test]
    fn test_split_payments_checkout() {
        let temp_file = NamedTempFile::new().expect("Temp file failed");
        let db = DatabaseManager::new(temp_file.path()).expect("DB init failed");

        let bread = db.find_product_by_barcode("8690504018087").unwrap();
        let items = vec![CartItem {
            product_id: bread.id.clone(),
            barcode: bread.barcode.clone(),
            name: bread.name.clone(),
            unit_price: 10.0,
            quantity: 10.0, // Toplam 100 TL
            unit: "adet".to_string(),
            vat_rate: bread.vat_rate,
            discount: 0.0,
            total: 100.0,
            is_weighed: false,
            notes: None,
        }];

        // Parçalı Ödeme: 60 TL Nakit + 40 TL Kredi Kartı
        let req = CheckoutRequest {
            items,
            payment_method: "SPLIT".to_string(),
            split_payments: Some(vec![
                SplitPayment {
                    method: "CASH".to_string(),
                    amount: 60.0,
                },
                SplitPayment {
                    method: "CREDIT_CARD".to_string(),
                    amount: 40.0,
                },
            ]),
            tender_amount: Some(60.0),
            notes: Some("Parçalı Ödeme Testi".to_string()),
            ..Default::default()
        };

        let sale = db.process_checkout(req).expect("Parçalı satış işlenmeli");
        assert_eq!(sale.grand_total, 100.0);
        assert_eq!(sale.payment_method, "SPLIT");
    }

    #[test]
    fn test_national_barcode_catalog_local_cache() {
        let temp_file = NamedTempFile::new().expect("Temp file failed");
        let db = DatabaseManager::new(temp_file.path()).expect("DB init failed");

        // 1. Başlangıçta katalog önbelleğinde yok
        assert!(db.lookup_national_catalog("8690637000001").is_none());

        // 2. 4.13 Milyon Ulusal Katalogdan gelen ürünü önbelleğe kaydet
        let national_prod = NationalProductInfo {
            barcode: "8690637000001".to_string(),
            name: "Eti Karam %70 Kakaolu Bitter Çikolata (75g)".to_string(),
            brand: Some("Eti".to_string()),
            category: Some("Atıştırmalık / Çikolata".to_string()),
            vat_rate: Some(10.0),
            unit: "adet".to_string(),
            is_weighed: false,
            suggested_price: Some(32.50),
            plu_code: None,
            source: "NATIONAL_CATALOG_4.13M".to_string(),
        };

        db.save_national_catalog_item(&national_prod)
            .expect("Ulusal ürün önbelleğe alınmalı");

        // 3. Önbellekten sorgula
        let cached = db
            .lookup_national_catalog("8690637000001")
            .expect("Önbellekten gelmeli");
        assert_eq!(cached.name, "Eti Karam %70 Kakaolu Bitter Çikolata (75g)");
        assert_eq!(cached.brand.as_deref(), Some("Eti"));
        assert_eq!(cached.suggested_price, Some(32.50));
    }
}
