// QORDY POS - Main Application Bootstrapper & Controller
const App = {
  ws: null,

  init() {
    Cart.init();
    this.initWebSocket();
    this.fetchStatus();
    this.loadCatalog();
    Views.loadProducts();
    Views.loadCustomers();
    this.loadParkedCount();

    setInterval(() => this.fetchStatus(), 3000);

    // Barcode scanner listener
    const barcodeInput = document.getElementById('posBarcodeField');
    if (barcodeInput) {
      barcodeInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
          e.preventDefault();
          this.handleBarcodeScan(barcodeInput.value);
          barcodeInput.value = '';
        }
      });
    }

    // Keyboard Shortcuts (F1-F12, Esc)
    window.addEventListener('keydown', (e) => {
      if (e.key === 'F2') { e.preventDefault(); this.openCashModal(); }
      else if (e.key === 'F3') { e.preventDefault(); Terminal.checkoutCard(); }
      else if (e.key === 'F4') { e.preventDefault(); this.openVeresiyeCustomerModal(); }
      else if (e.key === 'F6') { e.preventDefault(); Cart.clear(); }
      else if (e.key === 'F7') { e.preventDefault(); this.parkCurrentCart(); }
      else if (e.key === 'Escape') {
        document.querySelectorAll('.modal-overlay').forEach(m => m.style.display = 'none');
      }
    });
  },

  initWebSocket() {
    const loc = window.location;
    const wsUri = (loc.protocol === 'https:' ? 'wss://' : 'ws://') + loc.host + '/ws/pos';
    this.ws = new WebSocket(wsUri);

    this.ws.onopen = () => console.log('✅ POS WebSocket bağlı');
    this.ws.onclose = () => setTimeout(() => this.initWebSocket(), 2000);
    this.ws.onerror = (e) => console.warn('WS Hata', e);
  },

  async fetchStatus() {
    try {
      const d = await Api.getStatus();
      const elStore = document.getElementById('hdrStoreName');
      const elBranch = document.getElementById('hdrBranch');
      if (elStore) elStore.innerText = d.business_name;
      if (elBranch) elBranch.innerText = `${d.branch_name} • Kasa: ${d.register_id}`;

      const badge = document.getElementById('netStatusBadge');
      const txt = document.getElementById('netStatusText');
      if (badge && txt) {
        if (d.is_online) {
          badge.className = 'pill-badge';
          txt.innerText = d.pending_sync_count > 0 ? `Çevrimiçi (${d.pending_sync_count} Bekleyen)` : 'Çevrimiçi (Bulut Aktif)';
        } else {
          badge.className = 'pill-badge offline';
          txt.innerText = d.pending_sync_count > 0 ? `Çevrimdışı (${d.pending_sync_count} Bekleyen)` : 'Çevrimdışı (Lokal Mod)';
        }
      }
    } catch (e) {}
  },

  async triggerSyncNow() {
    this.showToast('Bulut eşitlemesi tetikleniyor...');
    await Api.triggerSync();
    setTimeout(() => this.fetchStatus(), 1500);
  },

  async loadCatalog() {
    try {
      const list = await Api.getQuickProducts();
      const grid = document.getElementById('posTilesGrid');
      if (!grid) return;
      grid.innerHTML = '';
      list.forEach(p => {
        const tile = document.createElement('div');
        tile.className = `pos-tile ${p.is_weighed ? 'weighed' : 'standard'}`;
        tile.onclick = () => Cart.addItem(p);
        tile.innerHTML = `
          <div class="tile-text">${p.name}</div>
          <div class="tile-bottom">
            <span class="tile-val">${p.price.toFixed(2)} ₺</span>
            <span class="tile-unit-label">${p.is_weighed ? 'kg başına' : p.unit}</span>
          </div>
        `;
        grid.appendChild(tile);
      });
    } catch (e) {}
  },

  async handleBarcodeScan(barcode) {
    const clean = barcode.trim();
    if (!clean) return;

    try {
      const d = await Api.scanBarcode(clean);
      if (d.found && d.suggested_item) {
        Cart.pushItem(d.suggested_item);
        this.showToast(`✅ ${d.suggested_item.name} eklendi`);
      } else {
        this.showToast(`❌ Barkod bulunamadı: ${clean}`, true);
      }
    } catch (e) {}
  },

  switchMainScreen(screenId) {
    document.querySelectorAll('.screen-view').forEach(s => s.classList.remove('active'));
    document.querySelectorAll('.nav-seg-btn').forEach(b => b.classList.remove('active'));

    const target = document.getElementById(`screen-${screenId}`);
    if (target) target.classList.add('active');

    const btns = Array.from(document.querySelectorAll('.nav-seg-btn'));
    const activeBtn = btns.find(b => b.getAttribute('onclick')?.includes(`'${screenId}'`));
    if (activeBtn) activeBtn.classList.add('active');

    if (screenId === 'pos') {
      const f = document.getElementById('posBarcodeField');
      if (f) f.focus();
    } else if (screenId === 'transactions') Views.loadTransactions();
    else if (screenId === 'receiving') Views.loadReceiving();
    else if (screenId === 'campaigns') Views.loadCampaigns();
    else if (screenId === 'transfers') Views.loadTransfers();
    else if (screenId === 'registers') Views.loadShiftStatus();
    else if (screenId === 'products') Views.loadProducts();
    else if (screenId === 'stock') Views.loadStockSessions();
    else if (screenId === 'customers') Views.loadCustomers();
    else if (screenId === 'wastes') Views.loadWastes();
    else if (screenId === 'labels') Views.loadShelfLabels();
    else if (screenId === 'cash') Views.loadCashMovements();
    else if (screenId === 'reports') Views.fetchAndDisplayZReport();
  },

  openCustomerDisplayTab() {
    window.open('/customer', '_blank', 'width=1024,height=768');
  },

  // Cash Checkout
  openCashModal() {
    if (Cart.getItems().length === 0) {
      this.showToast('Sepetiniz boş!', true);
      return;
    }
    const total = Cart.getTotal();
    document.getElementById('cashModalDue').innerText = `${total.toFixed(2)} ₺`;
    document.getElementById('cashGivenInput').value = total.toFixed(2);
    this.updateChangeCalculation();
    this.openModal('cashModal');
  },

  setCashVal(val) {
    document.getElementById('cashGivenInput').value = val;
    this.updateChangeCalculation();
  },

  setCashValExact() {
    const total = Cart.getTotal();
    document.getElementById('cashGivenInput').value = total.toFixed(2);
    this.updateChangeCalculation();
  },

  updateChangeCalculation() {
    const total = Cart.getTotal();
    const given = parseFloat(document.getElementById('cashGivenInput').value) || 0;
    const change = Math.max(0, given - total);
    document.getElementById('cashModalChange').innerText = `${change.toFixed(2)} ₺`;
  },

  finalizeCashSale() {
    const total = Cart.getTotal();
    const given = parseFloat(document.getElementById('cashGivenInput').value) || total;
    this.closeModal('cashModal');
    this.executeCheckout('CASH', given, null);
  },

  async executeCheckout(method, tender, customerId) {
    try {
      const payload = {
        items: Cart.getItems(),
        payment_method: method,
        tender_amount: tender,
        customer_id: customerId,
        discount_amount: 0.0,
        notes: null
      };

      const resp = await Api.checkout(payload);
      this.showToast(`🎉 Satış tamamlandı! Fiş: ${resp.receipt_no}`);
      Cart.clear();
      this.fetchStatus();
    } catch (e) {
      this.showToast('Satış işleminde hata!', true);
    }
  },

  // Veresiye
  openVeresiyeCustomerModal() {
    if (Cart.getItems().length === 0) {
      this.showToast('Sepetiniz boş!', true);
      return;
    }
    this.renderVeresiyeSelectionList(Views.customers);
    this.openModal('veresiyeSelectModal');
  },

  renderVeresiyeSelectionList(list) {
    const wrap = document.getElementById('veresiyeModalList');
    if (!wrap) return;
    wrap.innerHTML = '';
    list.forEach(c => {
      const div = document.createElement('div');
      div.style = 'background:var(--bg-card); padding:10px 14px; border-radius:8px; display:flex; justify-content:space-between; align-items:center; cursor:pointer;';
      div.onclick = () => {
        this.closeModal('veresiyeSelectModal');
        const total = Cart.getTotal();
        this.executeCheckout('VERESIYE', total, c.id);
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
  },

  filterVeresiyeList() {
    const q = document.getElementById('veresiyeSearch').value.toLowerCase();
    const filtered = Views.customers.filter(c => c.name.toLowerCase().includes(q) || c.phone.includes(q));
    this.renderVeresiyeSelectionList(filtered);
  },

  // Parked Carts
  async parkCurrentCart() {
    if (Cart.getItems().length === 0) return;
    const note = prompt('Bu sepet için bir not / masa / müşteri adı girin:', 'Müşteri 1');
    if (!note) return;

    try {
      await Api.parkCart({ note, items: Cart.getItems() });
      this.showToast('⏸️ Sepet bekletmeye alındı');
      Cart.clear();
      this.loadParkedCount();
    } catch (e) {}
  },

  async loadParkedCount() {
    try {
      const list = await Api.getParkedCarts();
      const el = document.getElementById('parkedBadgeCount');
      if (el) el.innerText = `(${list.length})`;
    } catch (e) {}
  },

  async openParkedCartsModal() {
    try {
      const list = await Api.getParkedCarts();
      const wrap = document.getElementById('parkedListContainer');
      if (!wrap) return;
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
              <button class="btn-top-action" style="background:var(--apple-blue);" onclick="App.restoreParkedCart('${p.id}')">Geri Yükle</button>
              <button class="btn-top-action" style="color:var(--apple-red);" onclick="App.deleteParkedCart('${p.id}')">Sil</button>
            </div>
          `;
          wrap.appendChild(div);
        });
      }
      this.openModal('parkedCartsModal');
    } catch (e) {}
  },

  async restoreParkedCart(id) {
    try {
      const list = await Api.getParkedCarts();
      const p = list.find(x => x.id === id);
      if (p) {
        Cart.items = p.items;
        Cart.render();
        await Api.deleteParkedCart(id);
        this.closeModal('parkedCartsModal');
        this.loadParkedCount();
        this.showToast('✅ Sepet kasaya geri çağrıldı');
      }
    } catch (e) {}
  },

  async deleteParkedCart(id) {
    try {
      await Api.deleteParkedCart(id);
      this.openParkedCartsModal();
      this.loadParkedCount();
    } catch (e) {}
  },

  // Modal & Toast Helpers
  openModal(id) {
    const el = document.getElementById(id);
    if (el) el.style.display = 'flex';
  },

  closeModal(id) {
    const el = document.getElementById(id);
    if (el) el.style.display = 'none';
  },

  showToast(msg, isError = false) {
    const t = document.getElementById('globalToast');
    if (!t) return;
    t.innerText = msg;
    t.style.borderLeftColor = isError ? 'var(--apple-red)' : 'var(--apple-green)';
    t.style.display = 'block';
    setTimeout(() => { t.style.display = 'none'; }, 3000);
  }
};

// Global delegation for onclick handlers in HTML
window.switchMainScreen = (id) => App.switchMainScreen(id);
window.triggerSyncNow = () => App.triggerSyncNow();
window.openTerminalConfigModal = () => Terminal.openConfigModal();
window.updateTerminalFieldsVisibility = () => Terminal.updateFieldsVisibility();
window.testTerminalConnection = () => Terminal.testConnection();
window.saveTerminalConfig = () => Terminal.saveConfig();
window.openParkedCartsModal = () => App.openParkedCartsModal();
window.parkCurrentCart = () => App.parkCurrentCart();
window.clearCart = () => Cart.clear();
window.openCashModal = () => App.openCashModal();
window.setCashVal = (v) => App.setCashVal(v);
window.setCashValExact = () => App.setCashValExact();
window.updateChangeCalculation = () => App.updateChangeCalculation();
window.finalizeCashSale = () => App.finalizeCashSale();
window.checkoutCard = () => Terminal.checkoutCard();
window.openVeresiyeCustomerModal = () => App.openVeresiyeCustomerModal();
window.filterVeresiyeList = () => App.filterVeresiyeList();
window.openNewProductModal = () => Views.openNewProductModal();
window.saveNewProduct = () => Views.saveNewProduct();
window.openNewStockSessionModal = () => Views.openNewStockSessionModal();
window.openNewCustomerModal = () => Views.openNewCustomerModal();
window.saveNewCustomer = () => Views.saveNewCustomer();
window.submitCustomerPayment = () => Views.submitCustomerPayment();
window.fetchAndDisplayZReport = () => Views.fetchAndDisplayZReport();
window.openCustomerDisplayTab = () => App.openCustomerDisplayTab();
window.closeModal = (id) => App.closeModal(id);

window.addEventListener('DOMContentLoaded', () => {
  App.init();
});
