pub fn get_pos_html() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="tr">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>QORDY Esnaf POS - Çevrimdışı & Online Perakende Satış</title>
  <style>
    :root {
      --bg-primary: #0f172a;
      --bg-secondary: #1e293b;
      --bg-card: #243247;
      --border-color: #334155;
      --text-main: #f8fafc;
      --text-muted: #94a3b8;
      --accent-green: #10b981;
      --accent-green-hover: #059669;
      --accent-blue: #3b82f6;
      --accent-blue-hover: #2563eb;
      --accent-amber: #f59e0b;
      --accent-red: #ef4444;
      --font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    }

    * { box-sizing: border-box; margin: 0; padding: 0; font-family: var(--font-family); user-select: none; }
    body { background: var(--bg-primary); color: var(--text-main); height: 100vh; overflow: hidden; display: flex; flex-direction: column; }

    /* Header Bar */
    header {
      background: var(--bg-secondary);
      border-bottom: 1px solid var(--border-color);
      height: 58px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 16px;
    }

    .brand-section { display: flex; align-items: center; gap: 12px; }
    .logo-badge {
      background: linear-gradient(135deg, #3b82f6, #10b981);
      color: white;
      font-weight: 900;
      font-size: 15px;
      padding: 6px 12px;
      border-radius: 8px;
      letter-spacing: 1px;
    }
    .store-info { display: flex; flex-direction: column; }
    .store-name { font-weight: 700; font-size: 15px; }
    .store-sub { font-size: 12px; color: var(--text-muted); }

    .header-center { display: flex; align-items: center; gap: 16px; font-size: 13px; color: var(--text-muted); }
    .cashier-badge { background: #334155; padding: 4px 10px; border-radius: 6px; color: var(--text-main); font-weight: 600; }

    .header-right { display: flex; align-items: center; gap: 12px; }
    .network-badge {
      display: flex;
      align-items: center;
      gap: 6px;
      padding: 6px 12px;
      border-radius: 20px;
      font-size: 12px;
      font-weight: 600;
      background: rgba(16, 185, 129, 0.15);
      color: var(--accent-green);
      border: 1px solid rgba(16, 185, 129, 0.3);
      cursor: pointer;
    }
    .network-badge.offline {
      background: rgba(239, 68, 68, 0.15);
      color: var(--accent-red);
      border-color: rgba(239, 68, 68, 0.3);
    }
    .status-dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: currentColor;
      box-shadow: 0 0 8px currentColor;
    }

    /* Main Container */
    main {
      flex: 1;
      display: grid;
      grid-template-columns: 460px 1fr;
      height: calc(100vh - 58px);
      overflow: hidden;
    }

    /* Left Panel: Cart & Payment */
    .cart-panel {
      background: var(--bg-secondary);
      border-right: 1px solid var(--border-color);
      display: flex;
      flex-direction: column;
      height: 100%;
    }

    .cart-header-actions {
      padding: 12px 16px;
      background: rgba(0,0,0,0.2);
      border-bottom: 1px solid var(--border-color);
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    .cart-title { font-weight: 700; font-size: 15px; display: flex; align-items: center; gap: 8px; }
    .btn-clear {
      background: transparent;
      border: 1px solid var(--border-color);
      color: var(--text-muted);
      padding: 4px 8px;
      border-radius: 6px;
      font-size: 12px;
      cursor: pointer;
    }
    .btn-clear:hover { background: rgba(239, 68, 68, 0.15); color: var(--accent-red); border-color: var(--accent-red); }

    .cart-table-wrap {
      flex: 1;
      overflow-y: auto;
      padding: 8px;
    }

    .cart-table {
      width: 100%;
      border-collapse: collapse;
      font-size: 13px;
    }
    .cart-table th {
      text-align: left;
      padding: 8px;
      color: var(--text-muted);
      border-bottom: 1px solid var(--border-color);
      font-weight: 600;
    }
    .cart-table td {
      padding: 10px 8px;
      border-bottom: 1px solid rgba(255,255,255,0.05);
      vertical-align: middle;
    }
    .item-name { font-weight: 600; font-size: 14px; margin-bottom: 2px; }
    .item-code { font-size: 11px; color: var(--text-muted); }
    .qty-controls { display: flex; align-items: center; gap: 4px; }
    .qty-btn {
      width: 24px;
      height: 24px;
      background: var(--bg-card);
      border: 1px solid var(--border-color);
      color: white;
      border-radius: 4px;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      font-weight: bold;
    }
    .qty-btn:hover { background: var(--accent-blue); }
    .item-del-btn {
      background: transparent;
      border: none;
      color: #64748b;
      cursor: pointer;
      font-size: 16px;
      padding: 4px;
    }
    .item-del-btn:hover { color: var(--accent-red); }

    .cart-empty-state {
      padding: 40px 20px;
      text-align: center;
      color: var(--text-muted);
      font-size: 14px;
      line-height: 1.6;
    }

    /* Summary & Payment Controls */
    .cart-footer {
      background: #182234;
      border-top: 1px solid var(--border-color);
      padding: 16px;
      display: flex;
      flex-direction: column;
      gap: 12px;
    }

    .summary-row {
      display: flex;
      justify-content: space-between;
      font-size: 13px;
      color: var(--text-muted);
    }
    .summary-total {
      display: flex;
      justify-content: space-between;
      align-items: baseline;
      padding-top: 8px;
      border-top: 1px dashed var(--border-color);
    }
    .total-label { font-size: 16px; font-weight: 700; color: var(--text-main); }
    .total-val { font-size: 28px; font-weight: 800; color: #38bdf8; }

    .payment-buttons-grid {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 8px;
    }

    .btn-pay {
      padding: 14px;
      border-radius: 10px;
      border: none;
      font-weight: 700;
      font-size: 15px;
      cursor: pointer;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 2px;
      transition: all 0.15s ease;
      color: white;
    }
    .btn-pay span.shortcut { font-size: 11px; opacity: 0.8; font-weight: 500; }
    .btn-cash { background: var(--accent-green); }
    .btn-cash:hover { background: var(--accent-green-hover); transform: translateY(-1px); }
    .btn-card { background: var(--accent-blue); }
    .btn-card:hover { background: var(--accent-blue-hover); transform: translateY(-1px); }
    .btn-veresiye { background: var(--accent-amber); grid-column: span 2; }
    .btn-veresiye:hover { background: #d97706; transform: translateY(-1px); }

    /* Right Panel: Catalog & Operations */
    .catalog-panel {
      display: flex;
      flex-direction: column;
      background: var(--bg-primary);
      height: 100%;
      overflow: hidden;
    }

    .search-bar-wrap {
      padding: 12px 16px;
      background: var(--bg-secondary);
      border-bottom: 1px solid var(--border-color);
      display: flex;
      gap: 12px;
    }

    .barcode-input-container {
      flex: 1;
      position: relative;
    }
    .barcode-input-container svg {
      position: absolute;
      left: 14px;
      top: 50%;
      transform: translateY(-50%);
      color: var(--text-muted);
    }
    .barcode-input {
      width: 100%;
      padding: 12px 14px 12px 42px;
      background: var(--bg-card);
      border: 2px solid var(--border-color);
      border-radius: 10px;
      color: white;
      font-size: 16px;
      font-weight: 600;
      outline: none;
      transition: border-color 0.2s;
    }
    .barcode-input:focus { border-color: var(--accent-blue); }

    .nav-tabs {
      display: flex;
      gap: 8px;
      padding: 12px 16px 0 16px;
      border-bottom: 1px solid var(--border-color);
      background: var(--bg-secondary);
    }
    .tab-btn {
      background: transparent;
      border: none;
      color: var(--text-muted);
      padding: 10px 16px;
      font-size: 13px;
      font-weight: 600;
      border-bottom: 3px solid transparent;
      cursor: pointer;
    }
    .tab-btn.active {
      color: var(--accent-blue);
      border-bottom-color: var(--accent-blue);
    }

    .tab-content {
      flex: 1;
      overflow-y: auto;
      padding: 16px;
    }

    /* Quick Products Grid */
    .products-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
      gap: 12px;
    }

    .product-tile {
      background: var(--bg-secondary);
      border: 1px solid var(--border-color);
      border-radius: 12px;
      padding: 14px 10px;
      display: flex;
      flex-direction: column;
      justify-content: space-between;
      height: 115px;
      cursor: pointer;
      transition: all 0.15s ease;
      position: relative;
      overflow: hidden;
    }
    .product-tile:hover {
      background: var(--bg-card);
      border-color: #475569;
      transform: translateY(-2px);
      box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    }
    .product-tile.manav { border-left: 4px solid #10b981; }
    .product-tile.temel { border-left: 4px solid #f59e0b; }
    .product-tile.icecek { border-left: 4px solid #3b82f6; }

    .tile-name {
      font-size: 13px;
      font-weight: 600;
      line-height: 1.3;
      color: var(--text-main);
    }
    .tile-price {
      font-size: 16px;
      font-weight: 800;
      color: #38bdf8;
      margin-top: 6px;
    }
    .tile-unit {
      font-size: 11px;
      color: var(--text-muted);
      font-weight: normal;
    }

    /* Modal Styling */
    .modal-backdrop {
      position: fixed;
      top: 0; left: 0; right: 0; bottom: 0;
      background: rgba(0,0,0,0.7);
      display: none;
      align-items: center;
      justify-content: center;
      z-index: 100;
    }
    .modal-box {
      background: var(--bg-secondary);
      border: 1px solid var(--border-color);
      border-radius: 16px;
      width: 480px;
      max-width: 90vw;
      padding: 24px;
      box-shadow: 0 20px 40px rgba(0,0,0,0.5);
    }
    .modal-title { font-size: 18px; font-weight: 700; margin-bottom: 16px; }
    .tender-calc-display {
      background: #0f172a;
      border: 1px solid var(--border-color);
      border-radius: 12px;
      padding: 16px;
      text-align: right;
      margin-bottom: 16px;
    }
    .tender-label { font-size: 12px; color: var(--text-muted); }
    .tender-val { font-size: 32px; font-weight: 800; color: #10b981; }
    .quick-cash-row { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; margin-bottom: 16px; }
    .btn-quick-cash {
      padding: 10px;
      border-radius: 8px;
      background: var(--bg-card);
      border: 1px solid var(--border-color);
      color: white;
      font-weight: 700;
      cursor: pointer;
    }
    .btn-quick-cash:hover { background: var(--accent-blue); }

    /* Toast Notification */
    .toast {
      position: fixed;
      bottom: 20px;
      right: 20px;
      background: #1e293b;
      border-left: 4px solid var(--accent-green);
      color: white;
      padding: 12px 20px;
      border-radius: 8px;
      box-shadow: 0 8px 24px rgba(0,0,0,0.4);
      display: none;
      z-index: 200;
      font-size: 13px;
      font-weight: 500;
    }
  </style>
</head>
<body>

  <!-- Header -->
  <header>
    <div class="brand-section">
      <div class="logo-badge">QORDY POS</div>
      <div class="store-info">
        <div class="store-name" id="businessName">QORDY Cadde Esnaf</div>
        <div class="store-sub" id="branchInfo">Kadıköy Çarşı Şubesi • Kasa: KASA-01</div>
      </div>
    </div>

    <div class="header-center">
      <div class="cashier-badge" id="cashierName">Kasiyer: Ahmet Kasiyer</div>
      <div id="liveClock">--:--:--</div>
    </div>

    <div class="header-right">
      <div class="network-badge" id="networkBadge" onclick="triggerManualSync()">
        <div class="status-dot"></div>
        <span id="networkText">Çevrimdışı (Lokal Mod)</span>
      </div>
      <button class="btn-clear" onclick="openZReportModal()">Z-Raporu</button>
    </div>
  </header>

  <!-- Main Work Area -->
  <main>
    <!-- Left Column: Cart & Checkout -->
    <div class="cart-panel">
      <div class="cart-header-actions">
        <div class="cart-title">
          <span>Satış Sepeti</span>
          <span style="font-size:12px; font-weight:normal; color:var(--text-muted)" id="cartItemCount">(0 Kalem)</span>
        </div>
        <button class="btn-clear" onclick="clearCart()">Sepeti Boşalt (F6)</button>
      </div>

      <div class="cart-table-wrap">
        <table class="cart-table" id="cartTable">
          <thead>
            <tr>
              <th>Ürün Adı</th>
              <th style="width: 100px;">Miktar</th>
              <th style="width: 80px; text-align:right">Fiyat</th>
              <th style="width: 80px; text-align:right">Tutar</th>
              <th style="width: 30px;"></th>
            </tr>
          </thead>
          <tbody id="cartTableBody">
            <!-- Dinamik kalemler buraya gelecek -->
          </tbody>
        </table>
        <div class="cart-empty-state" id="emptyCartNotice">
          🛒 Sepetiniz boş.<br/>
          Barkodu okutun veya sağdaki hızlı butonlara tıklayın.
        </div>
      </div>

      <div class="cart-footer">
        <div class="summary-row">
          <span>Ara Toplam</span>
          <span id="subtotalVal">0.00 ₺</span>
        </div>
        <div class="summary-row">
          <span>KDV Toplam</span>
          <span id="vatVal">0.00 ₺</span>
        </div>
        <div class="summary-row" id="discountRow" style="display:none; color:#10b981;">
          <span>İndirim</span>
          <span id="discountVal">-0.00 ₺</span>
        </div>
        <div class="summary-total">
          <span class="total-label">GENEL TOPLAM</span>
          <span class="total-val" id="grandTotalVal">0.00 ₺</span>
        </div>

        <div class="payment-buttons-grid">
          <button class="btn-pay btn-cash" onclick="initiateCheckout('CASH')">
            <span>NAKİT ÖDEME</span>
            <span class="shortcut">Klavye Kısayolu: [F2]</span>
          </button>
          <button class="btn-pay btn-card" onclick="initiateCheckout('CREDIT_CARD')">
            <span>KREDİ KARTI</span>
            <span class="shortcut">Klavye Kısayolu: [F3]</span>
          </button>
          <button class="btn-pay btn-veresiye" onclick="openVeresiyeModal()">
            <span>VERESİYE / CARİ BORÇ KAYDET</span>
            <span class="shortcut">Klavye Kısayolu: [F4]</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Right Column: Barcode & Touch Grid -->
    <div class="catalog-panel">
      <div class="search-bar-wrap">
        <div class="barcode-input-container">
          <svg width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v1m6 11h2m-6 0h-2v4m0-11v3m0 0h.01M12 12h4.01M16 20h4M4 12h4m12 0h.01M5 8h2a1 1 0 001-1V5a1 1 0 00-1-1H5a1 1 0 00-1 1v2a1 1 0 001 1zm12 0h2a1 1 0 001-1V5a1 1 0 00-1-1h-2a1 1 0 00-1 1v2a1 1 0 001 1zM5 20h2a1 1 0 001-1v-2a1 1 0 00-1-1H5a1 1 0 00-1 1v2a1 1 0 001 1z"></path>
          </svg>
          <input
            type="text"
            id="barcodeInput"
            class="barcode-input"
            placeholder="Barkod okutun veya ürün adı arayın (Enter)..."
            autocomplete="off"
            autofocus
          />
        </div>
      </div>

      <div class="nav-tabs">
        <button class="tab-btn active" onclick="switchTab('quick')">Hızlı Ürünler</button>
        <button class="tab-btn" onclick="switchTab('manav')">Manav & Terazi</button>
        <button class="tab-btn" onclick="switchTab('veresiye')">Veresiye Müşterileri</button>
      </div>

      <div class="tab-content" id="tabContent">
        <!-- Hızlı Ürünler Grid -->
        <div class="products-grid" id="quickProductsGrid">
          <!-- JS ile doldurulacak -->
        </div>
      </div>
    </div>
  </main>

  <!-- Cash Calculator Modal -->
  <div class="modal-backdrop" id="cashModal">
    <div class="modal-box">
      <div class="modal-title">Nakit Ödeme & Para Üstü</div>
      <div class="tender-calc-display">
        <div class="tender-label">Ödenecek Tutar</div>
        <div class="tender-val" id="modalTenderDue" style="font-size:24px; color:#38bdf8;">0.00 ₺</div>
      </div>

      <div class="quick-cash-row">
        <button class="btn-quick-cash" onclick="setCashTender(50)">50 ₺</button>
        <button class="btn-quick-cash" onclick="setCashTender(100)">100 ₺</button>
        <button class="btn-quick-cash" onclick="setCashTender(200)">200 ₺</button>
        <button class="btn-quick-cash" onclick="setCashExact()">Tam Para</button>
      </div>

      <div style="margin-bottom:12px">
        <label style="font-size:12px; color:var(--text-muted); display:block; margin-bottom:4px;">Alınan Nakit:</label>
        <input type="number" id="cashInput" class="barcode-input" style="padding-left:14px;" oninput="calcChange()">
      </div>

      <div class="tender-calc-display">
        <div class="tender-label">Verilecek Para Üstü</div>
        <div class="tender-val" id="modalChangeVal">0.00 ₺</div>
      </div>

      <div style="display:flex; gap:8px;">
        <button class="btn-clear" style="flex:1; padding:12px;" onclick="closeModal('cashModal')">Vazgeç</button>
        <button class="btn-pay btn-cash" style="flex:2; padding:12px;" onclick="finalizeCashPayment()">SATIŞI TAMAMLA & FİŞ BAS</button>
      </div>
    </div>
  </div>

  <!-- Veresiye Müşteri Seçim Modalı -->
  <div class="modal-backdrop" id="veresiyeModal">
    <div class="modal-box">
      <div class="modal-title">Veresiye Cari Seçimi</div>
      <div style="margin-bottom:12px;">
        <input type="text" id="customerSearchInput" class="barcode-input" placeholder="Müşteri adı veya telefon ara..." oninput="filterCustomers()">
      </div>
      <div style="max-height:220px; overflow-y:auto; margin-bottom:16px;" id="customerListModal">
        <!-- JS Dolduracak -->
      </div>
      <div style="display:flex; gap:8px;">
        <button class="btn-clear" style="flex:1; padding:12px;" onclick="closeModal('veresiyeModal')">Kapat</button>
      </div>
    </div>
  </div>

  <!-- Toast -->
  <div class="toast" id="toastMessage"></div>

  <script>
    // State
    let cart = [];
    let customers = [];
    let quickProducts = [];
    let currentSaleTotal = 0;
    let selectedCustomerId = null;

    // Clock
    setInterval(() => {
      const now = new Date();
      document.getElementById('liveClock').innerText = now.toLocaleTimeString('tr-TR');
    }, 1000);

    // Init
    window.addEventListener('DOMContentLoaded', () => {
      fetchStatus();
      loadQuickProducts();
      loadCustomers();
      setupKeyboardShortcuts();

      setInterval(fetchStatus, 3000);

      const barcodeInput = document.getElementById('barcodeInput');
      barcodeInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
          e.preventDefault();
          handleBarcodeSubmit(barcodeInput.value);
          barcodeInput.value = '';
        }
      });
    });

    function setupKeyboardShortcuts() {
      window.addEventListener('keydown', (e) => {
        if (e.key === 'F2') {
          e.preventDefault();
          initiateCheckout('CASH');
        } else if (e.key === 'F3') {
          e.preventDefault();
          initiateCheckout('CREDIT_CARD');
        } else if (e.key === 'F4') {
          e.preventDefault();
          openVeresiyeModal();
        } else if (e.key === 'F6') {
          e.preventDefault();
          clearCart();
        }
      });
    }

    async function fetchStatus() {
      try {
        const res = await fetch('/api/status');
        if (res.ok) {
          const data = await res.json();
          document.getElementById('businessName').innerText = data.business_name;
          document.getElementById('branchInfo').innerText = `${data.branch_name} • Kasa: ${data.register_id}`;
          document.getElementById('cashierName').innerText = `Kasiyer: ${data.cashier_name}`;

          const badge = document.getElementById('networkBadge');
          const netText = document.getElementById('networkText');
          if (data.is_online) {
            badge.className = 'network-badge';
            netText.innerText = data.pending_sync_count > 0 
              ? `Çevrimiçi (${data.pending_sync_count} Bekleyen)` 
              : 'Çevrimiçi (Bulut Aktif)';
          } else {
            badge.className = 'network-badge offline';
            netText.innerText = data.pending_sync_count > 0
              ? `Çevrimdışı (${data.pending_sync_count} Bekleyen)`
              : 'Çevrimdışı (Lokal Mod)';
          }
        }
      } catch (e) {
        // Offline durum
        const badge = document.getElementById('networkBadge');
        badge.className = 'network-badge offline';
        document.getElementById('networkText').innerText = 'Çevrimdışı (Lokal)';
      }
    }

    async function loadQuickProducts() {
      try {
        const res = await fetch('/api/products/quick');
        if (res.ok) {
          quickProducts = await res.json();
          renderQuickProducts(quickProducts);
        }
      } catch (e) {}
    }

    function renderQuickProducts(products) {
      const grid = document.getElementById('quickProductsGrid');
      grid.innerHTML = '';
      products.forEach(p => {
        const tile = document.createElement('div');
        let catClass = 'temel';
        if (p.category === 'Manav' || p.is_weighed) catClass = 'manav';
        else if (p.category === 'İçecek') catClass = 'icecek';

        tile.className = `product-tile ${catClass}`;
        tile.onclick = () => addProductToCart(p);
        tile.innerHTML = `
          <div class="tile-name">${p.name}</div>
          <div>
            <div class="tile-price">${p.price.toFixed(2)} ₺</div>
            <div class="tile-unit">${p.is_weighed ? 'kg başına' : p.unit}</div>
          </div>
        `;
        grid.appendChild(tile);
      });
    }

    async function handleBarcodeSubmit(rawBarcode) {
      const clean = rawBarcode.trim();
      if (!clean) return;

      try {
        const res = await fetch('/api/barcode/scan', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ barcode: clean })
        });

        if (res.ok) {
          const data = await res.json();
          if (data.found && data.suggested_item) {
            addItemToCart(data.suggested_item);
            showToast(`✅ ${data.suggested_item.name} eklendi`);
          } else {
            showToast(`❌ Barkod bulunamadı: ${clean}`, true);
          }
        }
      } catch (err) {
        showToast('Hata oluştu', true);
      }
    }

    function addProductToCart(product) {
      if (product.is_weighed) {
        const weightInput = prompt(`${product.name} için tartılan gramajı girin (Örn: 1.250 kg için 1.250 veya 1250g):`, "1.000");
        if (!weightInput) return;
        let qty = parseFloat(weightInput.replace(',', '.'));
        if (qty > 50) qty = qty / 1000; // gram girildiyse kg'ye çevir

        const total = qty * product.price;
        addItemToCart({
          product_id: product.id,
          barcode: product.barcode,
          name: product.name,
          unit_price: product.price,
          quantity: qty,
          unit: 'kg',
          vat_rate: product.vat_rate,
          discount: 0.0,
          total: total,
          is_weighed: true,
          notes: null
        });
      } else {
        // Normal adetli ürün
        const existing = cart.find(i => i.product_id === product.id && !i.is_weighed);
        if (existing) {
          existing.quantity += 1;
          existing.total = existing.quantity * existing.unit_price;
          renderCart();
        } else {
          addItemToCart({
            product_id: product.id,
            barcode: product.barcode,
            name: product.name,
            unit_price: product.price,
            quantity: 1.0,
            unit: product.unit,
            vat_rate: product.vat_rate,
            discount: 0.0,
            total: product.price,
            is_weighed: false,
            notes: null
          });
        }
      }
    }

    function addItemToCart(item) {
      cart.push(item);
      renderCart();
    }

    function changeQty(index, delta) {
      const item = cart[index];
      if (!item) return;

      if (item.is_weighed) {
        item.quantity += delta * 0.100; // 100g adımlar
      } else {
        item.quantity += delta;
      }

      if (item.quantity <= 0.001) {
        cart.splice(index, 1);
      } else {
        item.total = item.quantity * item.unit_price;
      }
      renderCart();
    }

    function removeItem(index) {
      cart.splice(index, 1);
      renderCart();
    }

    function clearCart() {
      cart = [];
      selectedCustomerId = null;
      renderCart();
      document.getElementById('barcodeInput').focus();
    }

    function renderCart() {
      const tbody = document.getElementById('cartTableBody');
      const emptyNotice = document.getElementById('emptyCartNotice');
      tbody.innerHTML = '';

      if (cart.length === 0) {
        emptyNotice.style.display = 'block';
        document.getElementById('cartItemCount').innerText = '(0 Kalem)';
      } else {
        emptyNotice.style.display = 'none';
        document.getElementById('cartItemCount').innerText = `(${cart.length} Kalem)`;
      }

      let subtotal = 0;
      let vatTotal = 0;

      cart.forEach((item, idx) => {
        subtotal += item.total;
        const itemVat = item.total - (item.total / (1.0 + (item.vat_rate / 100.0)));
        vatTotal += itemVat;

        const tr = document.createElement('tr');
        const qtyDisplay = item.is_weighed ? `${item.quantity.toFixed(3)} kg` : `${item.quantity} ${item.unit}`;

        tr.innerHTML = `
          <td>
            <div class="item-name">${item.name}</div>
            <div class="item-code">${item.barcode}</div>
          </td>
          <td>
            <div class="qty-controls">
              <button class="qty-btn" onclick="changeQty(${idx}, -1)">-</button>
              <span style="font-weight:600; min-width:45px; text-align:center;">${qtyDisplay}</span>
              <button class="qty-btn" onclick="changeQty(${idx}, 1)">+</button>
            </div>
          </td>
          <td style="text-align:right;">${item.unit_price.toFixed(2)}</td>
          <td style="text-align:right; font-weight:700; color:#38bdf8;">${item.total.toFixed(2)}</td>
          <td style="text-align:center;">
            <button class="item-del-btn" onclick="removeItem(${idx})">×</button>
          </td>
        `;
        tbody.appendChild(tr);
      });

      currentSaleTotal = subtotal;
      document.getElementById('subtotalVal').innerText = `${subtotal.toFixed(2)} ₺`;
      document.getElementById('vatVal').innerText = `${vatTotal.toFixed(2)} ₺`;
      document.getElementById('grandTotalVal').innerText = `${subtotal.toFixed(2)} ₺`;
    }

    function initiateCheckout(method) {
      if (cart.length === 0) {
        showToast('Sepetiniz boş!', true);
        return;
      }

      if (method === 'CASH') {
        document.getElementById('modalTenderDue').innerText = `${currentSaleTotal.toFixed(2)} ₺`;
        document.getElementById('cashInput').value = currentSaleTotal.toFixed(2);
        calcChange();
        openModal('cashModal');
      } else if (method === 'CREDIT_CARD') {
        executeCheckout('CREDIT_CARD', currentSaleTotal, null);
      }
    }

    function setCashTender(val) {
      document.getElementById('cashInput').value = val;
      calcChange();
    }

    function setCashExact() {
      document.getElementById('cashInput').value = currentSaleTotal.toFixed(2);
      calcChange();
    }

    function calcChange() {
      const given = parseFloat(document.getElementById('cashInput').value) || 0;
      const change = Math.max(0, given - currentSaleTotal);
      document.getElementById('modalChangeVal').innerText = `${change.toFixed(2)} ₺`;
    }

    function finalizeCashPayment() {
      const given = parseFloat(document.getElementById('cashInput').value) || currentSaleTotal;
      closeModal('cashModal');
      executeCheckout('CASH', given, null);
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
          showToast(`🎉 Satış tamamlandı! Fiş No: ${resp.receipt_no}`);
          clearCart();
          fetchStatus();
        } else {
          showToast('Satış işleminde hata oluştu', true);
        }
      } catch (e) {
        showToast('İletişim hatası', true);
      }
    }

    async function loadCustomers() {
      try {
        const res = await fetch('/api/customers');
        if (res.ok) {
          customers = await res.json();
        }
      } catch (e) {}
    }

    function openVeresiyeModal() {
      if (cart.length === 0) {
        showToast('Sepetiniz boş!', true);
        return;
      }
      renderCustomerModalList(customers);
      openModal('veresiyeModal');
    }

    function renderCustomerModalList(list) {
      const wrap = document.getElementById('customerListModal');
      wrap.innerHTML = '';
      list.forEach(c => {
        const div = document.createElement('div');
        div.style = 'background:var(--bg-card); padding:10px 14px; border-radius:8px; margin-bottom:8px; display:flex; justify-content:space-between; align-items:center; cursor:pointer;';
        div.onclick = () => {
          closeModal('veresiyeModal');
          executeCheckout('VERESIYE', currentSaleTotal, c.id);
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

    function filterCustomers() {
      const q = document.getElementById('customerSearchInput').value.toLowerCase();
      const filtered = customers.filter(c => c.name.toLowerCase().includes(q) || c.phone.includes(q));
      renderCustomerModalList(filtered);
    }

    async function triggerManualSync() {
      showToast('Bulut eşitlemesi tetikleniyor...');
      try {
        await fetch('/api/sync/trigger', { method: 'POST' });
        setTimeout(fetchStatus, 1500);
      } catch (e) {}
    }

    async function openZReportModal() {
      try {
        const res = await fetch('/api/reports/z');
        if (res.ok) {
          const z = await res.json();
          alert(`📊 GÜN SONU Z RAPORU (${z.date})\nToplam Satış: ${z.total_sales_count} Adet\nNet Ciro: ${z.total_net.toFixed(2)} TL\nNakit: ${z.cash_total.toFixed(2)} TL\nKredi Kartı: ${z.card_total.toFixed(2)} TL\nVeresiye: ${z.veresiye_total.toFixed(2)} TL\n\n(Yazıcıya gönderildi)`);
        }
      } catch (e) {
        alert('Rapor alınamadı.');
      }
    }

    function switchTab(tab) {
      const btns = document.querySelectorAll('.tab-btn');
      btns.forEach(b => b.classList.remove('active'));
      event.target.classList.add('active');

      if (tab === 'quick') {
        renderQuickProducts(quickProducts);
      } else if (tab === 'manav') {
        renderQuickProducts(quickProducts.filter(p => p.is_weighed || p.category === 'Manav'));
      } else if (tab === 'veresiye') {
        const grid = document.getElementById('quickProductsGrid');
        grid.innerHTML = '';
        customers.forEach(c => {
          const tile = document.createElement('div');
          tile.className = 'product-tile';
          tile.style = 'border-left: 4px solid #f59e0b;';
          tile.innerHTML = `
            <div class="tile-name">${c.name}</div>
            <div>
              <div class="tile-price" style="color:#f59e0b;">${c.balance.toFixed(2)} ₺</div>
              <div class="tile-unit">Mevcut Borç</div>
            </div>
          `;
          grid.appendChild(tile);
        });
      }
    }

    function openModal(id) { document.getElementById(id).style.display = 'flex'; }
    function closeModal(id) { document.getElementById(id).style.display = 'none'; document.getElementById('barcodeInput').focus(); }

    function showToast(msg, isError = false) {
      const t = document.getElementById('toastMessage');
      t.innerText = msg;
      t.style.borderLeftColor = isError ? 'var(--accent-red)' : 'var(--accent-green)';
      t.style.display = 'block';
      setTimeout(() => { t.style.display = 'none'; }, 3000);
    }
  </script>
</body>
</html>"#
}
