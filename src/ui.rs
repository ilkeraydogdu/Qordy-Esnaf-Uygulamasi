pub fn get_pos_html() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="tr">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>QORDY Esnaf POS - Çevrimdışı & Online Perakende Satış</title>
  <style>
    :root {
      --bg-base: #0b0f19;
      --bg-sidebar: #111827;
      --bg-card: #1f2937;
      --bg-card-hover: #283548;
      --border: rgba(255, 255, 255, 0.08);
      --border-focus: #3b82f6;
      --text-main: #f9fafb;
      --text-muted: #9ca3af;
      --apple-blue: #3b82f6;
      --apple-blue-hover: #2563eb;
      --apple-green: #10b981;
      --apple-green-hover: #059669;
      --apple-amber: #f59e0b;
      --apple-red: #ef4444;
      --apple-purple: #8b5cf6;
      --radius-sm: 8px;
      --radius-md: 12px;
      --radius-lg: 16px;
      --font-stack: -apple-system, BlinkMacSystemFont, "SF Pro Display", "SF Pro Text", "Segoe UI", Roboto, sans-serif;
    }

    * { box-sizing: border-box; margin: 0; padding: 0; font-family: var(--font-stack); user-select: none; }
    body { background: var(--bg-base); color: var(--text-main); height: 100vh; overflow: hidden; display: flex; flex-direction: column; }

    /* Top Navigation Header (Apple Style) */
    header {
      background: var(--bg-sidebar);
      border-bottom: 1px solid var(--border);
      height: 56px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 16px;
      z-index: 50;
    }

    .brand-group { display: flex; align-items: center; gap: 12px; }
    .brand-logo {
      background: linear-gradient(135deg, var(--apple-blue), var(--apple-green));
      color: white;
      font-weight: 800;
      font-size: 14px;
      padding: 5px 12px;
      border-radius: var(--radius-sm);
      letter-spacing: 0.5px;
    }
    .store-details { display: flex; flex-direction: column; }
    .store-name { font-weight: 700; font-size: 14px; }
    .store-branch { font-size: 11px; color: var(--text-muted); }

    /* Center Nav Tabs */
    .nav-segmented {
      display: flex;
      background: rgba(0, 0, 0, 0.3);
      padding: 3px;
      border-radius: var(--radius-md);
      border: 1px solid var(--border);
      gap: 2px;
    }
    .nav-seg-btn {
      background: transparent;
      border: none;
      color: var(--text-muted);
      padding: 6px 14px;
      border-radius: var(--radius-sm);
      font-size: 13px;
      font-weight: 600;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 6px;
      transition: all 0.15s ease;
    }
    .nav-seg-btn:hover { color: var(--text-main); }
    .nav-seg-btn.active {
      background: var(--apple-blue);
      color: white;
      box-shadow: 0 2px 8px rgba(59, 130, 246, 0.35);
    }

    /* Header Right Actions */
    .header-actions { display: flex; align-items: center; gap: 10px; }
    .pill-badge {
      display: flex;
      align-items: center;
      gap: 6px;
      padding: 5px 12px;
      border-radius: 20px;
      font-size: 12px;
      font-weight: 600;
      background: rgba(16, 185, 129, 0.12);
      color: var(--apple-green);
      border: 1px solid rgba(16, 185, 129, 0.25);
      cursor: pointer;
    }
    .pill-badge.offline {
      background: rgba(239, 68, 68, 0.12);
      color: var(--apple-red);
      border-color: rgba(239, 68, 68, 0.25);
    }
    .pulse-dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: currentColor;
      box-shadow: 0 0 6px currentColor;
    }

    .btn-top-action {
      background: var(--bg-card);
      border: 1px solid var(--border);
      color: var(--text-main);
      padding: 6px 12px;
      border-radius: var(--radius-sm);
      font-size: 12px;
      font-weight: 600;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 6px;
    }
    .btn-top-action:hover { background: var(--bg-card-hover); border-color: rgba(255,255,255,0.15); }

    /* Main Screen Containers */
    main { flex: 1; height: calc(100vh - 56px); overflow: hidden; position: relative; }
    .screen-view {
      position: absolute;
      top: 0; left: 0; right: 0; bottom: 0;
      display: none;
      overflow: hidden;
    }
    .screen-view.active { display: flex; }

    /* ==========================================================
       SCREEN 1: POS KASA SATIŞ
       ========================================================== */
    .pos-layout {
      width: 100%;
      height: 100%;
      display: grid;
      grid-template-columns: 460px 1fr;
    }

    /* Cart Column */
    .cart-side {
      background: var(--bg-sidebar);
      border-right: 1px solid var(--border);
      display: flex;
      flex-direction: column;
      height: 100%;
    }
    .cart-topbar {
      padding: 10px 16px;
      background: rgba(0,0,0,0.2);
      border-bottom: 1px solid var(--border);
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    .cart-title { font-weight: 700; font-size: 14px; display: flex; align-items: center; gap: 8px; }
    .cart-actions { display: flex; gap: 6px; }

    .cart-items-scroll {
      flex: 1;
      overflow-y: auto;
      padding: 8px;
    }
    .cart-table { width: 100%; border-collapse: collapse; font-size: 13px; }
    .cart-table th {
      text-align: left;
      padding: 8px;
      color: var(--text-muted);
      border-bottom: 1px solid var(--border);
      font-weight: 600;
      font-size: 12px;
    }
    .cart-table td {
      padding: 8px;
      border-bottom: 1px solid rgba(255,255,255,0.04);
      vertical-align: middle;
    }
    .item-title { font-weight: 600; font-size: 13px; margin-bottom: 2px; }
    .item-meta { font-size: 11px; color: var(--text-muted); }
    .qty-box { display: flex; align-items: center; gap: 4px; }
    .qty-btn {
      width: 24px;
      height: 24px;
      background: var(--bg-card);
      border: 1px solid var(--border);
      color: white;
      border-radius: 4px;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      font-weight: 700;
    }
    .qty-btn:hover { background: var(--apple-blue); }

    .cart-footer-box {
      background: #0f172a;
      border-top: 1px solid var(--border);
      padding: 14px 16px;
      display: flex;
      flex-direction: column;
      gap: 10px;
    }
    .row-summary { display: flex; justify-content: space-between; font-size: 13px; color: var(--text-muted); }
    .row-grand {
      display: flex;
      justify-content: space-between;
      align-items: baseline;
      padding-top: 8px;
      border-top: 1px dashed var(--border);
    }
    .grand-label { font-size: 15px; font-weight: 700; color: white; }
    .grand-val { font-size: 28px; font-weight: 800; color: #38bdf8; }

    .pay-grid {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 8px;
    }
    .btn-checkout {
      padding: 12px;
      border-radius: var(--radius-sm);
      border: none;
      font-weight: 700;
      font-size: 14px;
      cursor: pointer;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      color: white;
      transition: transform 0.1s;
    }
    .btn-checkout span.sub { font-size: 10px; opacity: 0.8; font-weight: normal; margin-top: 2px; }
    .btn-cash { background: var(--apple-green); }
    .btn-cash:hover { background: var(--apple-green-hover); }
    .btn-card { background: var(--apple-blue); }
    .btn-card:hover { background: var(--apple-blue-hover); }
    .btn-veresiye { background: var(--apple-amber); grid-column: span 2; }
    .btn-veresiye:hover { background: #d97706; }

    /* Catalog & Quick Grid Side */
    .catalog-side {
      display: flex;
      flex-direction: column;
      height: 100%;
      background: var(--bg-base);
    }
    .search-strip {
      padding: 10px 16px;
      background: var(--bg-sidebar);
      border-bottom: 1px solid var(--border);
      display: flex;
      gap: 10px;
    }
    .barcode-input-field {
      flex: 1;
      padding: 12px 14px;
      background: var(--bg-card);
      border: 2px solid var(--border);
      border-radius: var(--radius-sm);
      color: white;
      font-size: 15px;
      font-weight: 600;
      outline: none;
      transition: border-color 0.2s;
    }
    .barcode-input-field:focus { border-color: var(--apple-blue); }

    .catalog-categories {
      display: flex;
      gap: 8px;
      padding: 10px 16px;
      background: var(--bg-sidebar);
      border-bottom: 1px solid var(--border);
      overflow-x: auto;
    }
    .cat-pill {
      background: var(--bg-card);
      border: 1px solid var(--border);
      color: var(--text-muted);
      padding: 6px 14px;
      border-radius: 20px;
      font-size: 12px;
      font-weight: 600;
      cursor: pointer;
      white-space: nowrap;
    }
    .cat-pill.active {
      background: rgba(59, 130, 246, 0.2);
      color: var(--apple-blue);
      border-color: var(--apple-blue);
    }

    .tiles-grid {
      flex: 1;
      overflow-y: auto;
      padding: 16px;
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
      gap: 12px;
      align-content: start;
    }
    .pos-tile {
      background: var(--bg-sidebar);
      border: 1px solid var(--border);
      border-radius: var(--radius-md);
      padding: 12px;
      display: flex;
      flex-direction: column;
      justify-content: space-between;
      height: 110px;
      cursor: pointer;
      transition: all 0.15s ease;
      position: relative;
    }
    .pos-tile:hover {
      background: var(--bg-card);
      transform: translateY(-2px);
      box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    }
    .pos-tile.weighed { border-left: 4px solid var(--apple-green); }
    .pos-tile.standard { border-left: 4px solid var(--apple-blue); }
    .tile-text { font-size: 13px; font-weight: 600; line-height: 1.3; }
    .tile-bottom { display: flex; justify-content: space-between; align-items: baseline; }
    .tile-val { font-size: 15px; font-weight: 800; color: #38bdf8; }
    .tile-unit-label { font-size: 11px; color: var(--text-muted); }

    /* ==========================================================
       SCREEN 2: ÜRÜN YÖNETİMİ & BARKODLU ÜRÜN EKLEME
       ========================================================== */
    .crud-layout {
      width: 100%;
      height: 100%;
      display: flex;
      flex-direction: column;
      padding: 20px;
      gap: 16px;
      background: var(--bg-base);
    }
    .crud-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    .crud-title { font-size: 20px; font-weight: 700; }
    .table-container {
      flex: 1;
      background: var(--bg-sidebar);
      border: 1px solid var(--border);
      border-radius: var(--radius-md);
      overflow-y: auto;
    }
    .data-table {
      width: 100%;
      border-collapse: collapse;
      font-size: 13px;
    }
    .data-table th {
      padding: 12px 14px;
      background: var(--bg-card);
      color: var(--text-muted);
      font-weight: 600;
      text-align: left;
      border-bottom: 1px solid var(--border);
      position: sticky;
      top: 0;
    }
    .data-table td {
      padding: 12px 14px;
      border-bottom: 1px solid rgba(255,255,255,0.04);
      vertical-align: middle;
    }
    .badge-unit {
      background: rgba(59, 130, 246, 0.15);
      color: var(--apple-blue);
      padding: 2px 8px;
      border-radius: 4px;
      font-size: 11px;
      font-weight: 600;
    }

    /* ==========================================================
       SCREEN 3: STOK SAYIM & ENVANTER DENETİMİ
       ========================================================== */
    .stock-layout {
      width: 100%;
      height: 100%;
      display: grid;
      grid-template-columns: 360px 1fr;
      background: var(--bg-base);
    }
    .stock-sidebar {
      background: var(--bg-sidebar);
      border-right: 1px solid var(--border);
      padding: 16px;
      display: flex;
      flex-direction: column;
      gap: 12px;
    }
    .stock-main {
      padding: 16px;
      display: flex;
      flex-direction: column;
      gap: 12px;
      overflow-y: auto;
    }
    .variance-negative { color: var(--apple-red); font-weight: 700; }
    .variance-positive { color: var(--apple-green); font-weight: 700; }
    .variance-zero { color: var(--text-muted); }

    /* ==========================================================
       MODAL SYSTEM (Apple Clean Style)
       ========================================================== */
    .modal-overlay {
      position: fixed;
      top: 0; left: 0; right: 0; bottom: 0;
      background: rgba(0, 0, 0, 0.75);
      backdrop-filter: blur(8px);
      display: none;
      align-items: center;
      justify-content: center;
      z-index: 1000;
    }
    .modal-window {
      background: var(--bg-sidebar);
      border: 1px solid var(--border);
      border-radius: var(--radius-lg);
      width: 520px;
      max-width: 90vw;
      padding: 24px;
      box-shadow: 0 24px 48px rgba(0,0,0,0.6);
      display: flex;
      flex-direction: column;
      gap: 16px;
    }
    .modal-heading { font-size: 18px; font-weight: 700; }
    .form-group { display: flex; flex-direction: column; gap: 6px; }
    .form-group label { font-size: 12px; color: var(--text-muted); font-weight: 600; }
    .form-input {
      padding: 10px 12px;
      background: var(--bg-card);
      border: 1px solid var(--border);
      border-radius: var(--radius-sm);
      color: white;
      font-size: 14px;
      outline: none;
    }
    .form-input:focus { border-color: var(--apple-blue); }
    .form-row { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }

    /* Toast Notification */
    .toast-pill {
      position: fixed;
      bottom: 24px;
      right: 24px;
      background: var(--bg-card);
      border: 1px solid var(--border);
      border-left: 4px solid var(--apple-green);
      color: white;
      padding: 12px 20px;
      border-radius: var(--radius-md);
      box-shadow: 0 8px 30px rgba(0,0,0,0.5);
      display: none;
      z-index: 2000;
      font-size: 13px;
      font-weight: 600;
    }
  </style>
</head>
<body>

  <!-- Top Apple Navigation Bar -->
  <header>
    <div class="brand-group">
      <div class="brand-logo">QORDY</div>
      <div class="store-details">
        <div class="store-name" id="hdrStoreName">QORDY Cadde Esnaf</div>
        <div class="store-branch" id="hdrBranch">Kadıköy Çarşı Şubesi • Kasa: KASA-01</div>
      </div>
    </div>

    <!-- Navigation Tabs -->
    <nav class="nav-segmented">
      <button class="nav-seg-btn active" onclick="switchMainScreen('pos')">🛒 Kasa (POS)</button>
      <button class="nav-seg-btn" onclick="openCustomerDisplayTab()">📺 2. Ekran</button>
      <button class="nav-seg-btn" onclick="switchMainScreen('products')">📦 Ürünler</button>
      <button class="nav-seg-btn" onclick="switchMainScreen('stock')">📋 Stok Sayım</button>
      <button class="nav-seg-btn" onclick="switchMainScreen('customers')">📒 Veresiye Defteri</button>
      <button class="nav-seg-btn" onclick="switchMainScreen('reports')">📊 Z-Raporu</button>
    </nav>

    <!-- Right Controls -->
    <div class="header-actions">
      <div class="pill-badge" id="netStatusBadge" onclick="triggerSyncNow()">
        <div class="pulse-dot"></div>
        <span id="netStatusText">Çevrimdışı (Lokal)</span>
      </div>
      <button class="btn-top-action" onclick="openParkedCartsModal()">
        <span>⏸️ Bekleyenler</span>
        <span id="parkedBadgeCount" style="color:var(--apple-amber); font-weight:700;">(0)</span>
      </button>
    </div>
  </header>

  <!-- Screens Container -->
  <main>
    <!-- SCREEN 1: POS KASA -->
    <div class="screen-view pos-layout active" id="screen-pos">
      <!-- Left: Cart -->
      <div class="cart-side">
        <div class="cart-topbar">
          <div class="cart-title">
            <span>Satış Sepeti</span>
            <span id="cartCountBadge" style="font-size:12px; color:var(--text-muted);">(0 Kalem)</span>
          </div>
          <div class="cart-actions">
            <button class="btn-top-action" onclick="parkCurrentCart()">Beklet (F7)</button>
            <button class="btn-top-action" onclick="clearCart()">Temizle (F6)</button>
          </div>
        </div>

        <div class="cart-items-scroll">
          <table class="cart-table">
            <thead>
              <tr>
                <th>Ürün</th>
                <th style="width: 100px;">Miktar</th>
                <th style="width: 75px; text-align:right;">Birim</th>
                <th style="width: 80px; text-align:right;">Tutar</th>
                <th style="width: 30px;"></th>
              </tr>
            </thead>
            <tbody id="cartTableBody"></tbody>
          </table>
          <div id="cartEmptyMsg" style="padding:40px 20px; text-align:center; color:var(--text-muted); font-size:13px;">
            🛒 Sepet boş.<br/>Barkod okutun veya sağdaki ürünlere tıklayın.
          </div>
        </div>

        <div class="cart-footer-box">
          <div class="row-summary">
            <span>Ara Toplam</span>
            <span id="posSubtotal">0.00 ₺</span>
          </div>
          <div class="row-summary">
            <span>KDV Toplam</span>
            <span id="posVat">0.00 ₺</span>
          </div>
          <div class="row-grand">
            <span class="grand-label">GENEL TOPLAM</span>
            <span class="grand-val" id="posGrandTotal">0.00 ₺</span>
          </div>

          <div class="pay-grid">
            <button class="btn-checkout btn-cash" onclick="openCashModal()">
              <span>NAKİT [F2]</span>
              <span class="sub">Para Üstü Hesapla</span>
            </button>
            <button class="btn-checkout btn-card" onclick="checkoutCard()">
              <span>KREDİ KARTI [F3]</span>
              <span class="sub">Temassız / Çipli</span>
            </button>
            <button class="btn-checkout btn-veresiye" onclick="openVeresiyeCustomerModal()">
              <span>VERESİYE / CARİ BORÇ [F4]</span>
              <span class="sub">Müşteri Hesabına Ekle</span>
            </button>
          </div>
        </div>
      </div>

      <!-- Right: Search & Quick Tiles -->
      <div class="catalog-side">
        <div class="search-strip">
          <input
            type="text"
            id="posBarcodeField"
            class="barcode-input-field"
            placeholder="Barkod okutun veya ürün adı arayın (Enter)..."
            autocomplete="off"
            autofocus
          />
        </div>

        <div class="catalog-categories" id="posCategoriesBar">
          <!-- Dinamik Kategoriler -->
        </div>

        <div class="tiles-grid" id="posTilesGrid">
          <!-- Hızlı Ürünler Grid -->
        </div>
      </div>
    </div>

    <!-- SCREEN 2: ÜRÜN YÖNETİMİ -->
    <div class="screen-view crud-layout" id="screen-products">
      <div class="crud-header">
        <div>
          <div class="crud-title">Ürün & Fiyat Kataloğu</div>
          <div style="font-size:12px; color:var(--text-muted);">Çevrimdışı SQLite veritabanındaki tüm barkodlu ürünler</div>
        </div>
        <button class="btn-checkout btn-cash" style="padding:10px 18px; font-size:13px;" onclick="openNewProductModal()">
          + Yeni Ürün Ekle
        </button>
      </div>

      <div class="table-container">
        <table class="data-table">
          <thead>
            <tr>
              <th>Barkod</th>
              <th>Ürün Adı</th>
              <th>Kategori</th>
              <th>Birim</th>
              <th>Fiyat</th>
              <th>KDV</th>
              <th>Mevcut Stok</th>
              <th>Tür</th>
              <th style="text-align:right;">İşlem</th>
            </tr>
          </thead>
          <tbody id="productsTableBody"></tbody>
        </table>
      </div>
    </div>

    <!-- SCREEN 3: STOK SAYIM & ENVANTER -->
    <div class="screen-view stock-layout" id="screen-stock">
      <div class="stock-sidebar">
        <div style="font-weight:700; font-size:15px;">Stok Sayım Oturumları</div>
        <button class="btn-top-action" style="justify-content:center; background:var(--apple-blue);" onclick="openNewStockSessionModal()">
          + Yeni Sayım Başlat
        </button>
        <div id="stockSessionsList" style="display:flex; flex-direction:column; gap:8px; overflow-y:auto; flex:1;">
          <!-- Sayım listesi -->
        </div>
      </div>

      <div class="stock-main" id="stockSessionDetail">
        <div style="text-align:center; padding:60px; color:var(--text-muted);">
          Lütfen soldan bir sayım oturumu seçin veya yeni sayım başlatın.
        </div>
      </div>
    </div>

    <!-- SCREEN 4: VERESİYE DEFTERİ -->
    <div class="screen-view crud-layout" id="screen-customers">
      <div class="crud-header">
        <div>
          <div class="crud-title">Veresiye Müşteri & Cari Defteri</div>
          <div style="font-size:12px; color:var(--text-muted);">Müşteri borç bakiyeleri, limitler ve tahsilat kayıtları</div>
        </div>
        <button class="btn-checkout btn-cash" style="padding:10px 18px; font-size:13px;" onclick="openNewCustomerModal()">
          + Yeni Cari Ekle
        </button>
      </div>

      <div class="table-container">
        <table class="data-table">
          <thead>
            <tr>
              <th>Müşteri Adı Soyadı</th>
              <th>Telefon</th>
              <th>Güncel Borç</th>
              <th>Kredi Limiti</th>
              <th>Not / Adres</th>
              <th style="text-align:right;">İşlem</th>
            </tr>
          </thead>
          <tbody id="customersTableBody"></tbody>
        </table>
      </div>
    </div>

    <!-- SCREEN 5: Z-RAPORU & KASA -->
    <div class="screen-view crud-layout" id="screen-reports">
      <div class="crud-header">
        <div>
          <div class="crud-title">Kasa Finans & Gün Sonu Z-Raporu</div>
          <div style="font-size:12px; color:var(--text-muted);">Mevcut vardiya cirosu, nakit/kart dağılımı ve mali özet</div>
        </div>
        <button class="btn-checkout btn-card" style="padding:10px 18px; font-size:13px;" onclick="fetchAndDisplayZReport()">
          📊 Z-Raporu Yazdır
        </button>
      </div>

      <div id="zReportDisplayArea" style="max-width:600px; background:var(--bg-sidebar); border:1px solid var(--border); border-radius:var(--radius-lg); padding:24px;">
        <!-- Z Raporu Özeti -->
      </div>
    </div>
  </main>

  <!-- ==========================================================
       MODALLER
       ========================================================== -->

  <!-- Nakit Ödeme Modalı -->
  <div class="modal-overlay" id="cashModal">
    <div class="modal-window">
      <div class="modal-heading">Nakit Ödeme & Para Üstü</div>
      <div style="background:#0b0f19; padding:16px; border-radius:12px; text-align:right;">
        <div style="font-size:12px; color:var(--text-muted);">Ödenecek Tutar</div>
        <div style="font-size:28px; font-weight:800; color:#38bdf8;" id="cashModalDue">0.00 ₺</div>
      </div>
      <div style="display:grid; grid-template-columns:repeat(4,1fr); gap:8px;">
        <button class="btn-top-action" style="justify-content:center;" onclick="setCashVal(50)">50 ₺</button>
        <button class="btn-top-action" style="justify-content:center;" onclick="setCashVal(100)">100 ₺</button>
        <button class="btn-top-action" style="justify-content:center;" onclick="setCashVal(200)">200 ₺</button>
        <button class="btn-top-action" style="justify-content:center;" onclick="setCashValExact()">Tam Para</button>
      </div>
      <div class="form-group">
        <label>Alınan Nakit:</label>
        <input type="number" id="cashGivenInput" class="form-input" style="font-size:18px; font-weight:700;" oninput="updateChangeCalculation()">
      </div>
      <div style="background:#0b0f19; padding:16px; border-radius:12px; text-align:right;">
        <div style="font-size:12px; color:var(--text-muted);">Para Üstü</div>
        <div style="font-size:32px; font-weight:800; color:var(--apple-green);" id="cashModalChange">0.00 ₺</div>
      </div>
      <div style="display:flex; gap:10px;">
        <button class="btn-top-action" style="flex:1; justify-content:center;" onclick="closeModal('cashModal')">İptal</button>
        <button class="btn-checkout btn-cash" style="flex:2;" onclick="finalizeCashSale()">SATIŞI ONAYLA & FİŞ BAS</button>
      </div>
    </div>
  </div>

  <!-- Veresiye Müşteri Seçim Modalı -->
  <div class="modal-overlay" id="veresiyeSelectModal">
    <div class="modal-window">
      <div class="modal-heading">Veresiye Borç Kaydet</div>
      <input type="text" id="veresiyeSearch" class="form-input" placeholder="Müşteri adı veya telefon ara..." oninput="filterVeresiyeList()">
      <div id="veresiyeModalList" style="max-height:260px; overflow-y:auto; display:flex; flex-direction:column; gap:8px;"></div>
      <button class="btn-top-action" style="justify-content:center;" onclick="closeModal('veresiyeSelectModal')">Kapat</button>
    </div>
  </div>

  <!-- Yeni Ürün Ekle Modalı -->
  <div class="modal-overlay" id="newProductModal">
    <div class="modal-window">
      <div class="modal-heading">Yeni Ürün Ekle</div>
      <div class="form-row">
        <div class="form-group">
          <label>Barkod:</label>
          <input type="text" id="npBarcode" class="form-input" placeholder="Barkod okutun veya girin">
        </div>
        <div class="form-group">
          <label>Kategori:</label>
          <input type="text" id="npCategory" class="form-input" placeholder="Örn: Temel Gıda">
        </div>
      </div>
      <div class="form-group">
        <label>Ürün Adı:</label>
        <input type="text" id="npName" class="form-input" placeholder="Örn: Köy Ekmeği">
      </div>
      <div class="form-row">
        <div class="form-group">
          <label>Satış Fiyatı (TL):</label>
          <input type="number" step="0.01" id="npPrice" class="form-input" placeholder="0.00">
        </div>
        <div class="form-group">
          <label>KDV Oranı (%):</label>
          <select id="npVat" class="form-input">
            <option value="1.0">%1 (Temel Gıda / Ekmek)</option>
            <option value="10.0" selected>%10 (Standart Gıda / Restoran)</option>
            <option value="20.0">%20 (Genel Ticari)</option>
          </select>
        </div>
      </div>
      <div class="form-row">
        <div class="form-group">
          <label>Başlangıç Stoğu:</label>
          <input type="number" step="0.1" id="npStock" class="form-input" value="100.0">
        </div>
        <div class="form-group">
          <label>Birim:</label>
          <select id="npUnit" class="form-input">
            <option value="adet">adet</option>
            <option value="kg">kg (Tartılı)</option>
            <option value="paket">paket</option>
            <option value="koli">koli</option>
          </select>
        </div>
      </div>
      <div style="display:flex; gap:10px; margin-top:10px;">
        <button class="btn-top-action" style="flex:1; justify-content:center;" onclick="closeModal('newProductModal')">Vazgeç</button>
        <button class="btn-checkout btn-cash" style="flex:2;" onclick="saveNewProduct()">Kaydet</button>
      </div>
    </div>
  </div>

  <!-- Yeni Cari (Müşteri) Ekle Modalı -->
  <div class="modal-overlay" id="newCustomerModal">
    <div class="modal-window">
      <div class="modal-heading">Yeni Cari Müşteri Kaydı</div>
      <div class="form-group">
        <label>Ad Soyad / Firma Ünvanı:</label>
        <input type="text" id="ncName" class="form-input" placeholder="Örn: Komşu Mehmet Bey">
      </div>
      <div class="form-row">
        <div class="form-group">
          <label>Telefon Numarası:</label>
          <input type="text" id="ncPhone" class="form-input" placeholder="0532 000 00 00">
        </div>
        <div class="form-group">
          <label>Veresiye Kredi Limiti (TL):</label>
          <input type="number" id="ncLimit" class="form-input" value="2500.0">
        </div>
      </div>
      <div class="form-group">
        <label>Adres / Açıklama Notu:</label>
        <input type="text" id="ncNote" class="form-input" placeholder="Daire no, cadde vb.">
      </div>
      <div style="display:flex; gap:10px; margin-top:10px;">
        <button class="btn-top-action" style="flex:1; justify-content:center;" onclick="closeModal('newCustomerModal')">Vazgeç</button>
        <button class="btn-checkout btn-cash" style="flex:2;" onclick="saveNewCustomer()">Kaydet</button>
      </div>
    </div>
  </div>

  <!-- Tahsilat (Borç Ödemesi) Modalı -->
  <div class="modal-overlay" id="paymentModal">
    <div class="modal-window">
      <div class="modal-heading" id="payModalTitle">Borç Tahsilatı</div>
      <div class="form-group">
        <label>Ödenen Tutar (TL):</label>
        <input type="number" step="0.01" id="payModalAmount" class="form-input" style="font-size:18px; font-weight:700;">
      </div>
      <div class="form-group">
        <label>Açıklama / Makbuz Notu:</label>
        <input type="text" id="payModalNote" class="form-input" value="Elden Nakit Tahsilat">
      </div>
      <div style="display:flex; gap:10px; margin-top:10px;">
        <button class="btn-top-action" style="flex:1; justify-content:center;" onclick="closeModal('paymentModal')">Vazgeç</button>
        <button class="btn-checkout btn-cash" style="flex:2;" onclick="submitCustomerPayment()">Tahsilatı Kaydet & Makbuz Bas</button>
      </div>
    </div>
  </div>

  <!-- Bekletilen Sepetler Modalı -->
  <div class="modal-overlay" id="parkedCartsModal">
    <div class="modal-window">
      <div class="modal-heading">Bekletilen Sepetler (Park)</div>
      <div id="parkedListContainer" style="max-height:280px; overflow-y:auto; display:flex; flex-direction:column; gap:8px;"></div>
      <button class="btn-top-action" style="justify-content:center;" onclick="closeModal('parkedCartsModal')">Kapat</button>
    </div>
  </div>

  <!-- Toast -->
  <div class="toast-pill" id="globalToast"></div>

  <!-- ==========================================================
       JAVASCRIPT LOGIC
       ========================================================== -->
  <script>
    // State
    let cart = [];
    let products = [];
    let customers = [];
    let activeStockSessionId = null;
    let selectedCustomerIdForPayment = null;
    let ws = null;

    // Başlatma
    window.addEventListener('DOMContentLoaded', () => {
      initWebSocket();
      fetchStatus();
      loadPosCatalog();
      loadProductsList();
      loadCustomersList();
      loadParkedCount();

      setInterval(fetchStatus, 3000);

      // Barkod Okuyucu Enter Dinleyici
      const barcodeInput = document.getElementById('posBarcodeField');
      barcodeInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
          e.preventDefault();
          handleBarcodeScan(barcodeInput.value);
          barcodeInput.value = '';
        }
      });

      // Kısayollar
      window.addEventListener('keydown', (e) => {
        if (e.key === 'F2') { e.preventDefault(); openCashModal(); }
        else if (e.key === 'F3') { e.preventDefault(); checkoutCard(); }
        else if (e.key === 'F4') { e.preventDefault(); openVeresiyeCustomerModal(); }
        else if (e.key === 'F6') { e.preventDefault(); clearCart(); }
        else if (e.key === 'F7') { e.preventDefault(); parkCurrentCart(); }
      });
    });

    // WebSocket (Zero Latency Broadcast to 2. Ekran / CFD)
    function initWebSocket() {
      const loc = window.location;
      const wsUri = (loc.protocol === 'https:' ? 'wss://' : 'ws://') + loc.host + '/ws/pos';
      ws = new WebSocket(wsUri);

      ws.onopen = () => console.log('✅ POS WebSocket bağlı');
      ws.onclose = () => setTimeout(initWebSocket, 2000);
      ws.onerror = (e) => console.warn('WS Hata', e);
    }

    function broadcastCfdUpdate(stateType = 'ACTIVE_SALE') {
      const subtotal = cart.reduce((acc, i) => acc + i.total, 0);
      const cfdData = {
        state: cart.length === 0 ? 'IDLE' : stateType,
        items: cart,
        subtotal: subtotal,
        discount: 0.0,
        grand_total: subtotal,
        tender: 0.0,
        change: 0.0,
        payment_method: null,
        customer_name: null,
        updated_at: new Date().toISOString()
      };

      fetch('/api/cfd/update', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(cfdData)
      }).catch(() => {});
    }

    // Ekranlar Arası Geçiş
    function switchMainScreen(screenId) {
      document.querySelectorAll('.screen-view').forEach(s => s.classList.remove('active'));
      document.querySelectorAll('.nav-seg-btn').forEach(b => b.classList.remove('active'));

      const target = document.getElementById(`screen-${screenId}`);
      if (target) target.classList.add('active');

      if (screenId === 'pos') document.getElementById('posBarcodeField').focus();
      else if (screenId === 'products') loadProductsList();
      else if (screenId === 'stock') loadStockSessions();
      else if (screenId === 'customers') loadCustomersList();
      else if (screenId === 'reports') fetchAndDisplayZReport();
    }

    function openCustomerDisplayTab() {
      window.open('/customer', '_blank', 'width=1024,height=768');
    }

    // Durum Kontrolü
    async function fetchStatus() {
      try {
        const res = await fetch('/api/status');
        if (res.ok) {
          const d = await res.json();
          document.getElementById('hdrStoreName').innerText = d.business_name;
          document.getElementById('hdrBranch').innerText = `${d.branch_name} • Kasa: ${d.register_id}`;

          const badge = document.getElementById('netStatusBadge');
          const txt = document.getElementById('netStatusText');
          if (d.is_online) {
            badge.className = 'pill-badge';
            txt.innerText = d.pending_sync_count > 0 ? `Çevrimiçi (${d.pending_sync_count} Bekleyen)` : 'Çevrimiçi (Bulut Aktif)';
          } else {
            badge.className = 'pill-badge offline';
            txt.innerText = d.pending_sync_count > 0 ? `Çevrimdışı (${d.pending_sync_count} Bekleyen)` : 'Çevrimdışı (Lokal Mod)';
          }
        }
      } catch (e) {}
    }

    async function triggerSyncNow() {
      showToast('Bulut eşitlemesi tetikleniyor...');
      await fetch('/api/sync/trigger', { method: 'POST' });
      setTimeout(fetchStatus, 1500);
    }

    // POS Sepet & Barkod
    async function loadPosCatalog() {
      try {
        const res = await fetch('/api/products/quick');
        if (res.ok) {
          const list = await res.json();
          renderPosTiles(list);
        }
      } catch (e) {}
    }

    function renderPosTiles(list) {
      const grid = document.getElementById('posTilesGrid');
      grid.innerHTML = '';
      list.forEach(p => {
        const tile = document.createElement('div');
        tile.className = `pos-tile ${p.is_weighed ? 'weighed' : 'standard'}`;
        tile.onclick = () => addProductToCart(p);
        tile.innerHTML = `
          <div class="tile-text">${p.name}</div>
          <div class="tile-bottom">
            <span class="tile-val">${p.price.toFixed(2)} ₺</span>
            <span class="tile-unit-label">${p.is_weighed ? 'kg başına' : p.unit}</span>
          </div>
        `;
        grid.appendChild(tile);
      });
    }

    async function handleBarcodeScan(barcode) {
      const clean = barcode.trim();
      if (!clean) return;

      try {
        const res = await fetch('/api/barcode/scan', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ barcode: clean })
        });
        if (res.ok) {
          const d = await res.json();
          if (d.found && d.suggested_item) {
            pushCartItem(d.suggested_item);
            showToast(`✅ ${d.suggested_item.name} eklendi`);
          } else {
            showToast(`❌ Barkod bulunamadı: ${clean}`, true);
          }
        }
      } catch (e) {}
    }

    function addProductToCart(p) {
      if (p.is_weighed) {
        const weightInput = prompt(`${p.name} tartım gramajını girin (Örn: 1.450 veya 1450g):`, "1.000");
        if (!weightInput) return;
        let qty = parseFloat(weightInput.replace(',', '.'));
        if (qty > 50) qty = qty / 1000;

        pushCartItem({
          product_id: p.id,
          barcode: p.barcode,
          name: p.name,
          unit_price: p.price,
          quantity: qty,
          unit: 'kg',
          vat_rate: p.vat_rate,
          discount: 0.0,
          total: qty * p.price,
          is_weighed: true,
          notes: null
        });
      } else {
        const existing = cart.find(i => i.product_id === p.id && !i.is_weighed);
        if (existing) {
          existing.quantity += 1;
          existing.total = existing.quantity * existing.unit_price;
          renderCart();
        } else {
          pushCartItem({
            product_id: p.id,
            barcode: p.barcode,
            name: p.name,
            unit_price: p.price,
            quantity: 1.0,
            unit: p.unit,
            vat_rate: p.vat_rate,
            discount: 0.0,
            total: p.price,
            is_weighed: false,
            notes: null
          });
        }
      }
    }

    function pushCartItem(item) {
      cart.push(item);
      renderCart();
    }

    function changeItemQty(idx, delta) {
      const item = cart[idx];
      if (!item) return;

      if (item.is_weighed) item.quantity += delta * 0.100;
      else item.quantity += delta;

      if (item.quantity <= 0.001) cart.splice(idx, 1);
      else item.total = item.quantity * item.unit_price;

      renderCart();
    }

    function removeCartItem(idx) {
      cart.splice(idx, 1);
      renderCart();
    }

    function clearCart() {
      cart = [];
      renderCart();
      broadcastCfdUpdate('IDLE');
      document.getElementById('posBarcodeField').focus();
    }

    function renderCart() {
      const tbody = document.getElementById('cartTableBody');
      const emptyMsg = document.getElementById('cartEmptyMsg');
      tbody.innerHTML = '';

      if (cart.length === 0) {
        emptyMsg.style.display = 'block';
        document.getElementById('cartCountBadge').innerText = '(0 Kalem)';
      } else {
        emptyMsg.style.display = 'none';
        document.getElementById('cartCountBadge').innerText = `(${cart.length} Kalem)`;
      }

      let subtotal = 0;
      let vatTotal = 0;

      cart.forEach((item, idx) => {
        subtotal += item.total;
        const itemVat = item.total - (item.total / (1.0 + (item.vat_rate / 100.0)));
        vatTotal += itemVat;

        const tr = document.createElement('tr');
        const qtyTxt = item.is_weighed ? `${item.quantity.toFixed(3)} kg` : `${item.quantity} ${item.unit}`;

        tr.innerHTML = `
          <td>
            <div class="item-title">${item.name}</div>
            <div class="item-meta">${item.barcode}</div>
          </td>
          <td>
            <div class="qty-box">
              <button class="qty-btn" onclick="changeItemQty(${idx}, -1)">-</button>
              <span style="min-width:40px; text-align:center; font-weight:600;">${qtyTxt}</span>
              <button class="qty-btn" onclick="changeItemQty(${idx}, 1)">+</button>
            </div>
          </td>
          <td style="text-align:right;">${item.unit_price.toFixed(2)}</td>
          <td style="text-align:right; font-weight:700; color:#38bdf8;">${item.total.toFixed(2)}</td>
          <td style="text-align:center;">
            <button class="qty-btn" style="background:transparent; border:none; color:#ef4444;" onclick="removeCartItem(${idx})">×</button>
          </td>
        `;
        tbody.appendChild(tr);
      });

      document.getElementById('posSubtotal').innerText = `${subtotal.toFixed(2)} ₺`;
      document.getElementById('posVat').innerText = `${vatTotal.toFixed(2)} ₺`;
      document.getElementById('posGrandTotal').innerText = `${subtotal.toFixed(2)} ₺`;

      broadcastCfdUpdate('ACTIVE_SALE');
    }

    // Ödeme & Çıkış
    function openCashModal() {
      if (cart.length === 0) { showToast('Sepetiniz boş!', true); return; }
      const total = cart.reduce((a, b) => a + b.total, 0);
      document.getElementById('cashModalDue').innerText = `${total.toFixed(2)} ₺`;
      document.getElementById('cashGivenInput').value = total.toFixed(2);
      updateChangeCalculation();
      openModal('cashModal');
    }

    function setCashVal(val) {
      document.getElementById('cashGivenInput').value = val;
      updateChangeCalculation();
    }

    function setCashValExact() {
      const total = cart.reduce((a, b) => a + b.total, 0);
      document.getElementById('cashGivenInput').value = total.toFixed(2);
      updateChangeCalculation();
    }

    function updateChangeCalculation() {
      const total = cart.reduce((a, b) => a + b.total, 0);
      const given = parseFloat(document.getElementById('cashGivenInput').value) || 0;
      const change = Math.max(0, given - total);
      document.getElementById('cashModalChange').innerText = `${change.toFixed(2)} ₺`;
    }

    function finalizeCashSale() {
      const total = cart.reduce((a, b) => a + b.total, 0);
      const given = parseFloat(document.getElementById('cashGivenInput').value) || total;
      closeModal('cashModal');
      executeCheckout('CASH', given, null);
    }

    function checkoutCard() {
      if (cart.length === 0) { showToast('Sepetiniz boş!', true); return; }
      const total = cart.reduce((a, b) => a + b.total, 0);
      executeCheckout('CREDIT_CARD', total, null);
    }

    async function executeCheckout(method, tender, customerId) {
      try {
        const payload = {
          items: cart,
          payment_method: method,
          tender_amount: tender,
          customer_id: customerId,
          discount_amount: 0.0,
          notes: null
        };

        const res = await fetch('/api/checkout', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload)
        });

        if (res.ok) {
          const resp = await res.json();
          showToast(`🎉 Satış tamamlandı! Fiş: ${resp.receipt_no}`);
          clearCart();
          fetchStatus();
        }
      } catch (e) {
        showToast('Satış işleminde hata!', true);
      }
    }

    // Veresiye Modalı
    function openVeresiyeCustomerModal() {
      if (cart.length === 0) { showToast('Sepetiniz boş!', true); return; }
      renderVeresiyeSelectionList(customers);
      openModal('veresiyeSelectModal');
    }

    function renderVeresiyeSelectionList(list) {
      const wrap = document.getElementById('veresiyeModalList');
      wrap.innerHTML = '';
      list.forEach(c => {
        const div = document.createElement('div');
        div.style = 'background:var(--bg-card); padding:10px 14px; border-radius:8px; display:flex; justify-content:space-between; align-items:center; cursor:pointer;';
        div.onclick = () => {
          closeModal('veresiyeSelectModal');
          const total = cart.reduce((a, b) => a + b.total, 0);
          executeCheckout('VERESIYE', total, c.id);
        };
        div.innerHTML = `
          <div>
            <div style="font-weight:700;">${c.name}</div>
            <div style="font-size:12px; color:var(--text-muted);">${c.phone}</div>
          </div>
          <div style="text-align:right;">
            <div style="font-weight:700; color:#f59e0b;">Borç: ${c.balance.toFixed(2)} ₺</div>
            <div style="font-size:11px; color:var(--text-muted);">Limit: ${c.credit_limit} ₺</div>
          </div>
        `;
        wrap.appendChild(div);
      });
    }

    function filterVeresiyeList() {
      const q = document.getElementById('veresiyeSearch').value.toLowerCase();
      const filtered = customers.filter(c => c.name.toLowerCase().includes(q) || c.phone.includes(q));
      renderVeresiyeSelectionList(filtered);
    }

    // Sepet Bekletme (Park)
    async function parkCurrentCart() {
      if (cart.length === 0) return;
      const note = prompt('Bu sepet için bir not / masa / müşteri adı girin:', 'Müşteri 1');
      if (!note) return;

      try {
        const res = await fetch('/api/cart/park', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ note, items: cart })
        });
        if (res.ok) {
          showToast('⏸️ Sepet bekletmeye alındı');
          clearCart();
          loadParkedCount();
        }
      } catch (e) {}
    }

    async function loadParkedCount() {
      try {
        const res = await fetch('/api/cart/parked');
        if (res.ok) {
          const list = await res.json();
          document.getElementById('parkedBadgeCount').innerText = `(${list.length})`;
        }
      } catch (e) {}
    }

    async function openParkedCartsModal() {
      try {
        const res = await fetch('/api/cart/parked');
        if (res.ok) {
          const list = await res.json();
          const wrap = document.getElementById('parkedListContainer');
          wrap.innerHTML = '';
          if (list.length === 0) {
            wrap.innerHTML = '<div style="color:var(--text-muted); text-align:center; padding:20px;">Bekletilen sepet yok.</div>';
          } else {
            list.forEach(p => {
              const div = document.createElement('div');
              div.style = 'background:var(--bg-card); padding:10px 14px; border-radius:8px; display:flex; justify-content:space-between; align-items:center;';
              div.innerHTML = `
                <div>
                  <div style="font-weight:700;">${p.note}</div>
                  <div style="font-size:12px; color:var(--text-muted);">${p.item_count} Kalem • ${new Date(p.parked_at).toLocaleTimeString()}</div>
                </div>
                <div style="display:flex; align-items:center; gap:8px;">
                  <span style="font-weight:700; color:#38bdf8;">${p.grand_total.toFixed(2)} ₺</span>
                  <button class="btn-top-action" style="background:var(--apple-blue);" onclick="restoreParkedCart('${p.id}')">Geri Yükle</button>
                  <button class="btn-top-action" style="color:var(--apple-red);" onclick="deleteParkedCart('${p.id}')">Sil</button>
                </div>
              `;
              wrap.appendChild(div);
            });
          }
          openModal('parkedCartsModal');
        }
      } catch (e) {}
    }

    async function restoreParkedCart(id) {
      const res = await fetch('/api/cart/parked');
      if (res.ok) {
        const list = await res.json();
        const p = list.find(x => x.id === id);
        if (p) {
          cart = p.items;
          renderCart();
          await fetch(`/api/cart/parked/${id}`, { method: 'DELETE' });
          closeModal('parkedCartsModal');
          loadParkedCount();
          showToast('✅ Sepet kasaya geri çağrıldı');
        }
      }
    }

    async function deleteParkedCart(id) {
      await fetch(`/api/cart/parked/${id}`, { method: 'DELETE' });
      openParkedCartsModal();
      loadParkedCount();
    }

    // Ürün Yönetimi (CRUD)
    async function loadProductsList() {
      try {
        const res = await fetch('/api/products');
        if (res.ok) {
          products = await res.json();
          renderProductsTable(products);
        }
      } catch (e) {}
    }

    function renderProductsTable(list) {
      const tbody = document.getElementById('productsTableBody');
      tbody.innerHTML = '';
      list.forEach(p => {
        const tr = document.createElement('tr');
        tr.innerHTML = `
          <td style="font-family:monospace; font-weight:600;">${p.barcode}</td>
          <td style="font-weight:600;">${p.name}</td>
          <td><span class="badge-unit">${p.category}</span></td>
          <td>${p.unit}</td>
          <td style="font-weight:700; color:#38bdf8;">${p.price.toFixed(2)} ₺</td>
          <td>%${p.vat_rate.toFixed(0)}</td>
          <td style="font-weight:600;">${p.stock_qty.toFixed(1)}</td>
          <td>${p.is_weighed ? 'Terazi / Tartılı' : 'Adetli'}</td>
          <td style="text-align:right;">
            <button class="btn-top-action" style="display:inline-flex; color:var(--apple-red);" onclick="deleteProduct('${p.id}')">Sil</button>
          </td>
        `;
        tbody.appendChild(tr);
      });
    }

    function openNewProductModal() { openModal('newProductModal'); }

    async function saveNewProduct() {
      const barcode = document.getElementById('npBarcode').value.trim();
      const name = document.getElementById('npName').value.trim();
      const category = document.getElementById('npCategory').value.trim() || 'Genel';
      const price = parseFloat(document.getElementById('npPrice').value) || 0;
      const vat_rate = parseFloat(document.getElementById('npVat').value) || 10.0;
      const stock_qty = parseFloat(document.getElementById('npStock').value) || 0;
      const unit = document.getElementById('npUnit').value;
      const is_weighed = unit === 'kg';

      if (!barcode || !name || price <= 0) {
        alert('Lütfen barkod, ürün adı ve geçerli bir fiyat girin.');
        return;
      }

      const res = await fetch('/api/products', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ barcode, name, category, price, vat_rate, stock_qty, unit, is_weighed, quick_order: null })
      });

      if (res.ok) {
        closeModal('newProductModal');
        showToast('✅ Ürün kataloğa eklendi');
        loadProductsList();
        loadPosCatalog();
      }
    }

    async function deleteProduct(id) {
      if (!confirm('Bu ürünü silmek istediğinize emin misiniz?')) return;
      await fetch(`/api/products/${id}`, { method: 'DELETE' });
      loadProductsList();
      loadPosCatalog();
    }

    // Cari / Müşteri Yönetimi
    async function loadCustomersList() {
      try {
        const res = await fetch('/api/customers');
        if (res.ok) {
          customers = await res.json();
          renderCustomersTable(customers);
        }
      } catch (e) {}
    }

    function renderCustomersTable(list) {
      const tbody = document.getElementById('customersTableBody');
      tbody.innerHTML = '';
      list.forEach(c => {
        const tr = document.createElement('tr');
        tr.innerHTML = `
          <td style="font-weight:700;">${c.name}</td>
          <td>${c.phone}</td>
          <td style="font-weight:700; color:var(--apple-amber);">${c.balance.toFixed(2)} ₺</td>
          <td>${c.credit_limit.toFixed(2)} ₺</td>
          <td style="color:var(--text-muted);">${c.note || '-'}</td>
          <td style="text-align:right;">
            <button class="btn-top-action" style="display:inline-flex; background:var(--apple-green);" onclick="openPaymentModal('${c.id}', '${c.name}')">Tahsilat Yap</button>
          </td>
        `;
        tbody.appendChild(tr);
      });
    }

    function openNewCustomerModal() { openModal('newCustomerModal'); }

    async function saveNewCustomer() {
      const name = document.getElementById('ncName').value.trim();
      const phone = document.getElementById('ncPhone').value.trim();
      const credit_limit = parseFloat(document.getElementById('ncLimit').value) || 2000.0;
      const note = document.getElementById('ncNote').value.trim();

      if (!name || !phone) {
        alert('Lütfen ad soyad ve telefon girin.');
        return;
      }

      const res = await fetch('/api/customers', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name, phone, credit_limit, note: note || null })
      });

      if (res.ok) {
        closeModal('newCustomerModal');
        showToast('✅ Cari müşteri kaydedildi');
        loadCustomersList();
      }
    }

    function openPaymentModal(cid, cname) {
      selectedCustomerIdForPayment = cid;
      document.getElementById('payModalTitle').innerText = `${cname} - Borç Tahsilatı`;
      document.getElementById('payModalAmount').value = '';
      openModal('paymentModal');
    }

    async function submitCustomerPayment() {
      const amount = parseFloat(document.getElementById('payModalAmount').value) || 0;
      const note = document.getElementById('payModalNote').value.trim();

      if (amount <= 0) { alert('Lütfen geçerli bir ödeme tutarı girin.'); return; }

      const res = await fetch('/api/customers/payment', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ customer_id: selectedCustomerIdForPayment, amount, note })
      });

      if (res.ok) {
        closeModal('paymentModal');
        showToast('✅ Tahsilat kaydedildi ve makbuz basıldı');
        loadCustomersList();
      }
    }

    // Stok Sayım
    async function loadStockSessions() {
      try {
        const res = await fetch('/api/stock/sessions');
        if (res.ok) {
          const list = await res.json();
          const wrap = document.getElementById('stockSessionsList');
          wrap.innerHTML = '';
          list.forEach(s => {
            const div = document.createElement('div');
            div.style = 'background:var(--bg-card); padding:10px 14px; border-radius:8px; cursor:pointer;';
            div.onclick = () => openStockSessionDetail(s.id, s.title, s.status);
            div.innerHTML = `
              <div style="font-weight:700;">${s.title}</div>
              <div style="font-size:11px; color:var(--text-muted);">${new Date(s.started_at).toLocaleDateString()} • Durum: ${s.status}</div>
            `;
            wrap.appendChild(div);
          });
        }
      } catch (e) {}
    }

    async function openNewStockSessionModal() {
      const title = prompt('Sayım Başlığı / Alanı:', `Genel Depo Sayımı - ${new Date().toLocaleDateString()}`);
      if (!title) return;

      const res = await fetch('/api/stock/sessions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title, count_type: 'FULL', notes: null })
      });

      if (res.ok) {
        const sess = await res.json();
        loadStockSessions();
        openStockSessionDetail(sess.id, sess.title, 'OPEN');
      }
    }

    async function openStockSessionDetail(id, title, status) {
      activeStockSessionId = id;
      const detailArea = document.getElementById('stockSessionDetail');
      detailArea.innerHTML = `
        <div style="display:flex; justify-content:space-between; align-items:center;">
          <div>
            <div style="font-size:18px; font-weight:700;">${title}</div>
            <div style="font-size:12px; color:var(--text-muted);">Durum: ${status}</div>
          </div>
          ${status === 'OPEN' ? `<button class="btn-checkout btn-cash" style="padding:8px 16px; font-size:13px;" onclick="commitStockSession('${id}')">✓ Sayımı Onayla & Stokları Eşitle</button>` : ''}
        </div>

        ${status === 'OPEN' ? `
        <div style="display:flex; gap:10px; margin-top:10px;">
          <input type="text" id="stockScanField" class="form-input" style="flex:1;" placeholder="Sayım için barkod okutun..." onkeydown="if(event.key==='Enter'){scanStockItem('${id}');}">
          <input type="number" id="stockQtyField" class="form-input" style="width:100px;" value="1" placeholder="Adet">
          <button class="btn-top-action" style="background:var(--apple-blue);" onclick="scanStockItem('${id}')">Ekle</button>
        </div>` : ''}

        <div class="table-container" style="margin-top:14px;">
          <table class="data-table">
            <thead>
              <tr>
                <th>Barkod</th>
                <th>Ürün</th>
                <th>Sistem Stoğu</th>
                <th>Sayılan Miktar</th>
                <th>Fark (Varyans)</th>
              </tr>
            </thead>
            <tbody id="stockItemsTableBody"></tbody>
          </table>
        </div>
      `;

      loadStockItemsList(id);
    }

    async function loadStockItemsList(id) {
      const res = await fetch(`/api/stock/sessions/${id}/items`);
      if (res.ok) {
        const items = await res.json();
        const tbody = document.getElementById('stockItemsTableBody');
        tbody.innerHTML = '';
        items.forEach(i => {
          const tr = document.createElement('tr');
          let varClass = 'variance-zero';
          if (i.variance_qty < 0) varClass = 'variance-negative';
          else if (i.variance_qty > 0) varClass = 'variance-positive';

          tr.innerHTML = `
            <td style="font-family:monospace;">${i.barcode}</td>
            <td style="font-weight:600;">${i.product_name}</td>
            <td>${i.system_qty} ${i.unit}</td>
            <td style="font-weight:700;">${i.counted_qty} ${i.unit}</td>
            <td class="${varClass}">${i.variance_qty > 0 ? '+' : ''}${i.variance_qty} ${i.unit}</td>
          `;
          tbody.appendChild(tr);
        });
      }
    }

    async function scanStockItem(sessionId) {
      const barcode = document.getElementById('stockScanField').value.trim();
      const qty = parseFloat(document.getElementById('stockQtyField').value) || 1.0;
      if (!barcode) return;

      const res = await fetch(`/api/stock/sessions/${sessionId}/scan`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ barcode, qty })
      });

      if (res.ok) {
        document.getElementById('stockScanField').value = '';
        document.getElementById('stockScanField').focus();
        loadStockItemsList(sessionId);
      } else {
        alert('Barkod bulunamadı!');
      }
    }

    async function commitStockSession(sessionId) {
      if (!confirm('Sayımı onaylamak ve ürün stoklarını bu miktarlarla güncellemek istediğinize emin misiniz?')) return;
      const res = await fetch(`/api/stock/sessions/${sessionId}/commit`, { method: 'POST' });
      if (res.ok) {
        showToast('✅ Stoklar başarıyla güncellendi');
        loadStockSessions();
      }
    }

    // Z-Raporu
    async function fetchAndDisplayZReport() {
      const res = await fetch('/api/reports/z');
      if (res.ok) {
        const z = await res.json();
        const area = document.getElementById('zReportDisplayArea');
        area.innerHTML = `
          <div style="font-size:18px; font-weight:800; margin-bottom:12px; color:var(--text-main);">GÜN SONU Z RAPORU</div>
          <div style="font-size:12px; color:var(--text-muted); margin-bottom:16px;">Tarih: ${z.date} • Z No: ${z.z_number}</div>

          <div style="display:flex; justify-content:space-between; padding:8px 0; border-bottom:1px solid var(--border);">
            <span>Toplam Fiş Adedi:</span>
            <span style="font-weight:700;">${z.total_sales_count} Adet</span>
          </div>
          <div style="display:flex; justify-content:space-between; padding:8px 0; border-bottom:1px solid var(--border);">
            <span>Nakit Tahsilat:</span>
            <span style="font-weight:700; color:var(--apple-green);">${z.cash_total.toFixed(2)} ₺</span>
          </div>
          <div style="display:flex; justify-content:space-between; padding:8px 0; border-bottom:1px solid var(--border);">
            <span>Kredi Kartı:</span>
            <span style="font-weight:700; color:var(--apple-blue);">${z.card_total.toFixed(2)} ₺</span>
          </div>
          <div style="display:flex; justify-content:space-between; padding:8px 0; border-bottom:1px solid var(--border);">
            <span>Veresiye Satış:</span>
            <span style="font-weight:700; color:var(--apple-amber);">${z.veresiye_total.toFixed(2)} ₺</span>
          </div>
          <div style="display:flex; justify-content:space-between; padding:12px 0; border-bottom:2px solid var(--border); font-size:16px;">
            <span style="font-weight:700;">NET CİRO:</span>
            <span style="font-weight:800; color:#38bdf8;">${z.total_net.toFixed(2)} ₺</span>
          </div>

          <div style="margin-top:16px;">
            <div style="font-weight:700; font-size:13px; margin-bottom:8px;">KDV Dökümü:</div>
            ${z.vat_breakdown.map(v => `
              <div style="display:flex; justify-content:space-between; font-size:12px; color:var(--text-muted); padding:4px 0;">
                <span>%${v.rate.toFixed(0)} KDV Matrah / Vergi:</span>
                <span>${v.taxable_amount.toFixed(2)} ₺ / ${v.vat_amount.toFixed(2)} ₺</span>
              </div>
            `).join('')}
          </div>
        `;
        showToast('📊 Z-Raporu hazırlandı ve yazıcıya yollandı');
      }
    }

    // Modal Helper'lar
    function openModal(id) { document.getElementById(id).style.display = 'flex'; }
    function closeModal(id) { document.getElementById(id).style.display = 'none'; }

    function showToast(msg, isError = false) {
      const t = document.getElementById('globalToast');
      t.innerText = msg;
      t.style.borderLeftColor = isError ? 'var(--apple-red)' : 'var(--apple-green)';
      t.style.display = 'block';
      setTimeout(() => { t.style.display = 'none'; }, 3000);
    }
  </script>
