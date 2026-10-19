use crate::models::{Customer, LedgerEntry, Sale, ZReport};
use chrono::Local;

/// ESC/POS Termal Yazıcı Komutları ve Türkçe Fiş / Rapor Üreticisi
pub struct EscPosBuilder {
    buffer: Vec<u8>,
}

impl EscPosBuilder {
    pub fn new() -> Self {
        let mut b = Self { buffer: Vec::with_capacity(4096) };
        b.init();
        b
    }

    /// Yazıcıyı Başlat (ESC @)
    pub fn init(&mut self) -> &mut Self {
        self.buffer.extend_from_slice(&[0x1B, 0x40]);
        self
    }

    /// Karakter Kod Tablosunu Ayarla (CP857 - Türkçe)
    #[allow(dead_code)]
    pub fn set_codepage_cp857(&mut self) -> &mut Self {
        // ESC t 18 (veya bazı modellerde 19 / 58)
        self.buffer.extend_from_slice(&[0x1B, 0x74, 18]);
        self
    }

    /// Hizalama (0: Sol, 1: Orta, 2: Sağ)
    pub fn align(&mut self, alignment: u8) -> &mut Self {
        self.buffer.extend_from_slice(&[0x1B, 0x61, alignment]);
        self
    }

    /// Kalın Yazı (Bold)
    pub fn bold(&mut self, enable: bool) -> &mut Self {
        self.buffer.extend_from_slice(&[0x1B, 0x45, if enable { 1 } else { 0 }]);
        self
    }

    /// Yazı Boyutu (0: Normal, 1: Çift Yükseklik, 2: Çift Genişlik, 3: İkisi Birden)
    pub fn size(&mut self, double_width: bool, double_height: bool) -> &mut Self {
        let mut n = 0u8;
        if double_height {
            n |= 0x01;
        }
        if double_width {
            n |= 0x10;
        }
        self.buffer.extend_from_slice(&[0x1D, 0x21, n]);
        self
    }

    /// Altı Çizili
    #[allow(dead_code)]
    pub fn underline(&mut self, enable: bool) -> &mut Self {
        self.buffer.extend_from_slice(&[0x1B, 0x2D, if enable { 1 } else { 0 }]);
        self
    }

    /// Düz Metin Ekle (Türkçe karakterleri güvenli transliterasyon ile çevirir)
    pub fn text(&mut self, s: &str) -> &mut Self {
        let safe_text = sanitize_turkish_text(s);
        self.buffer.extend_from_slice(safe_text.as_bytes());
        self
    }

    /// Satır Başı Ekle
    pub fn feed(&mut self, lines: u8) -> &mut Self {
        self.buffer.extend_from_slice(&[0x1B, 0x64, lines]);
        self
    }

    /// Ayırıcı Çizgi (42 karakter standart 80mm veya 32 karakter 58mm)
    pub fn divider(&mut self, char_count: usize) -> &mut Self {
        let line = "-".repeat(char_count);
        self.text(&format!("{}\n", line));
        self
    }

    #[allow(dead_code)]
    pub fn double_divider(&mut self, char_count: usize) -> &mut Self {
        let line = "=".repeat(char_count);
        self.text(&format!("{}\n", line));
        self
    }

    /// İki Sütunlu Satır (Sol metin + Sağ tutar)
    pub fn row(&mut self, left: &str, right: &str, max_width: usize) -> &mut Self {
        let left_safe = sanitize_turkish_text(left);
        let right_safe = sanitize_turkish_text(right);

        let space_count = if max_width > left_safe.len() + right_safe.len() {
            max_width - (left_safe.len() + right_safe.len())
        } else {
            1
        };

        let formatted = format!("{}{}{}\n", left_safe, " ".repeat(space_count), right_safe);
        self.buffer.extend_from_slice(formatted.as_bytes());
        self
    }

    /// Kağıt Kes (ESC/POS GS V 0)
    pub fn cut(&mut self) -> &mut Self {
        self.feed(4);
        self.buffer.extend_from_slice(&[0x1D, 0x56, 0x00]);
        self
    }

    /// Çekmeceyi Aç (Cash Drawer Kick: ESC p 0 25 250)
    pub fn open_drawer(&mut self) -> &mut Self {
        self.buffer.extend_from_slice(&[0x1B, 0x70, 0x00, 0x19, 0xFA]);
        self
    }

