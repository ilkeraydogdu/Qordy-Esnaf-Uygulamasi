pub struct ParsedBarcode {
    pub raw_barcode: String,
    pub lookup_code: String,       // Ürün veritabanında aranacak kod (Ör: 2700142 veya 8690504...)
    pub item_code: Option<String>, // Terazi ürün kodu (Ör: "00142")
    pub is_weighed: bool,
    pub weight_kg: Option<f64>,
    pub embedded_price: Option<f64>,
}

pub struct BarcodeParser;

impl BarcodeParser {
    /// Barkod dizesini analiz et ve terazi/gramaj bilgilerini ayrıştır
    pub fn parse(input: &str) -> ParsedBarcode {
        let clean = input.trim();

        // 1. Terazi Barkodu Kontrolü (13 Haneli ve 27 veya 28 ile başlayan)
        if clean.len() == 13 && (clean.starts_with("27") || clean.starts_with("28")) {
            let prefix = &clean[0..2];
            let item_code = clean[2..7].to_string(); // 5 haneli ürün kodu
            let value_part = &clean[7..12];         // 5 haneli gramaj veya tutar

            let parsed_num: f64 = value_part.parse().unwrap_or(0.0);

            if prefix == "27" {
                // 27: Ağırlık / Gramaj Kodlu Terazi Barkodu (WWWWW gram)
                // Örnek: 2700124014503 -> 01450 gram = 1.450 kg
                let weight_kg = parsed_num / 1000.0;
                return ParsedBarcode {
                    raw_barcode: clean.to_string(),
                    lookup_code: format!("27{}", item_code), // DB'de '2700124' veya '00124' olarak aranır
                    item_code: Some(item_code),
                    is_weighed: true,
                    weight_kg: Some(weight_kg),
                    embedded_price: None,
                };
            } else if prefix == "28" {
                // 28: Fiyat Kodlu Terazi Barkodu (TTTTT kuruş)
                // Örnek: 2800124085503 -> 8550 kuruş = 85.50 TL
                let price_tl = parsed_num / 100.0;
                return ParsedBarcode {
                    raw_barcode: clean.to_string(),
                    lookup_code: format!("28{}", item_code),
                    item_code: Some(item_code),
                    is_weighed: true,
                    weight_kg: None,
                    embedded_price: Some(price_tl),
                };
            }
        }

        // 2. Standart Perakende Barkodu (EAN-13, EAN-8, CODE-128 vb.)
        ParsedBarcode {
            raw_barcode: clean.to_string(),
            lookup_code: clean.to_string(),
            item_code: None,
            is_weighed: false,
            weight_kg: None,
            embedded_price: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_barcode() {
        let parsed = BarcodeParser::parse("8690504018087");
        assert_eq!(parsed.lookup_code, "8690504018087");
        assert!(!parsed.is_weighed);
        assert_eq!(parsed.weight_kg, None);
    }

    #[test]
    fn test_weighed_gram_barcode() {
        // 27 (prefix) + 00124 (domates) + 01450 (1.450 kg) + 3 (checksum)
        let parsed = BarcodeParser::parse("2700124014503");
        assert!(parsed.is_weighed);
        assert_eq!(parsed.item_code.as_deref(), Some("00124"));
        assert_eq!(parsed.lookup_code, "2700124");
        assert_eq!(parsed.weight_kg, Some(1.450));
    }

    #[test]
    fn test_price_embedded_barcode() {
        // 28 (prefix) + 00124 (kıyma) + 08550 (85.50 TL) + 3 (checksum)
        let parsed = BarcodeParser::parse("2800124085503");
        assert!(parsed.is_weighed);
        assert_eq!(parsed.embedded_price, Some(85.50));
    }
}