</body>
</html>"#
}

pub fn get_customer_display_html() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="tr">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Müşteri Ekranı (CFD) - QORDY</title>
  <style>
    :root {
      --bg: #090d16;
      --card: #131b2e;
      --border: rgba(255, 255, 255, 0.08);
      --text: #f8fafc;
      --muted: #94a3b8;
      --blue: #3b82f6;
      --green: #10b981;
      --font: -apple-system, BlinkMacSystemFont, "SF Pro Display", Roboto, sans-serif;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; font-family: var(--font); }
    body { background: var(--bg); color: var(--text); height: 100vh; overflow: hidden; display: flex; flex-direction: column; }

    header {
      background: var(--card);
      border-bottom: 1px solid var(--border);
      height: 64px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 24px;
    }
    .brand { font-size: 20px; font-weight: 800; background: linear-gradient(135deg, var(--blue), var(--green)); -webkit-background-clip: text; -webkit-text-fill-color: transparent; }

    main {
      flex: 1;
      display: grid;
      grid-template-columns: 1fr 420px;
      height: calc(100vh - 64px);
    }

    .items-panel {
      padding: 24px;
      overflow-y: auto;
      border-right: 1px solid var(--border);
    }
    .item-row {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 14px 0;
      border-bottom: 1px solid var(--border);
      animation: fadeIn 0.2s ease;
    }
    .item-name { font-size: 18px; font-weight: 700; margin-bottom: 4px; }
    .item-meta { font-size: 14px; color: var(--muted); }
    .item-price { font-size: 22px; font-weight: 800; color: #38bdf8; }

    .summary-panel {
      background: #0f172a;
      padding: 32px 24px;
      display: flex;
      flex-direction: column;
      justify-content: space-between;
    }
    .total-title { font-size: 16px; color: var(--muted); font-weight: 600; text-transform: uppercase; }
    .total-number { font-size: 54px; font-weight: 900; color: #38bdf8; margin: 12px 0; }

    .idle-view {
      position: absolute;
      top: 64px; left: 0; right: 0; bottom: 0;
      background: var(--bg);
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 16px;
      z-index: 10;
    }
    .idle-title { font-size: 36px; font-weight: 800; }
    .idle-sub { font-size: 18px; color: var(--muted); }

    .success-view {
      position: absolute;
      top: 64px; left: 0; right: 0; bottom: 0;
      background: #064e3b;
      display: none;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 16px;
      z-index: 20;
    }

    @keyframes fadeIn { from { opacity: 0; transform: translateY(6px); } to { opacity: 1; transform: translateY(0); } }
  </style>
</head>
<body>
  <header>
    <div class="brand">QORDY CADDE ESNAF</div>
    <div id="clockDisplay" style="font-size:16px; font-weight:600; color:var(--muted);">--:--:--</div>
  </header>

  <main>
    <div class="items-panel">
      <div id="itemsContainer"></div>
    </div>

    <div class="summary-panel">
      <div>
        <div class="total-title">Ödenecek Tutar</div>
        <div class="total-number" id="grandTotalDisplay">0.00 ₺</div>
        <div style="display:flex; justify-content:space-between; color:var(--muted); font-size:15px; margin-top:8px;">
          <span>Ara Toplam:</span>
          <span id="subtotalDisplay">0.00 ₺</span>
        </div>
      </div>

      <div style="background:var(--card); padding:16px; border-radius:12px; text-align:center; border:1px solid var(--border);">
        <div style="font-size:13px; color:var(--muted); margin-bottom:4px;">Temassız & QR ile Ödeme</div>
        <div style="font-weight:700; color:var(--green);">Hazır • Kartınızı Yaklaştırın</div>
      </div>
    </div>
  </main>

  <!-- Idle Screen -->
  <div class="idle-view" id="idleView">
    <div style="font-size:64px;">👋</div>
    <div class="idle-title">Hoş Geldiniz</div>
    <div class="idle-sub">QORDY Cadde Esnaf • Kadıköy Çarşı Şubesi</div>
  </div>

  <!-- Success Screen -->
  <div class="success-view" id="successView">
    <div style="font-size:72px;">✓</div>
    <div style="font-size:36px; font-weight:800;">Ödeme Başarıyla Alındı</div>
    <div style="font-size:22px; color:#a7f3d0;" id="successChangeText">Para Üstü: 0.00 ₺</div>
    <div style="font-size:16px; opacity:0.8;">Teşekkür eder, iyi günler dileriz!</div>
  </div>

  <script>
    setInterval(() => {
      document.getElementById('clockDisplay').innerText = new Date().toLocaleTimeString('tr-TR');
    }, 1000);

    function connectWs() {
      const loc = window.location;
      const uri = (loc.protocol === 'https:' ? 'wss://' : 'ws://') + loc.host + '/ws/pos';
      const ws = new WebSocket(uri);

      ws.onmessage = (e) => {
        try {
          const msg = JSON.parse(e.data);
          if (msg.type === 'CFD_UPDATE') {
            handleCfdState(msg.data);
          }
        } catch (err) {}
      };

      ws.onclose = () => setTimeout(connectWs, 1500);
    }

    function handleCfdState(data) {
      const idle = document.getElementById('idleView');
      const success = document.getElementById('successView');
      const itemsContainer = document.getElementById('itemsContainer');

      if (data.state === 'IDLE' || !data.items || data.items.length === 0) {
        idle.style.display = 'flex';
        success.style.display = 'none';
        itemsContainer.innerHTML = '';
        return;
      }

      if (data.state === 'PAYMENT_COMPLETED') {
        idle.style.display = 'none';
        success.style.display = 'flex';
        document.getElementById('successChangeText').innerText = data.change > 0 ? `Para Üstü: ${data.change.toFixed(2)} ₺` : 'Ödeme Tamamlandı';
        setTimeout(() => {
          success.style.display = 'none';
          idle.style.display = 'flex';
        }, 6000);
        return;
      }

      // Active Sale
      idle.style.display = 'none';
      success.style.display = 'none';

      itemsContainer.innerHTML = '';
      data.items.forEach(item => {
        const div = document.createElement('div');
        div.className = 'item-row';
        div.innerHTML = `
          <div>
            <div class="item-name">${item.name}</div>
            <div class="item-meta">${item.is_weighed ? `${item.quantity.toFixed(3)} kg` : `${item.quantity} ${item.unit}`} x ${item.unit_price.toFixed(2)} ₺</div>
          </div>
          <div class="item-price">${item.total.toFixed(2)} ₺</div>
        `;
        itemsContainer.appendChild(div);
      });

      document.getElementById('grandTotalDisplay').innerText = `${data.grand_total.toFixed(2)} ₺`;
      document.getElementById('subtotalDisplay').innerText = `${data.subtotal.toFixed(2)} ₺`;
    }

    connectWs();
  </script>
</body>
</html>"#
}
