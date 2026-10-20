# QORDY Esnaf POS (Rust Edition) - v3.0

> **Çevrimdışı Çalışabilen (Offline-First), Sıfır Veri Kayıplı, Kriptografik Güvenlikli ve Bulut Senkronizasyonlu Masaüstü Perakende Satış Sistemi**  
> Hem tek şubeli mahalle esnafı (Bakkal, Manav, Kasap, Tekel) hem de yüzlerce şubeli zincir marketler için özel olarak **Rust** ile geliştirilmiştir.

---

## 🌟 Temel Mimari ve Güvenlik Standartları

### 1. Kriptografik Kimlik Doğrulama & Oturum Güvenliği (Auth & Session)
- **Tuzlanmış SHA-256 Kriptografi:** Kasiyer PIN kodları ve yönetici şifreleri veritabanında asla düz metin (plain-text) tutulmaz; kullanıcı bazlı rastgele UUID tuzlama (salt) ile özetlenir.
- **Zamanlama Saldırılarına Karşı Koruma (Constant-Time Verification):** Yan kanal (side-channel) saldırılarını engellemek amacıyla bit düzeyinde sabit süreli karşılaştırma uygulanır.
- **Kaba Kuvvet (Brute-Force) & Hız Sınırlama (Rate Limiting):** Üst üste 5 hatalı giriş denemesinde hesap 30 saniye boyunca otomatik olarak dondurulur ve donanım kilitlenir.
- **Kalıcı Masaüstü Oturumu (Persistent Sessions):** SQLite ve şifrelenmiş yerel depolama entegrasyonuyla macOS ve Windows üzerinde elektrik/uygulama kapanmalarında dahi oturum 30 gün boyunca güvenle korunur.
- **Hızlı Kasa Kilitleme (Lock Screen - [F12]):** Kasiyer kasadan ayrıldığında tek tuşla veya boşta kalma süresinde ekran kilitlenir, yetkisiz satış ve tahsilat engellenir.

### 2. %100 Çevrimdışı Çalışma (Offline-First SQLite WAL Mode)
- İnternet kablosu çekilse veya elektrik/ağ kesintisi olsa bile satış yapmaya, barkod okutmaya, fiş yazdırmaya ve veresiye defterine borç kaydetmeye kesintisiz devam eder.
- SQLite Write-Ahead Logging (WAL) ve `PRAGMA synchronous = NORMAL` ile ani kapanmalarda dahi veri bütünlüğü ve sıfır veri kaybı garanti edilir.

### 3. Atomik Satış & Outbox Senkronizasyon Deseni (Atomic Outbox Pattern)
- Bir fiş kesildiğinde tek bir veritabanı transaction'ı içinde:
  - Ürün stokları anında düşülür.
  - Müşteri veresiye borcu güncellenir ve cari defter hareketi kaydedilir.
  - Satış başlığı ve kalemleri yerel veritabanına yazılır.
  - İşlem buluta iletilmek üzere benzersiz `idempotency_key` ile `outbox_sync` tablosuna eklenir (İki defa yazılma önlenir).

### 4. Otomatik Çevrimdışı/Çevrimiçi Algılama & Senkronizasyon
- İnternet geri geldiği salisede `SyncEngine` durumu algılar ve bekleyen tüm çevrimdışı fişleri sırayla güvenle `qordy.com` bulut sunucusuna yükler.
- Bulutta yapılan fiyat ve ürün değişiklikleri cihaza delta senkronizasyonu ile çekilir.

### 5. Türkiye Perakende & Terazi Barkod Standardı Desteği
- **Standart Barkodlar:** EAN-13, EAN-8, CODE-128.
- **Gramaj Kodlu Terazi Barkodları (`27` Prefix):** `27 IIIII WWWWW C` -> Gramaj otomatik kg'ye çevrilir ve tartım hesaplanır (Örn: Domates, Elma).
- **Fiyat Kodlu Terazi Barkodları (`28` Prefix):** `28 IIIII TTTTT C` -> Barkoda gömülü kuruş tutarı ayrıştırılır (Örn: Kasap / Şarküteri).

### 6. ESC/POS & EFT-POS Donanım Entegrasyonu
- Standart 80mm ve 58mm termal yazıcılarla doğrudan uyumlu, Türkçe karakter desteği.
- Nakit ödemelerde para çekmecesini otomatik tetikleme (Cash drawer kick).
- Gün Sonu Z-Raporu, Gün İçi X-Raporu ve Veresiye Tahsilat Makbuzu çıktısı.
- Beko, Hugin, Ingenico EFT-POS GMP3 TCP protokolü ve 2. Müşteri Gösterge Ekranı (CFD).

---

## 🚀 Derleme ve Test

```bash
# Tüm birim ve entegrasyon testlerini çalıştır (29/29)
cargo test

# Geliştirme modunda çalıştır
cargo run

# Optimize edilmiş binary üret
cargo build --release
```

---

## 🔒 Varsayılan Giriş Bilgileri (Geliştirme / İlk Kurulum)
- **Kasiyer:** Ahmet Kasiyer • PIN: `1234`
- **Yönetici:** Fatma Yönetici • PIN: `9999` • Şifre: `admin`