    /// Byte Verisini Al
    pub fn to_bytes(&self) -> Vec<u8> {
        self.buffer.clone()
    }
}

/// Türkçe karakterleri termal yazıcıların desteklediği temiz ASCII karakterlere dönüştürür.
/// Bu sayede bozuk karakter (? veya anlamsız simgeler) basılmasını önler.
pub fn sanitize_turkish_text(input: &str) -> String {
    input.chars().map(|c| match c {
        'ı' => 'i', 'İ' => 'I',
        'ş' => 's', 'Ş' => 'S',
        'ğ' => 'g', 'Ğ' => 'G',
        'ü' => 'u', 'Ü' => 'U',
        'ö' => 'o', 'Ö' => 'O',
        'ç' => 'c', 'Ç' => 'C',
        other => other,
    }).collect()
}

pub struct ReceiptPrinter;

#[allow(dead_code)]
impl ReceiptPrinter {
    /// Perakende Satış Fişi Oluştur (80mm / 42 Karakter)
    pub fn build_sale_receipt(sale: &Sale) -> Vec<u8> {
        let mut b = EscPosBuilder::new();
        let w = 42;

        // Başlık
        b.align(1)
            .bold(true)
            .size(true, true)
            .text("QORDY PERAKENDE\n")
            .size(false, false)
            .bold(false)
            .text(&format!("{}\n", sale.branch_name))
            .text("Tel: 0216 555 00 22 - Kadikoy / ISTANBUL\n")
            .text("BILGI FISIDIR - MALI DEGERI YOKTUR\n")
            .divider(w);

        // Fiş Detayları
        b.align(0)
            .row("FIS NO:", &sale.receipt_no, w)
            .row("TARIH:", &Local::now().format("%d.%m.%Y %H:%M").to_string(), w)
            .row("KASA / KASIYER:", &format!("{}/{}", sale.register_id, sale.cashier_name), w);

        if let Some(ref cname) = sale.customer_name {
            b.row("MUSTERI (CARI):", cname, w);
        }

        b.double_divider(w);

        // Kalemler
        b.bold(true)
            .row("URUN ADI", "TUTAR (TL)", w)
            .bold(false)
            .divider(w);

        for item in &sale.items {
            let qty_line = if item.is_weighed {
                format!("{:.3} kg x {:.2} TL", item.quantity, item.unit_price)
            } else {
                format!("{:.0} {} x {:.2} TL", item.quantity, item.unit, item.unit_price)
            };

            b.bold(true).text(&format!("{}\n", sanitize_turkish_text(&item.name)));
            b.bold(false).row(&format!("  {}", qty_line), &format!("{:.2}", item.total), w);
        }

        b.double_divider(w);

        // Toplamlar
        b.row("ARA TOPLAM:", &format!("{:.2} TL", sale.subtotal), w);
        if sale.discount_amount > 0.0 {
            b.row("INDIRIM:", &format!("-{:.2} TL", sale.discount_amount), w);
        }
        b.row("KDV TOPLAM:", &format!("{:.2} TL", sale.vat_amount), w);

        b.bold(true)
            .size(false, true)
            .row("GENEL TOPLAM:", &format!("{:.2} TL", sale.grand_total), w)
            .size(false, false)
            .bold(false);

        // Ödeme Yöntemi
        let odeme_adi = match sale.payment_method.as_str() {
            "CASH" => "NAKIT",
            "CREDIT_CARD" => "KREDI KARTI",
            "VERESIYE" => "VERESIYE (CARI BORC)",
            _ => "DIGER",
        };
        b.divider(w);
        b.row("ODEME TIPI:", odeme_adi, w);

        if sale.payment_method == "CASH" {
            b.row("ODENEN NAKIT:", &format!("{:.2} TL", sale.tender_amount), w);
            b.bold(true)
                .row("PARA USTU:", &format!("{:.2} TL", sale.change_amount), w)
                .bold(false);
        }

        // Alt Bilgi & Barkod
        b.divider(w)
            .align(1)
            .text("IYI GUNLER DILERIZ, YINE BEKLERIZ\n")
            .text("Bu fis yerel QORDY Offline POS tarafindan\n")
            .text("guvenle uretilmistir.\n")
            .feed(2);

        // Nakit ödemede çekmeceyi otomatik aç
        if sale.payment_method == "CASH" {
            b.open_drawer();
        }

        b.cut();
        b.to_bytes()
    }

