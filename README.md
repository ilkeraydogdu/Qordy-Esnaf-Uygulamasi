# QORDY Esnaf POS (Rust Edition) - v3.0

> **Çevrimdışı Çalışabilen (Offline-First), Sıfır Veri Kayıplı ve Bulut Senkronizasyonlu Masaüstü Perakende Satış Sistemi**  
> Hem tek şubeli mahalle esnafı (Bakkal, Manav, Kasap, Tekel) hem de yüzlerce şubeli zincir marketler (BİM, A101 tarzı) için özel olarak Rust ile sıfırdan geliştirilmiştir.

---

## 🌟 Temel Mimari ve Güvenlik Özellikleri

1. **%100 Çevrimdışı Çalışma (Offline-First SQLite WAL Mode):**
   - İnternet kablosu çekilse veya elektrik/ağ kesintisi olsa bile satış yapmaya, barkod okutmaya, fiş yazdırmaya ve veresiye defterine borç kaydetmeye devam eder.
   - SQLite Write-Ahead Logging (WAL) ve `PRAGMA synchronous = NORMAL` ile ani kapanmalarda dahi veri bütünlüğü ve sıfır veri kaybı garanti edilir.

2. **Atomik Satış & Outbox Senkronizasyon Deseni (Atomic Outbox Pattern):**
   - Bir fiş kesildiğinde tek bir veritabanı transaction'ı içinde:
     - Ürün stokları anında düşülür.
     - Müşteri veresiye borcu güncellenir ve cari defter hareketi kaydedilir.
     - Satış başlığı ve kalemleri yerel veritabanına yazılır.
     - İşlem buluta iletilmek üzere benzersiz `idempotency_key` ile `outbox_sync` tablosuna eklenir.
   - Bu sayede aynı işlem iki defa yazılamaz (Idempotency garantisi).

3. **Otomatik Çevrimdışı/Çevrimiçi Algılama & Senkronizasyon:**
   - İnternet geri geldiği salisede `SyncEngine` watchdog'ı durumu algılar ve bekleyen tüm çevrimdışı fişleri sırayla güvenle buluta yükler.
   - Bulutta yapılan fiyat değişiklikleri periyodik olarak cihaza çekilir (Delta Catalog Pull).

4. **Türkiye Perakende & Terazi Barkod Standardı Desteği:**
   - **Standart Barkodlar:** EAN-13, EAN-8, CODE-128.
   - **Gramaj Kodlu Terazi Barkodları (`27` Prefix):** `27 IIIII WWWWW C` -> Gramaj otomatik kg'ye çevrilir ve tartım hesaplanır (Örn: Domates, Elma).
   - **Fiyat Kodlu Terazi Barkodları (`28` Prefix):** `28 IIIII TTTTT C` -> Barkoda gömülü kuruş tutarı ayrıştırılır (Örn: Kasap / Şarküteri).

5. **ESC/POS Termal Fiş & Raporlama:**
   - Standart 80mm ve 58mm termal yazıcılarla doğrudan uyumlu.
   - Türkçe karakter transliterasyonu ile bozuk harf basımını engeller.
   - Nakit ödemelerde para çekmecesini otomatik tetikler (Cash drawer kick).
   - Gün Sonu Z-Raporu ve Veresiye Tahsilat Makbuzu çıktısı.

6. **Çapraz Platform:**
   - macOS (Apple Silicon & Intel) ve Windows (10/11) üzerinde yerel pencere (Tao + Wry) ile minimum RAM tüketimi (< 40MB).

---

## 🚀 Derleme ve Test

```bash
# Testleri çalıştır
cargo test

# Geliştirme modunda çalıştır
cargo run

# Optimize edilmiş binary üret
cargo build --release
```
