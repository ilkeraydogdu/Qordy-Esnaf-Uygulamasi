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

    /* Shelf Label Styles (Turkish Market Standard) */
    .shelf-label-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
      gap: 16px;
      padding: 16px;
      overflow-y: auto;
      flex: 1;
    }
    .shelf-label-card {
      background: #ffffff;
      color: #111827;
      border-radius: var(--radius-md);
      padding: 14px;
      display: flex;
      flex-direction: column;
      justify-content: space-between;
      border: 2px solid #e5e7eb;
      box-shadow: 0 4px 14px rgba(0,0,0,0.15);
      position: relative;
    }
    .shelf-card-header {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      margin-bottom: 6px;
    }
    .shelf-prod-name {
      font-size: 15px;
      font-weight: 800;
      color: #0f172a;
      line-height: 1.25;
    }
    .shelf-badge-origin {
      background: #dc2626;
      color: white;
      font-size: 10px;
      font-weight: 700;
      padding: 2px 6px;
      border-radius: 4px;
      white-space: nowrap;
    }
    .shelf-price-row {
      display: flex;
      align-items: baseline;
      justify-content: space-between;
      margin: 10px 0;
      padding-bottom: 8px;
      border-bottom: 2px dashed #cbd5e1;
    }
    .shelf-price-main {
      font-size: 36px;
      font-weight: 900;
      color: #b91c1c;
      letter-spacing: -1px;
    }
    .shelf-price-unit {
      font-size: 11px;
      color: #64748b;
      font-weight: 600;
    }
    .shelf-barcode-box {
      display: flex;
      flex-direction: column;
      align-items: center;
      background: #f8fafc;
      padding: 6px;
      border-radius: 6px;
    }
    .shelf-barcode-bars {
      font-family: monospace;
      letter-spacing: 3px;
      font-size: 20px;
      font-weight: 900;
      color: #0f172a;
    }
    .shelf-barcode-num {
      font-size: 11px;
      font-family: monospace;
      font-weight: 700;
      color: #334155;
    }

    /* Terminal Payment Pulse Animation */
    .pulse-terminal {
      width: 68px;
      height: 68px;
      border-radius: 50%;
      background: rgba(59, 130, 246, 0.2);
      border: 3px solid var(--apple-blue);
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 28px;
      animation: pulse-ring 1.5s infinite;
      margin: 0 auto;
    }
    @keyframes pulse-ring {
      0% { transform: scale(0.95); box-shadow: 0 0 0 0 rgba(59, 130, 246, 0.5); }
      70% { transform: scale(1.05); box-shadow: 0 0 0 16px rgba(59, 130, 246, 0); }
      100% { transform: scale(0.95); box-shadow: 0 0 0 0 rgba(59, 130, 246, 0); }
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
      <button class="nav-seg-btn active" onclick="switchMainScreen('pos')">🛒 Kasa</button>
      <button class="nav-seg-btn" onclick="switchMainScreen('transactions')">🧾 İşlemler & İade</button>
      <button class="nav-seg-btn" onclick="switchMainScreen('products')">📦 Ürünler</button>
      <button class="nav-seg-btn" onclick="switchMainScreen('stock')">📋 Stok Sayım</button>
      <button class="nav-seg-btn" onclick="switchMainScreen('customers')">📒 Veresiye Defteri</button>
      <button class="nav-seg-btn" onclick="switchMainScreen('wastes')">🗑️ Fire Takibi</button>
      <button class="nav-seg-btn" onclick="switchMainScreen('labels')">🏷️ Raf Etiketleri</button>
      <button class="nav-seg-btn" onclick="switchMainScreen('cash')">💵 Kasa Hareketleri</button>
      <button class="nav-seg-btn" onclick="switchMainScreen('reports')">📊 Z-Raporu</button>
      <button class="nav-seg-btn" onclick="openCustomerDisplayTab()">📺 2. Ekran</button>
    </nav>

    <!-- Right Controls -->
    <div class="header-actions">
      <div class="pill-badge" id="netStatusBadge" onclick="triggerSyncNow()">
        <div class="pulse-dot"></div>
        <span id="netStatusText">Çevrimdışı (Lokal)</span>
      </div>
      <button class="btn-top-action" onclick="openTerminalConfigModal()">
        <span>💳 POS Cihazı</span>
      </button>
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

    <!-- SCREEN 6: İŞLEM GEÇMİŞİ & İADE -->
    <div class="screen-view crud-layout" id="screen-transactions">
      <div class="crud-header">
        <div>
          <div class="crud-title">Satış İşlemleri & Kalem İade Merkezi</div>
          <div style="font-size:12px; color:var(--text-muted);">Geçmiş satış fişleri, tahsilatlar ve kalem bazlı parçalı iade yönetimi</div>
        </div>
        <div style="display:flex; gap:10px;">
          <input type="text" id="transactionsSearch" class="crud-search-bar" style="width:260px;" placeholder="Fiş No veya Tarih Ara..." oninput="filterTransactions()">
          <button class="btn-top-action" onclick="loadTransactionsList()">🔄 Yenile</button>
        </div>
      </div>
      <div class="crud-table-wrap">
        <table class="crud-table">
          <thead>
            <tr>
              <th>Fiş No</th>
              <th>Tarih / Saat</th>
              <th>Ödeme Türü</th>
              <th>Tutar (₺)</th>
              <th>Kalem Sayısı</th>
              <th>Durum</th>
              <th style="text-align:right;">İşlem</th>
            </tr>
          </thead>
          <tbody id="transactionsTableBody"></tbody>
        </table>
      </div>
    </div>

    <!-- SCREEN 7: FİRE / ZAYİ TAKİBİ -->
    <div class="screen-view crud-layout" id="screen-wastes">
      <div class="crud-header">
        <div>
          <div class="crud-title">Fire & Zayi Takip Merkezi</div>
          <div style="font-size:12px; color:var(--text-muted);">Bozulma, bayatlama, kırılma ve SKT nedeniyle stoktan otomatik düşülen ürünler</div>
        </div>
        <div style="display:flex; gap:10px;">
          <button class="btn-checkout btn-cash" style="padding:10px 18px; font-size:13px;" onclick="openNewWasteModal()">
            + Yeni Fire / Zayi Kaydet
          </button>
          <button class="btn-top-action" onclick="loadWasteList()">🔄 Yenile</button>
        </div>
      </div>
      <div class="crud-table-wrap">
        <table class="crud-table">
          <thead>
            <tr>
              <th>Tarih / Saat</th>
              <th>Ürün Adı</th>
              <th>Barkod</th>
              <th>Miktar</th>
              <th>Zarar / Maliyet (₺)</th>
              <th>Sebep</th>
            </tr>
          </thead>
          <tbody id="wasteTableBody"></tbody>
        </table>
      </div>
    </div>

    <!-- SCREEN 8: RAF ETİKETLERİ -->
    <div class="screen-view crud-layout" id="screen-labels">
      <div class="crud-header">
        <div>
          <div class="crud-title">Raf & Barkod Etiket Merkezi</div>
          <div style="font-size:12px; color:var(--text-muted);">Türk perakende standartlarında yerli üretim logolu raf etiketleri ve barkod basımı</div>
        </div>
        <div style="display:flex; gap:10px;">
          <button class="btn-top-action" onclick="window.print()">🖨️ Sayfayı Yazdır (Termal / A4)</button>
          <button class="btn-top-action" onclick="loadShelfLabels()">🔄 Yenile</button>
        </div>
      </div>
      <div class="shelf-label-grid" id="shelfLabelsGrid">
        <!-- Raf Etiketleri Dinamik Yüklenir -->
      </div>
    </div>

    <!-- SCREEN 9: KASA HAREKETLERİ (AVANS & MASRAF) -->
    <div class="screen-view crud-layout" id="screen-cash">
      <div class="crud-header">
        <div>
          <div class="crud-title">Kasa Finans & Nakit Hareketleri</div>
          <div style="font-size:12px; color:var(--text-muted);">Gün başı devir, bozuk para avans girişi ve dükkan masraf çıkışları</div>
        </div>
        <div style="display:flex; gap:10px;">
          <button class="btn-checkout btn-cash" style="padding:10px 16px; font-size:13px;" onclick="openCashMovementModal('IN')">
            + Kasa Avans Girişi
          </button>
          <button class="btn-checkout btn-danger" style="padding:10px 16px; font-size:13px;" onclick="openCashMovementModal('OUT')">
            - Masraf Çıkışı Yap
          </button>
          <button class="btn-top-action" onclick="loadCashMovements()">🔄 Yenile</button>
        </div>
      </div>
      <div class="crud-table-wrap">
        <table class="crud-table">
          <thead>
            <tr>
              <th>Tarih / Saat</th>
              <th>Hareket Türü</th>
              <th>Tutar (₺)</th>
              <th>Açıklama / Sebep</th>
              <th>Kasa / Kasiyer</th>
            </tr>
          </thead>
          <tbody id="cashMovementsTableBody"></tbody>
        </table>
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

  <!-- Kalem Bazlı İade Modalı -->
  <div class="modal-overlay" id="refundModal">
    <div class="modal-window" style="width:620px;">
      <div class="modal-heading">Kalem Bazlı Parçalı İade</div>
      <div style="background:#0b0f19; padding:12px 16px; border-radius:10px; display:flex; justify-content:space-between; align-items:center;">
        <div>
          <span style="font-size:12px; color:var(--text-muted);">Fiş No: </span>
          <span id="refundModalReceiptNo" style="font-weight:700; font-family:monospace; color:var(--apple-blue);"></span>
        </div>
        <div>
          <span style="font-size:12px; color:var(--text-muted);">Orijinal Tutar: </span>
          <span id="refundModalOriginalTotal" style="font-weight:700; color:var(--text-main);"></span>
        </div>
      </div>
      <div style="font-size:12px; font-weight:600; color:var(--text-muted);">İade Edilecek Kalemleri Seçin:</div>
      <div id="refundItemsList" style="max-height:220px; overflow-y:auto; display:flex; flex-direction:column; gap:8px;"></div>
      <div class="form-group">
        <label>İade Nedeni:</label>
        <select id="refundReasonSelect" class="form-input">
          <option value="Müşteri Vazgeçti">Müşteri Vazgeçti / İptal</option>
          <option value="Hatalı Ürün / Bozuk">Hatalı Ürün / Bozuk / Kusurlu</option>
          <option value="Yanlış Ürün Okutuldu">Yanlış Ürün / Barkod Hatası</option>
          <option value="Fiyat İtirazı">Fiyat Uyuşmazlığı</option>
          <option value="Diğer">Diğer</option>
        </select>
      </div>
      <div style="background:#0b0f19; padding:14px 16px; border-radius:10px; display:flex; justify-content:space-between; align-items:center;">
        <span style="font-size:14px; font-weight:700;">Toplam İade Tutarı:</span>
        <span id="refundTotalAmountText" style="font-size:24px; font-weight:900; color:var(--apple-green);">0.00 ₺</span>
      </div>
      <div style="display:flex; gap:10px;">
        <button class="btn-top-action" style="flex:1; justify-content:center;" onclick="closeModal('refundModal')">Vazgeç</button>
        <button class="btn-checkout btn-danger" style="flex:2;" onclick="confirmItemRefund()">İADEYİ ONAYLA & STOK İADE ET</button>
      </div>
    </div>
  </div>

  <!-- Yeni Fire / Zayi Kaydet Modalı -->
  <div class="modal-overlay" id="newWasteModal">
    <div class="modal-window">
      <div class="modal-heading">Yeni Fire / Zayi Kaydı</div>
      <div class="form-group">
        <label>Barkod veya Ürün:</label>
        <div style="display:flex; gap:8px;">
          <input type="text" id="wasteBarcode" class="form-input" style="flex:1;" placeholder="Barkod okutun veya yazın..." onchange="lookupWasteProduct()">
          <button class="btn-top-action" onclick="lookupWasteProduct()">Bul</button>
        </div>
        <div id="wasteProductInfo" style="font-size:12px; color:var(--apple-blue); font-weight:600; min-height:16px;"></div>
      </div>
      <div class="form-group">
        <label>Fire Miktarı (Adet veya KG):</label>
        <input type="number" id="wasteQuantity" class="form-input" step="0.01" min="0.01" placeholder="1.00">
      </div>
      <div class="form-group">
        <label>Fire / Zayi Nedeni:</label>
        <select id="wasteReason" class="form-input">
          <option value="BAYAT / SKT">Son Tüketim Tarihi Geçti (SKT / Bayat)</option>
          <option value="KIRILDI / HASAR">Kırıldı / Hasar Gördü / Ambalaj Yırtık</option>
          <option value="ÇÜRÜME / BOZULMA">Çürüme / Bozulma (Manav Reyonu)</option>
          <option value="DİĞER">Diğer Zayi</option>
        </select>
      </div>
      <div style="display:flex; gap:10px;">
        <button class="btn-top-action" style="flex:1; justify-content:center;" onclick="closeModal('newWasteModal')">İptal</button>
        <button class="btn-checkout btn-cash" style="flex:2;" onclick="saveWasteRecord()">KAYDET & STOKTAN DÜŞ</button>
      </div>
    </div>
  </div>

  <!-- Kasa Hareketi (Avans / Masraf) Modalı -->
  <div class="modal-overlay" id="cashMovementModal">
    <div class="modal-window">
      <div class="modal-heading" id="cashMovementHeading">Kasa Nakit Hareketi</div>
      <input type="hidden" id="cashMovementType" value="IN">
      <div class="form-group">
        <label>Tutar (₺):</label>
        <input type="number" id="cashMovementAmount" class="form-input" style="font-size:20px; font-weight:800;" step="0.5" placeholder="0.00">
      </div>
      <div class="form-group">
        <label>Açıklama / Sebep:</label>
        <input type="text" id="cashMovementReason" class="form-input" placeholder="Örn: Gün başı bozuk para avansı, dükkan temizlik alımı...">
      </div>
      <div style="display:flex; gap:10px;">
        <button class="btn-top-action" style="flex:1; justify-content:center;" onclick="closeModal('cashMovementModal')">İptal</button>
        <button class="btn-checkout btn-cash" style="flex:2;" id="btnSubmitCashMovement" onclick="submitCashMovement()">KAYDET</button>
      </div>
    </div>
  </div>

  <!-- Beko ÖKC & POS Cihazı Ayar Modalı -->
  <div class="modal-overlay" id="terminalConfigModal">
    <div class="modal-window">
      <div class="modal-heading">💳 POS Cihazı & ÖKC Donanım Ayarları</div>
      <div style="font-size:12px; color:var(--text-muted); line-height:1.4;">
        Masa başında veya fiziki cihaz henüz yokken <b>Sanal POS Simülatörü</b> ile birebir banka onayı test edebilir; dükkandaki canlı <b>Beko GMP3 ÖKC</b> cihazınızı yerel ağ üzerinden sisteme bozmadan entegre edebilirsiniz.
      </div>
      <div class="form-group">
        <label>Çalışma Modu:</label>
        <select id="terminalMode" class="form-input" onchange="updateTerminalFieldsVisibility()">
          <option value="MOCK">🧪 Sanal POS Simülatörü (Test & Geliştirme)</option>
          <option value="BEKO_TCP">🔌 Beko GMP3 ÖKC (Yerel Ağ / TCP-IP)</option>
        </select>
      </div>
      <div id="terminalNetworkFields" style="display:none; flex-direction:column; gap:12px;">
        <div class="form-row">
          <div class="form-group">
            <label>Cihaz IP Adresi:</label>
            <input type="text" id="terminalHost" class="form-input" placeholder="192.168.1.150">
          </div>
          <div class="form-group">
            <label>Port (GMP3):</label>
            <input type="number" id="terminalPort" class="form-input" placeholder="9000">
          </div>
        </div>
        <div class="form-group">
          <label>Zaman Aşımı (sn):</label>
          <input type="number" id="terminalTimeout" class="form-input" placeholder="45">
        </div>
      </div>
      <div id="terminalTestStatus" style="font-size:12px; font-weight:600; min-height:16px;"></div>
      <div style="display:flex; gap:10px;">
        <button class="btn-top-action" style="flex:1; justify-content:center;" onclick="testTerminalConnection()">🔌 Bağlantıyı Test Et</button>
        <button class="btn-checkout btn-card" style="flex:1.5;" onclick="saveTerminalConfig()">KAYDET & AKTİF ET</button>
      </div>
      <button class="btn-top-action" style="justify-content:center;" onclick="closeModal('terminalConfigModal')">Kapat</button>
    </div>
  </div>

  <!-- EFT-POS Kart Ödeme Modalı (Beko & Mock) -->
  <div class="modal-overlay" id="eftPosPayModal">
    <div class="modal-window" style="text-align:center;">
      <div class="modal-heading">EFT-POS Kart Tahsilatı</div>
      <div style="font-size:12px; color:var(--apple-blue); font-weight:600;" id="eftPosDeviceLabel">Sanal POS Simülatörü</div>
      <div style="background:#0b0f19; padding:20px; border-radius:14px; margin:8px 0;">
        <div style="font-size:12px; color:var(--text-muted);">Ödenecek Tutar</div>
        <div style="font-size:36px; font-weight:900; color:#38bdf8;" id="eftPosAmountText">0.00 ₺</div>
      </div>
      <div class="pulse-terminal" id="eftPosIcon">💳</div>
      <div style="font-size:14px; font-weight:700; color:var(--text-main);" id="eftPosStatusText">
        Kartınızı Okutunuz veya Şifrenizi Giriniz...
      </div>
      <div style="font-size:12px; color:var(--text-muted);" id="eftPosSubtext">
        Cihazdan provizyon yanıtı bekleniyor
      </div>
      <div id="eftPosDetails" style="display:none; background:#0b0f19; padding:12px; border-radius:10px; font-size:12px; text-align:left; font-family:monospace;">
      </div>
      <button class="btn-top-action" id="eftPosCancelBtn" style="justify-content:center; margin-top:8px;" onclick="closeModal('eftPosPayModal')">İşlemi İptal Et</button>
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

      const btns = Array.from(document.querySelectorAll('.nav-seg-btn'));
      const activeBtn = btns.find(b => b.getAttribute('onclick')?.includes(`'${screenId}'`));
      if (activeBtn) activeBtn.classList.add('active');

      if (screenId === 'pos') document.getElementById('posBarcodeField').focus();
      else if (screenId === 'transactions') loadTransactionsList();
      else if (screenId === 'products') loadProductsList();
      else if (screenId === 'stock') loadStockSessions();
      else if (screenId === 'customers') loadCustomersList();
      else if (screenId === 'wastes') loadWasteList();
      else if (screenId === 'labels') loadShelfLabels();
      else if (screenId === 'cash') loadCashMovements();
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

    async function checkoutCard() {
      if (cart.length === 0) { showToast('Sepetiniz boş!', true); return; }
      const total = cart.reduce((a, b) => a + b.total, 0);

      // EFT-POS Modalını hazırla
      document.getElementById('eftPosAmountText').innerText = `${total.toFixed(2)} ₺`;
      document.getElementById('eftPosStatusText').innerText = 'Kartınızı Okutunuz veya Şifrenizi Giriniz...';
      document.getElementById('eftPosStatusText').style.color = 'var(--text-main)';
      document.getElementById('eftPosSubtext').innerText = 'Cihazdan provizyon yanıtı bekleniyor';
      document.getElementById('eftPosIcon').className = 'pulse-terminal';
      document.getElementById('eftPosIcon').innerText = '💳';
      document.getElementById('eftPosDetails').style.display = 'none';
      document.getElementById('eftPosCancelBtn').style.display = 'block';

      try {
        const cfgRes = await fetch('/api/terminal/config');
        if (cfgRes.ok) {
          const cfg = await cfgRes.json();
          document.getElementById('eftPosDeviceLabel').innerText = cfg.mode === 'BEKO_TCP'
            ? `Beko GMP3 Terminali (${cfg.host}:${cfg.port})`
            : '🧪 Sanal POS Simülatörü (Test Modu)';
        }
      } catch (e) {}

      openModal('eftPosPayModal');

      // Ödeme İsteği Gönder
      try {
        const payRes = await fetch('/api/terminal/pay', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            amount: total,
            txn_type: 'SALE',
            order_ref: `POS-${Date.now()}`,
            print_slip: true
          })
        });

        if (payRes.ok) {
          const payData = await payRes.json();
          if (payData.is_approved) {
            document.getElementById('eftPosIcon').className = '';
            document.getElementById('eftPosIcon').innerText = '✅';
            document.getElementById('eftPosStatusText').innerText = 'ÖDEME ONAYLANDI!';
            document.getElementById('eftPosStatusText').style.color = 'var(--apple-green)';
            document.getElementById('eftPosSubtext').innerText = 'Banka provizyonu başarılı, fiş basılıyor...';
            document.getElementById('eftPosCancelBtn').style.display = 'none';

            const details = document.getElementById('eftPosDetails');
            details.style.display = 'block';
            details.innerHTML = `
              <div><b>Onay Kodu (Auth):</b> ${payData.auth_code}</div>
              <div><b>RRN Referans:</b> ${payData.rrn}</div>
              <div><b>Kart:</b> ${payData.card_masked} (${payData.card_brand})</div>
              <div><b>Terminal ID:</b> ${payData.terminal_id}</div>
            `;

            setTimeout(async () => {
              closeModal('eftPosPayModal');
              await executeCheckout('CREDIT_CARD', total, null);
            }, 1200);
            return;
          } else {
            throw new Error(payData.error_message || 'İşlem reddedildi');
          }
        } else {
          const errText = await payRes.text();
          throw new Error(errText || 'Terminal bağlantı hatası');
        }
      } catch (err) {
        document.getElementById('eftPosIcon').className = '';
        document.getElementById('eftPosIcon').innerText = '❌';
        document.getElementById('eftPosStatusText').innerText = 'İŞLEM BAŞARISIZ!';
        document.getElementById('eftPosStatusText').style.color = 'var(--apple-red)';
        document.getElementById('eftPosSubtext').innerText = err.message || 'Cihazdan provizyon alınamadı.';
      }
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
    // ==========================================================
    // 1. İŞLEMLER & PARÇALI İADE LOGIC
    // ==========================================================
    let salesHistory = [];
    let activeRefundSale = null;

    async function loadTransactionsList() {
      try {
        const res = await fetch('/api/sales/history');
        if (res.ok) {
          salesHistory = await res.json();
          renderTransactions(salesHistory);
        }
      } catch (e) {
        showToast('İşlemler listelenemedi', true);
      }
    }

    function renderTransactions(list) {
      const tbody = document.getElementById('transactionsTableBody');
      tbody.innerHTML = '';
      if (!list || list.length === 0) {
        tbody.innerHTML = `<tr><td colspan="7" style="text-align:center; padding:24px; color:var(--text-muted);">Kayıtlı satış işlemi bulunamadı.</td></tr>`;
        return;
      }

      list.forEach(s => {
        const tr = document.createElement('tr');
        const itemCount = s.items ? s.items.reduce((acc, i) => acc + i.quantity, 0) : 0;
        const methodBadge = s.payment_method === 'CASH'
          ? '<span style="color:var(--apple-green); font-weight:700;">Nakit</span>'
          : (s.payment_method === 'CREDIT_CARD'
             ? '<span style="color:var(--apple-blue); font-weight:700;">Kredi Kartı</span>'
             : '<span style="color:var(--apple-amber); font-weight:700;">Veresiye</span>');

        tr.innerHTML = `
          <td><span style="font-family:monospace; font-weight:700; color:#38bdf8;">${s.receipt_no}</span></td>
          <td style="color:var(--text-muted);">${new Date(s.created_at).toLocaleString('tr-TR')}</td>
          <td>${methodBadge}</td>
          <td><b>${s.grand_total.toFixed(2)} ₺</b></td>
          <td>${itemCount.toFixed(itemCount % 1 === 0 ? 0 : 2)} Kalem</td>
          <td><span class="pill-badge" style="display:inline-flex;">Tamamlandı</span></td>
          <td style="text-align:right;">
            <button class="btn-top-action" onclick="openRefundModal('${s.id}')">🔄 Kalem İade Et</button>
          </td>
        `;
        tbody.appendChild(tr);
      });
    }

    function filterTransactions() {
      const q = (document.getElementById('transactionsSearch').value || '').toLowerCase();
      const filtered = salesHistory.filter(s =>
        (s.receipt_no && s.receipt_no.toLowerCase().includes(q)) ||
        (s.created_at && s.created_at.toLowerCase().includes(q))
      );
      renderTransactions(filtered);
    }

    function openRefundModal(saleId) {
      const sale = salesHistory.find(s => s.id === saleId);
      if (!sale) return;
      activeRefundSale = sale;

      document.getElementById('refundModalReceiptNo').innerText = sale.receipt_no;
      document.getElementById('refundModalOriginalTotal').innerText = `${sale.grand_total.toFixed(2)} ₺`;

      const listDiv = document.getElementById('refundItemsList');
      listDiv.innerHTML = '';

      sale.items.forEach((item, idx) => {
        const itemRow = document.createElement('div');
        itemRow.style = 'background:var(--bg-card); padding:10px 14px; border-radius:8px; display:flex; justify-content:space-between; align-items:center;';
        itemRow.innerHTML = `
          <div style="display:flex; align-items:center; gap:10px;">
            <input type="checkbox" id="refundChk_${idx}" style="transform:scale(1.2); cursor:pointer;" onchange="calculateRefundTotal()">
            <div>
              <div style="font-weight:700; font-size:13px;">${item.name}</div>
              <div style="font-size:11px; color:var(--text-muted);">${item.unit_price.toFixed(2)} ₺ / ${item.unit} (Satılan: ${item.quantity})</div>
            </div>
          </div>
          <div style="display:flex; align-items:center; gap:8px;">
            <span style="font-size:12px; color:var(--text-muted);">İade Adedi:</span>
            <input type="number" id="refundQty_${idx}" class="form-input" style="width:75px; padding:4px 8px; text-align:center; font-weight:700;" value="${item.quantity}" min="0.01" max="${item.quantity}" step="${item.is_weighed ? '0.05' : '1'}" oninput="calculateRefundTotal()">
          </div>
        `;
        listDiv.appendChild(itemRow);
      });

      calculateRefundTotal();
      openModal('refundModal');
    }

    function calculateRefundTotal() {
      if (!activeRefundSale) return;
      let total = 0;
      activeRefundSale.items.forEach((item, idx) => {
        const chk = document.getElementById(`refundChk_${idx}`);
        const qtyInput = document.getElementById(`refundQty_${idx}`);
        if (chk && chk.checked && qtyInput) {
          const qty = parseFloat(qtyInput.value) || 0;
          total += qty * item.unit_price;
        }
      });
      document.getElementById('refundTotalAmountText').innerText = `${total.toFixed(2)} ₺`;
    }

    async function confirmItemRefund() {
      if (!activeRefundSale) return;
      const refundItems = [];
      let totalRefund = 0;

      activeRefundSale.items.forEach((item, idx) => {
        const chk = document.getElementById(`refundChk_${idx}`);
        const qtyInput = document.getElementById(`refundQty_${idx}`);
        if (chk && chk.checked && qtyInput) {
          const qty = parseFloat(qtyInput.value) || 0;
          if (qty > 0) {
            const itemTotal = qty * item.unit_price;
            totalRefund += itemTotal;
            refundItems.push({
              product_id: item.product_id,
              barcode: item.barcode,
              name: item.name,
              unit_price: item.unit_price,
              quantity: qty,
              unit: item.unit,
              total: itemTotal
            });
          }
        }
      });

      if (refundItems.length === 0) {
        showToast('Lütfen en az bir ürün seçiniz!', true);
        return;
      }

      const reason = document.getElementById('refundReasonSelect').value;

      try {
        const res = await fetch('/api/sales/refund', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            sale_id: activeRefundSale.id,
            items: refundItems,
            refund_amount: totalRefund,
            reason: reason
          })
        });

        if (res.ok) {
          const data = await res.json();
          showToast(`✅ İade tamamlandı! İade Tutarı: ${data.refunded_amount.toFixed(2)} ₺`);
          closeModal('refundModal');
          loadTransactionsList();
          loadProductsList();
          fetchStatus();
        } else {
          const err = await res.text();
          showToast(`İade başarısız: ${err}`, true);
        }
      } catch (e) {
        showToast('İade isteği gönderilemedi', true);
      }
    }

    // ==========================================================
    // 2. FİRE / ZAYİ TAKİBİ LOGIC
    // ==========================================================
    async function loadWasteList() {
      try {
        const res = await fetch('/api/waste');
        if (res.ok) {
          const list = await res.json();
          renderWasteTable(list);
        }
      } catch (e) {
        showToast('Fire listesi alınamadı', true);
      }
    }

    function renderWasteTable(list) {
      const tbody = document.getElementById('wasteTableBody');
      tbody.innerHTML = '';
      if (!list || list.length === 0) {
        tbody.innerHTML = `<tr><td colspan="6" style="text-align:center; padding:24px; color:var(--text-muted);">Kayıtlı fire/zayi bulunmamaktadır.</td></tr>`;
        return;
      }

      list.forEach(w => {
        const tr = document.createElement('tr');
        tr.innerHTML = `
          <td style="color:var(--text-muted);">${new Date(w.recorded_at).toLocaleString('tr-TR')}</td>
          <td><b>${w.product_name}</b></td>
          <td><span style="font-family:monospace; color:var(--apple-blue);">${w.barcode}</span></td>
          <td>${w.quantity} ${w.unit}</td>
          <td><span style="color:var(--apple-red); font-weight:700;">-${w.cost_loss.toFixed(2)} ₺</span></td>
          <td><span class="pill-badge" style="color:var(--apple-amber);">${w.reason}</span></td>
        `;
        tbody.appendChild(tr);
      });
    }

    function openNewWasteModal() {
      document.getElementById('wasteBarcode').value = '';
      document.getElementById('wasteQuantity').value = '1';
      document.getElementById('wasteProductInfo').innerText = '';
      openModal('newWasteModal');
    }

    async function lookupWasteProduct() {
      const code = document.getElementById('wasteBarcode').value.trim();
      if (!code) return;
      try {
        const res = await fetch('/api/barcode/scan', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ barcode: code })
        });
        if (res.ok) {
          const data = await res.json();
          if (data.found && data.product) {
            document.getElementById('wasteProductInfo').innerText = `✓ Bulundu: ${data.product.name} (Stok: ${data.product.stock_qty} ${data.product.unit})`;
          } else {
            document.getElementById('wasteProductInfo').innerText = '⚠️ Ürün bulunamadı';
          }
        }
      } catch (e) {}
    }

    async function saveWasteRecord() {
      const barcode = document.getElementById('wasteBarcode').value.trim();
      const qty = parseFloat(document.getElementById('wasteQuantity').value) || 0;
      const reason = document.getElementById('wasteReason').value;

      if (!barcode || qty <= 0) {
        showToast('Lütfen geçerli barkod ve miktar giriniz!', true);
        return;
      }

      try {
        const res = await fetch('/api/waste', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            barcode: barcode,
            quantity: qty,
            reason: reason
          })
        });

        if (res.ok) {
          const record = await res.json();
          showToast(`🗑️ Fire kaydedildi (${record.product_name}) ve stoktan düşüldü!`);
          closeModal('newWasteModal');
          loadWasteList();
          loadProductsList();
        } else {
          const err = await res.text();
          showToast(`Hata: ${err}`, true);
        }
      } catch (e) {
        showToast('Fire kaydı başarısız', true);
      }
    }

    // ==========================================================
    // 3. RAF ETİKETLERİ LOGIC
    // ==========================================================
    async function loadShelfLabels() {
      try {
        const res = await fetch('/api/labels');
        if (res.ok) {
          const labels = await res.json();
          renderShelfLabels(labels);
        }
      } catch (e) {
        showToast('Etiketler yüklenemedi', true);
      }
    }

    function renderShelfLabels(labels) {
      const grid = document.getElementById('shelfLabelsGrid');
      grid.innerHTML = '';
      if (!labels || labels.length === 0) {
        grid.innerHTML = '<div style="color:var(--text-muted); padding:32px;">Kayıtlı ürün etiketi bulunamadı.</div>';
        return;
      }

      labels.forEach(l => {
        const card = document.createElement('div');
        card.className = 'shelf-label-card';
        const whole = Math.floor(l.price);
        const kurus = Math.round((l.price - whole) * 100).toString().padStart(2, '0');

        card.innerHTML = `
          <div>
            <div class="shelf-card-header">
              <span class="shelf-badge-origin">🇹🇷 YERLİ ÜRETİM</span>
              <span style="font-size:10px; font-weight:700; color:#64748b;">KDV %${l.vat_rate.toFixed(0)} DAHİL</span>
            </div>
            <div class="shelf-prod-name">${l.name}</div>
            <div style="font-size:11px; color:#64748b; font-weight:600; margin-top:2px;">Kategori: ${l.category}</div>
          </div>

          <div class="shelf-price-row">
            <div>
              <span class="shelf-price-main">${whole}<span style="font-size:22px;">,${kurus}</span> ₺</span>
            </div>
            <div class="shelf-price-unit">Birim: ${l.unit}</div>
          </div>

          <div class="shelf-barcode-box">
            <div class="shelf-barcode-bars">|||||| | |||| | ||||||</div>
            <div class="shelf-barcode-num">${l.barcode}</div>
          </div>
        `;
        grid.appendChild(card);
      });
    }

    // ==========================================================
    // 4. KASA HAREKETLERİ (AVANS & MASRAF)
    // ==========================================================
    async function loadCashMovements() {
      try {
        const res = await fetch('/api/cash/movements');
        if (res.ok) {
          const movements = await res.json();
          renderCashMovements(movements);
        }
      } catch (e) {
        showToast('Kasa hareketleri alınamadı', true);
      }
    }

    function renderCashMovements(list) {
      const tbody = document.getElementById('cashMovementsTableBody');
      tbody.innerHTML = '';
      if (!list || list.length === 0) {
        tbody.innerHTML = `<tr><td colspan="5" style="text-align:center; padding:24px; color:var(--text-muted);">Henüz bir kasa hareketi kaydedilmedi.</td></tr>`;
        return;
      }

      list.forEach(m => {
        const tr = document.createElement('tr');
        const isEntry = m.movement_type === 'IN';
        tr.innerHTML = `
          <td style="color:var(--text-muted);">${new Date(m.created_at).toLocaleString('tr-TR')}</td>
          <td>
            <span class="pill-badge" style="color:${isEntry ? 'var(--apple-green)' : 'var(--apple-red)'};">
              ${isEntry ? '🟢 Kasa Giriş / Avans' : '🔴 Kasa Çıkış / Masraf'}
            </span>
          </td>
          <td><b>${isEntry ? '+' : '-'}${m.amount.toFixed(2)} ₺</b></td>
          <td>${m.reason}</td>
          <td><span style="font-size:12px; color:var(--text-muted);">${m.register_id} (${m.cashier_name})</span></td>
        `;
        tbody.appendChild(tr);
      });
    }

    function openCashMovementModal(type) {
      document.getElementById('cashMovementType').value = type;
      document.getElementById('cashMovementAmount').value = '';
      document.getElementById('cashMovementReason').value = '';
      const heading = document.getElementById('cashMovementHeading');
      const submitBtn = document.getElementById('btnSubmitCashMovement');

      if (type === 'IN') {
        heading.innerText = '🟢 Kasa Avansı & Nakit Girişi';
        submitBtn.innerText = 'AVANSI KAYDET';
        submitBtn.className = 'btn-checkout btn-cash';
      } else {
        heading.innerText = '🔴 Dükkan Masrafı & Nakit Çıkışı';
        submitBtn.innerText = 'MASRAFI KAYDET';
        submitBtn.className = 'btn-checkout btn-danger';
      }

      openModal('cashMovementModal');
    }

    async function submitCashMovement() {
      const type = document.getElementById('cashMovementType').value;
      const amount = parseFloat(document.getElementById('cashMovementAmount').value) || 0;
      const reason = document.getElementById('cashMovementReason').value.trim();

      if (amount <= 0 || !reason) {
        showToast('Lütfen geçerli tutar ve açıklama giriniz!', true);
        return;
      }

      try {
        const res = await fetch('/api/cash/movements', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            movement_type: type,
            amount: amount,
            reason: reason
          })
        });

        if (res.ok) {
          showToast(`💵 Kasa hareketi başarıyla işlendi (${amount.toFixed(2)} ₺)`);
          closeModal('cashMovementModal');
          loadCashMovements();
        } else {
          showToast('Kasa hareketi kaydedilemedi', true);
        }
      } catch (e) {
        showToast('İstek hatası', true);
      }
    }

    // ==========================================================
    // 5. BEKO ÖKC & POS CİHAZI AYARLARI
    // ==========================================================
    async function openTerminalConfigModal() {
      try {
        const res = await fetch('/api/terminal/config');
        if (res.ok) {
          const cfg = await res.json();
          document.getElementById('terminalMode').value = cfg.mode || 'MOCK';
          document.getElementById('terminalHost').value = cfg.host || '192.168.1.150';
          document.getElementById('terminalPort').value = cfg.port || 9000;
          document.getElementById('terminalTimeout').value = cfg.timeout_seconds || 45;
          updateTerminalFieldsVisibility();
        }
      } catch (e) {}

      document.getElementById('terminalTestStatus').innerText = '';
      openModal('terminalConfigModal');
    }

    function updateTerminalFieldsVisibility() {
      const mode = document.getElementById('terminalMode').value;
      const netFields = document.getElementById('terminalNetworkFields');
      netFields.style.display = mode === 'BEKO_TCP' ? 'flex' : 'none';
    }

    async function testTerminalConnection() {
      const statusDiv = document.getElementById('terminalTestStatus');
      statusDiv.innerHTML = '<span style="color:var(--apple-blue);">🔌 Cihaz bağlantısı sınanıyor...</span>';

      try {
        const res = await fetch('/api/terminal/test', { method: 'POST' });
        if (res.ok) {
          const data = await res.json();
          if (data.connected) {
            statusDiv.innerHTML = `<span style="color:var(--apple-green);">✅ BAŞARILI: ${data.mode} erişilebilir ve hazır.</span>`;
          } else {
            statusDiv.innerHTML = `<span style="color:var(--apple-red);">⚠️ BAĞLANILAMADI: ${data.mode} yanıt vermedi. IP/Portu kontrol ediniz.</span>`;
          }
        }
      } catch (e) {
        statusDiv.innerHTML = '<span style="color:var(--apple-red);">Bağlantı testi sırasında hata oluştu.</span>';
      }
    }

    async function saveTerminalConfig() {
      const mode = document.getElementById('terminalMode').value;
      const host = document.getElementById('terminalHost').value.trim() || '192.168.1.150';
      const port = parseInt(document.getElementById('terminalPort').value) || 9000;
      const timeout = parseInt(document.getElementById('terminalTimeout').value) || 45;

      try {
        const res = await fetch('/api/terminal/config', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            mode: mode,
            host: host,
            port: port,
            timeout_seconds: timeout
          })
        });

        if (res.ok) {
          showToast('✅ POS Cihazı yapılandırması güncellendi!');
          closeModal('terminalConfigModal');
        } else {
          showToast('Ayarlar kaydedilemedi', true);
        }
      } catch (e) {
        showToast('Ayarlar kaydedilemedi', true);
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