    /// Veresiye Tahsilat Makbuzu
    #[allow(dead_code)]
    pub fn build_payment_receipt(entry: &LedgerEntry, customer: &Customer) -> Vec<u8> {
        let mut b = EscPosBuilder::new();
        let w = 42;

        b.align(1)
            .bold(true)
            .size(true, true)
            .text("TAHSILAT MAKBUZU\n")
            .size(false, false)
            .bold(false)
            .text("QORDY CARI DEFTAR YONETIMI\n")
            .divider(w);

        b.align(0)
            .row("ISLEM NO:", &entry.id[0..8], w)
            .row("TARIH:", &Local::now().format("%d.%m.%Y %H:%M").to_string(), w)
            .row("MUSTERI:", &customer.name, w)
            .row("TELEFON:", &customer.phone, w)
            .divider(w);

        let onceki_bakiye = customer.balance + entry.amount;
        b.row("ONCEKI BORC:", &format!("{:.2} TL", onceki_bakiye), w);
        b.bold(true)
            .size(false, true)
            .row("ODENEN TUTAR:", &format!("{:.2} TL", entry.amount), w)
            .size(false, false)
            .bold(false);
        b.divider(w);
        b.bold(true)
            .row("KALAN GUNCEL BORC:", &format!("{:.2} TL", entry.balance_after), w)
            .bold(false);

        if let Some(ref note) = entry.note {
            b.feed(1).text(&format!("Not: {}\n", sanitize_turkish_text(note)));
        }

        b.divider(w)
            .align(1)
            .text("Tahsilat yapilmistir.\nTesekkur Ederiz.\n")
            .feed(2)
            .open_drawer()
            .cut();
        b.to_bytes()
    }

    /// Gün Sonu Z Raporu
    pub fn build_z_report(report: &ZReport) -> Vec<u8> {
        let mut b = EscPosBuilder::new();
        let w = 42;

        b.align(1)
            .bold(true)
            .size(true, true)
            .text("GUN SONU Z RAPORU\n")
            .size(false, false)
            .bold(false)
            .text(&format!("{}\n", report.branch_name))
            .text(&format!("KASA: {} | KASIYER: {}\n", report.register_id, report.cashier_name))
            .text(&format!("TARIH: {}\n", report.date))
            .text(&format!("Z NO: {}\n", report.z_number))
            .double_divider(w);

        b.align(0)
            .bold(true)
            .text("SATIS VE FINANSAL OZET\n")
            .bold(false)
            .divider(w)
            .row("TOPLAM SATIS SAYISI:", &format!("{}", report.total_sales_count), w)
            .row("BRUT TOPLAM:", &format!("{:.2} TL", report.total_gross), w)
            .row("TOPLAM INDIRIM:", &format!("-{:.2} TL", report.total_discount), w)
            .bold(true)
            .row("NET CIRO:", &format!("{:.2} TL", report.total_net), w)
            .bold(false)
            .divider(w);

        b.bold(true)
            .text("ODEME TIPI DAGILIMI\n")
            .bold(false)
            .divider(w)
            .row("NAKIT TAHSILAT:", &format!("{:.2} TL", report.cash_total), w)
            .row("KREDI KARTI:", &format!("{:.2} TL", report.card_total), w)
            .row("VERESIYE SATIS:", &format!("{:.2} TL", report.veresiye_total), w)
            .divider(w);

        b.bold(true)
            .text("KDV DETAYI\n")
            .bold(false)
            .divider(w);

        for vat in &report.vat_breakdown {
            let left = format!("%{:.0} KDV Matrah / KDV", vat.rate);
            let right = format!("{:.2} / {:.2}", vat.taxable_amount, vat.vat_amount);
            b.row(&left, &right, w);
        }

        b.double_divider(w)
            .align(1)
            .text("MALi HAFIZA VE CIHAZ GUVENLI KAPATMA\n")
            .text("GUN SONU RAPORU BASARIYLA ALINDI\n")
            .feed(3)
            .cut();
        b.to_bytes()
    }
}
