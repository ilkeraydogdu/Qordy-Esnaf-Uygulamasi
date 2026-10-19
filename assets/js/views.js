// QORDY POS - Enterprise Retail Views Controller
const Views = {
  salesHistory: [],
  activeRefundSale: null,
  products: [],
  customers: [],
  activeStockSessionId: null,
  selectedCustomerIdForPayment: null,

  // 1. Transactions & Itemized Refund
  async loadTransactions() {
    try {
      this.salesHistory = await Api.getSalesHistory();
      this.renderTransactions(this.salesHistory);
    } catch (e) {
      App.showToast('İşlemler listelenemedi', true);
    }
  },

  renderTransactions(list) {
    const tbody = document.getElementById('transactionsTableBody');
    if (!tbody) return;
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
          <button class="btn-top-action" onclick="Views.openRefundModal('${s.id}')">🔄 Kalem İade Et</button>
        </td>
      `;
      tbody.appendChild(tr);
    });
  },

  filterTransactions() {
    const q = (document.getElementById('transactionsSearch').value || '').toLowerCase();
    const filtered = this.salesHistory.filter(s =>
      (s.receipt_no && s.receipt_no.toLowerCase().includes(q)) ||
      (s.created_at && s.created_at.toLowerCase().includes(q))
    );
    this.renderTransactions(filtered);
  },

  openRefundModal(saleId) {
    const sale = this.salesHistory.find(s => s.id === saleId);
    if (!sale) return;
    this.activeRefundSale = sale;

    document.getElementById('refundModalReceiptNo').innerText = sale.receipt_no;
    document.getElementById('refundModalOriginalTotal').innerText = `${sale.grand_total.toFixed(2)} ₺`;

    const listDiv = document.getElementById('refundItemsList');
    listDiv.innerHTML = '';

    sale.items.forEach((item, idx) => {
      const itemRow = document.createElement('div');
      itemRow.style = 'background:var(--bg-card); padding:10px 14px; border-radius:8px; display:flex; justify-content:space-between; align-items:center;';
      itemRow.innerHTML = `
        <div style="display:flex; align-items:center; gap:10px;">
          <input type="checkbox" id="refundChk_${idx}" style="transform:scale(1.2); cursor:pointer;" onchange="Views.calculateRefundTotal()">
          <div>
            <div style="font-weight:700; font-size:13px;">${item.name}</div>
            <div style="font-size:11px; color:var(--text-muted);">${item.unit_price.toFixed(2)} ₺ / ${item.unit} (Satılan: ${item.quantity})</div>
          </div>
        </div>
        <div style="display:flex; align-items:center; gap:8px;">
          <span style="font-size:12px; color:var(--text-muted);">İade Adedi:</span>
          <input type="number" id="refundQty_${idx}" class="form-input" style="width:75px; padding:4px 8px; text-align:center; font-weight:700;" value="${item.quantity}" min="0.01" max="${item.quantity}" step="${item.is_weighed ? '0.05' : '1'}" oninput="Views.calculateRefundTotal()">
        </div>
      `;
      listDiv.appendChild(itemRow);
    });

    this.calculateRefundTotal();
    App.openModal('refundModal');
  },

  calculateRefundTotal() {
    if (!this.activeRefundSale) return;
    let total = 0;
    this.activeRefundSale.items.forEach((item, idx) => {
      const chk = document.getElementById(`refundChk_${idx}`);
      const qtyInput = document.getElementById(`refundQty_${idx}`);
      if (chk && chk.checked && qtyInput) {
        const qty = parseFloat(qtyInput.value) || 0;
        total += qty * item.unit_price;
      }
    });
    document.getElementById('refundTotalAmountText').innerText = `${total.toFixed(2)} ₺`;
  },

  async confirmItemRefund() {
    if (!this.activeRefundSale) return;
    const refundItems = [];
    let totalRefund = 0;

    this.activeRefundSale.items.forEach((item, idx) => {
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
      App.showToast('Lütfen en az bir ürün seçiniz!', true);
      return;
    }

    const reason = document.getElementById('refundReasonSelect').value;

    try {
      const data = await Api.refundSale({
        sale_id: this.activeRefundSale.id,
        items: refundItems,
        refund_amount: totalRefund,
        reason: reason
      });

      App.showToast(`✅ İade tamamlandı! İade Tutarı: ${data.refunded_amount.toFixed(2)} ₺`);
      App.closeModal('refundModal');
      this.loadTransactions();
      this.loadProducts();
      App.fetchStatus();
    } catch (e) {
      App.showToast(`İade başarısız: ${e.message}`, true);
    }
  },

  // 2. Product Management (CRUD)
  async loadProducts() {
    try {
      this.products = await Api.getProducts();
      const tbody = document.getElementById('productsTableBody');
      if (!tbody) return;
      tbody.innerHTML = '';
      this.products.forEach(p => {
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
            <button class="btn-top-action" style="display:inline-flex; color:var(--apple-red);" onclick="Views.deleteProduct('${p.id}')">Sil</button>
          </td>
        `;
        tbody.appendChild(tr);
      });
    } catch (e) {}
  },

  openNewProductModal() {
    App.openModal('newProductModal');
  },

  async saveNewProduct() {
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

    try {
      await Api.createProduct({ barcode, name, category, price, vat_rate, stock_qty, unit, is_weighed, quick_order: null });
      App.closeModal('newProductModal');
      App.showToast('✅ Ürün kataloğa eklendi');
      this.loadProducts();
      App.loadCatalog();
    } catch (e) {
      App.showToast('Ürün eklenemedi', true);
    }
  },

  async deleteProduct(id) {
    if (!confirm('Bu ürünü silmek istediğinize emin misiniz?')) return;
    try {
      await Api.deleteProduct(id);
      this.loadProducts();
      App.loadCatalog();
    } catch (e) {}
  },

  // 3. Customers & Veresiye Ledger
  async loadCustomers() {
    try {
      this.customers = await Api.getCustomers();
      const tbody = document.getElementById('customersTableBody');
      if (!tbody) return;
      tbody.innerHTML = '';
      this.customers.forEach(c => {
        const tr = document.createElement('tr');
        tr.innerHTML = `
          <td style="font-weight:700;">${c.name}</td>
          <td>${c.phone}</td>
          <td style="font-weight:700; color:var(--apple-amber);">${c.balance.toFixed(2)} ₺</td>
          <td>${c.credit_limit.toFixed(2)} ₺</td>
          <td style="color:var(--text-muted);">${c.note || '-'}</td>
          <td style="text-align:right;">
            <button class="btn-top-action" style="display:inline-flex; background:var(--apple-green);" onclick="Views.openPaymentModal('${c.id}', '${c.name}')">Tahsilat Yap</button>
          </td>
        `;
        tbody.appendChild(tr);
      });
    } catch (e) {}
  },

  openNewCustomerModal() {
    App.openModal('newCustomerModal');
  },

  async saveNewCustomer() {
    const name = document.getElementById('ncName').value.trim();
    const phone = document.getElementById('ncPhone').value.trim();
    const credit_limit = parseFloat(document.getElementById('ncLimit').value) || 2000.0;
    const note = document.getElementById('ncNote').value.trim();

    if (!name || !phone) {
      alert('Lütfen ad soyad ve telefon girin.');
      return;
    }

    try {
      await Api.createCustomer({ name, phone, credit_limit, note: note || null });
      App.closeModal('newCustomerModal');
      App.showToast('✅ Cari müşteri kaydedildi');
      this.loadCustomers();
    } catch (e) {
      App.showToast('Müşteri kaydedilemedi', true);
    }
  },

  openPaymentModal(cid, cname) {
    this.selectedCustomerIdForPayment = cid;
    document.getElementById('payModalTitle').innerText = `${cname} - Borç Tahsilatı`;
    document.getElementById('payModalAmount').value = '';
    App.openModal('paymentModal');
  },

  async submitCustomerPayment() {
    const amount = parseFloat(document.getElementById('payModalAmount').value) || 0;
    const note = document.getElementById('payModalNote').value.trim();

    if (amount <= 0) { alert('Lütfen geçerli bir ödeme tutarı girin.'); return; }

    try {
      await Api.recordPayment({ customer_id: this.selectedCustomerIdForPayment, amount, note });
      App.closeModal('paymentModal');
      App.showToast('✅ Tahsilat kaydedildi ve makbuz basıldı');
      this.loadCustomers();
    } catch (e) {
      App.showToast('Tahsilat kaydedilemedi', true);
    }
  },

  // 4. Stock Count Audits
  async loadStockSessions() {
    try {
      const list = await Api.getStockSessions();
      const wrap = document.getElementById('stockSessionsList');
      if (!wrap) return;
      wrap.innerHTML = '';
      list.forEach(s => {
        const div = document.createElement('div');
        div.style = 'background:var(--bg-card); padding:10px 14px; border-radius:8px; cursor:pointer;';
        div.onclick = () => this.openStockSessionDetail(s.id, s.title, s.status);
        div.innerHTML = `
          <div style="font-weight:700;">${s.title}</div>
          <div style="font-size:11px; color:var(--text-muted);">${new Date(s.started_at).toLocaleDateString()} • Durum: ${s.status}</div>
        `;
        wrap.appendChild(div);
      });
    } catch (e) {}
  },

  async openNewStockSessionModal() {
    const title = prompt('Sayım Başlığı / Alanı:', `Genel Depo Sayımı - ${new Date().toLocaleDateString()}`);
    if (!title) return;

    try {
      const sess = await Api.createStockSession({ title, count_type: 'FULL', notes: null });
      this.loadStockSessions();
      this.openStockSessionDetail(sess.id, sess.title, 'OPEN');
    } catch (e) {}
  },

  async openStockSessionDetail(id, title, status) {
    this.activeStockSessionId = id;
    const detailArea = document.getElementById('stockSessionDetail');
    if (!detailArea) return;

    detailArea.innerHTML = `
      <div style="display:flex; justify-content:space-between; align-items:center;">
        <div>
          <div style="font-size:18px; font-weight:700;">${title}</div>
          <div style="font-size:12px; color:var(--text-muted);">Durum: ${status}</div>
        </div>
        ${status === 'OPEN' ? `<button class="btn-checkout btn-cash" style="padding:8px 16px; font-size:13px;" onclick="Views.commitStockSession('${id}')">✓ Sayımı Onayla & Stokları Eşitle</button>` : ''}
      </div>

      ${status === 'OPEN' ? `
      <div style="display:flex; gap:10px; margin-top:10px;">
        <input type="text" id="stockScanField" class="form-input" style="flex:1;" placeholder="Sayım için barkod okutun..." onkeydown="if(event.key==='Enter'){Views.scanStockItem('${id}');}">
        <input type="number" id="stockQtyField" class="form-input" style="width:100px;" value="1" placeholder="Adet">
        <button class="btn-top-action" style="background:var(--apple-blue);" onclick="Views.scanStockItem('${id}')">Ekle</button>
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

    this.loadStockItemsList(id);
  },

  async loadStockItemsList(id) {
    try {
      const items = await Api.getStockSessionItems(id);
      const tbody = document.getElementById('stockItemsTableBody');
      if (!tbody) return;
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
    } catch (e) {}
  },

  async scanStockItem(sessionId) {
    const barcode = document.getElementById('stockScanField').value.trim();
    const qty = parseFloat(document.getElementById('stockQtyField').value) || 1.0;
    if (!barcode) return;

    try {
      await Api.scanStockItem(sessionId, barcode, qty);
      document.getElementById('stockScanField').value = '';
      document.getElementById('stockScanField').focus();
      this.loadStockItemsList(sessionId);
    } catch (e) {
      alert('Barkod bulunamadı!');
    }
  },

  async commitStockSession(sessionId) {
    if (!confirm('Sayımı onaylamak ve ürün stoklarını bu miktarlarla güncellemek istediğinize emin misiniz?')) return;
    try {
      await Api.commitStockSession(sessionId);
      App.showToast('✅ Stoklar başarıyla güncellendi');
      this.loadStockSessions();
    } catch (e) {}
  },

  // 5. Waste / Fire Tracking
  async loadWastes() {
    try {
      const list = await Api.getWasteRecords();
      const tbody = document.getElementById('wasteTableBody');
      if (!tbody) return;
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
    } catch (e) {
      App.showToast('Fire listesi alınamadı', true);
    }
  },

  openNewWasteModal() {
    document.getElementById('wasteBarcode').value = '';
    document.getElementById('wasteQuantity').value = '1';
    document.getElementById('wasteProductInfo').innerText = '';
    App.openModal('newWasteModal');
  },

  async lookupWasteProduct() {
    const code = document.getElementById('wasteBarcode').value.trim();
    if (!code) return;
    try {
      const data = await Api.scanBarcode(code);
      if (data.found && data.product) {
        document.getElementById('wasteProductInfo').innerText = `✓ Bulundu: ${data.product.name} (Stok: ${data.product.stock_qty} ${data.product.unit})`;
      } else {
        document.getElementById('wasteProductInfo').innerText = '⚠️ Ürün bulunamadı';
      }
    } catch (e) {}
  },

  async saveWasteRecord() {
    const barcode = document.getElementById('wasteBarcode').value.trim();
    const qty = parseFloat(document.getElementById('wasteQuantity').value) || 0;
    const reason = document.getElementById('wasteReason').value;

    if (!barcode || qty <= 0) {
      App.showToast('Lütfen geçerli barkod ve miktar giriniz!', true);
      return;
    }

    try {
      const record = await Api.recordWaste({ barcode, quantity: qty, reason });
      App.showToast(`🗑️ Fire kaydedildi (${record.product_name}) ve stoktan düşüldü!`);
      App.closeModal('newWasteModal');
      this.loadWastes();
      this.loadProducts();
    } catch (e) {
      App.showToast(`Hata: ${e.message}`, true);
    }
  },

  // 6. Shelf Labels
  async loadShelfLabels() {
    try {
      const labels = await Api.getShelfLabels();
      const grid = document.getElementById('shelfLabelsGrid');
      if (!grid) return;
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
    } catch (e) {
      App.showToast('Etiketler yüklenemedi', true);
    }
  },

  // 7. Cash Movements (Avans / Masraf)
  async loadCashMovements() {
    try {
      const movements = await Api.getCashMovements();
      const tbody = document.getElementById('cashMovementsTableBody');
      if (!tbody) return;
      tbody.innerHTML = '';
      if (!movements || movements.length === 0) {
        tbody.innerHTML = `<tr><td colspan="5" style="text-align:center; padding:24px; color:var(--text-muted);">Henüz bir kasa hareketi kaydedilmedi.</td></tr>`;
        return;
      }

      movements.forEach(m => {
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
    } catch (e) {
      App.showToast('Kasa hareketleri alınamadı', true);
    }
  },

  openCashMovementModal(type) {
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

    App.openModal('cashMovementModal');
  },

  async submitCashMovement() {
    const type = document.getElementById('cashMovementType').value;
    const amount = parseFloat(document.getElementById('cashMovementAmount').value) || 0;
    const reason = document.getElementById('cashMovementReason').value.trim();

    if (amount <= 0 || !reason) {
      App.showToast('Lütfen geçerli tutar ve açıklama giriniz!', true);
      return;
    }

    try {
      await Api.recordCashMovement({ movement_type: type, amount, reason });
      App.showToast(`💵 Kasa hareketi başarıyla işlendi (${amount.toFixed(2)} ₺)`);
      App.closeModal('cashMovementModal');
      this.loadCashMovements();
    } catch (e) {
      App.showToast('Kasa hareketi kaydedilemedi', true);
    }
  },

  // 8. Z-Report
  async fetchAndDisplayZReport() {
    try {
      const z = await Api.getZReport();
      const area = document.getElementById('zReportDisplayArea');
      if (!area) return;

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
    } catch (e) {}
  },

  // ==========================================================
  // 9. MAL KABUL & ALIŞ İRSALİYESİ / TEDARİKÇİLER
  // ==========================================================
  suppliers: [],
  goodsReceipts: [],
  newReceiptItems: [],

  async loadReceiving() {
    try {
      const [supps, receipts] = await Promise.all([
        Api.getSuppliers(),
        Api.getGoodsReceipts()
      ]);
      this.suppliers = supps || [];
      this.goodsReceipts = receipts || [];
      this.renderGoodsReceipts(this.goodsReceipts);
    } catch (e) {
      App.showToast('Mal kabul verileri alınamadı', true);
    }
  },

  renderGoodsReceipts(list) {
    const tbody = document.getElementById('receivingTableBody');
    if (!tbody) return;
    tbody.innerHTML = '';
    if (!list || list.length === 0) {
      tbody.innerHTML = `<tr><td colspan="7" style="text-align:center; padding:24px; color:var(--text-muted);">Kayıtlı mal kabul fişi bulunamadı.</td></tr>`;
      return;
    }

    list.forEach(r => {
      const tr = document.createElement('tr');
      tr.innerHTML = `
        <td><span style="font-family:monospace; font-weight:700; color:var(--apple-blue);">${r.invoice_no}</span></td>
        <td style="color:var(--text-muted);">${new Date(r.created_at).toLocaleString('tr-TR')}</td>
        <td><b>${r.supplier_name}</b></td>
        <td>${r.items ? r.items.length : 0} Kalem</td>
        <td><b style="color:var(--apple-green);">${r.grand_total.toFixed(2)} ₺</b></td>
        <td style="font-size:12px; color:var(--text-muted);">${r.notes || '-'}</td>
        <td style="text-align:right;">
          <button class="btn-top-action" onclick="Views.viewGoodsReceiptDetail('${r.id}')">🔍 Fiş Detayı</button>
        </td>
      `;
      tbody.appendChild(tr);
    });
  },

  openNewSupplierModal() {
    document.getElementById('nsName').value = '';
    document.getElementById('nsContact').value = '';
    document.getElementById('nsPhone').value = '';
    document.getElementById('nsTaxInfo').value = '';
    App.openModal('newSupplierModal');
  },

  async saveNewSupplier() {
    const name = document.getElementById('nsName').value.trim();
    const contact = document.getElementById('nsContact').value.trim();
    const phone = document.getElementById('nsPhone').value.trim();
    const tax_info = document.getElementById('nsTaxInfo').value.trim();

    if (!name) {
      App.showToast('Tedarikçi firma adı zorunludur!', true);
      return;
    }

    try {
      await Api.createSupplier({ name, contact_person: contact || null, phone: phone || null, tax_info: tax_info || null });
      App.showToast(`✅ Tedarikçi kaydedildi: ${name}`);
      App.closeModal('newSupplierModal');
      this.loadReceiving();
    } catch (e) {
      App.showToast('Tedarikçi kaydedilemedi: ' + e.message, true);
    }
  },

  async openNewGoodsReceiptModal() {
    if (!this.suppliers || this.suppliers.length === 0) {
      try {
        this.suppliers = await Api.getSuppliers();
      } catch (e) {}
    }

    const sel = document.getElementById('ngrSupplierSelect');
    if (sel) {
      sel.innerHTML = '';
      if (!this.suppliers || this.suppliers.length === 0) {
        sel.innerHTML = '<option value="">(Tedarikçi Yok - Önce Tedarikçi Ekleyin)</option>';
      } else {
        this.suppliers.forEach(s => {
          sel.innerHTML += `<option value="${s.id}">${s.name}</option>`;
        });
      }
    }

    const randNo = `IRS-${new Date().getFullYear()}-${Math.floor(1000 + Math.random() * 9000)}`;
    document.getElementById('ngrInvoiceNo').value = randNo;
    document.getElementById('ngrNotes').value = '';
    document.getElementById('ngrBarcodeField').value = '';
    document.getElementById('ngrQtyField').value = '1';
    document.getElementById('ngrCostField').value = '';
    document.getElementById('ngrItemLookupStatus').innerText = '';

    this.newReceiptItems = [];
    this.renderNewGoodsReceiptItems();
    App.openModal('newGoodsReceiptModal');
  },

  async lookupAndAddReceivingItem() {
    const code = document.getElementById('ngrBarcodeField').value.trim();
    const qty = parseFloat(document.getElementById('ngrQtyField').value) || 1;
    let cost = parseFloat(document.getElementById('ngrCostField').value) || 0;
    const statusEl = document.getElementById('ngrItemLookupStatus');

    if (!code) {
      if (statusEl) statusEl.innerText = 'Lütfen barkod okutun veya ürün adı yazın';
      return;
    }

    // 1. Önce lokal ürünlerde ara
    let prod = this.products.find(p => p.barcode === code || p.id === code || p.name.toLowerCase().includes(code.toLowerCase()));
    
    // 2. Yoksa 4.13M Ulusal Katalogda ara
    if (!prod) {
      try {
        const cat = await Api.searchNationalCatalog(code);
        if (cat && cat.found) {
          prod = {
            id: 'NEW',
            barcode: cat.barcode,
            name: cat.name,
            price: cat.suggested_price || 0,
            vat_rate: cat.vat_rate || 10,
            unit: cat.unit || 'adet',
            is_weighed: cat.is_weighed || false
          };
          if (statusEl) statusEl.innerText = `✨ Ulusal Katalog: ${cat.name} (${cat.brand || 'Yerli Üretim'})`;
        }
      } catch (e) {}
    }

    if (!prod) {
      if (statusEl) statusEl.innerText = `⚠️ Ürün bulunamadı: ${code}. Önce Ürünler sayfasından tanımlayın.`;
      return;
    }

    if (cost <= 0) {
      // Varsayılan tahmini maliyet: satış fiyatının %75'i
      cost = prod.price > 0 ? Math.round(prod.price * 0.75 * 100) / 100 : 10.0;
    }

    this.newReceiptItems.push({
      product_id: prod.id,
      barcode: prod.barcode,
      name: prod.name,
      quantity: qty,
      purchase_cost: cost,
      total_cost: qty * cost
    });

    document.getElementById('ngrBarcodeField').value = '';
    document.getElementById('ngrCostField').value = '';
    document.getElementById('ngrQtyField').value = '1';
    if (statusEl) statusEl.innerText = `✅ Eklendi: ${prod.name}`;

    this.renderNewGoodsReceiptItems();
  },

  renderNewGoodsReceiptItems() {
    const tbody = document.getElementById('ngrItemsTableBody');
    if (!tbody) return;
    tbody.innerHTML = '';
    let grand = 0;

    this.newReceiptItems.forEach((item, idx) => {
      grand += item.total_cost;
      const tr = document.createElement('tr');
      tr.innerHTML = `
        <td><b>${item.name}</b></td>
        <td><span style="font-family:monospace; color:var(--text-muted);">${item.barcode}</span></td>
        <td>${item.quantity}</td>
        <td>${item.purchase_cost.toFixed(2)} ₺</td>
        <td><b>${item.total_cost.toFixed(2)} ₺</b></td>
        <td style="text-align:center;">
          <button class="qty-btn" style="background:transparent; border:none; color:var(--apple-red);" onclick="Views.removeReceivingItem(${idx})">×</button>
        </td>
      `;
      tbody.appendChild(tr);
    });

    const elTotal = document.getElementById('ngrGrandTotalText');
    if (elTotal) elTotal.innerText = `${grand.toFixed(2)} ₺`;
  },

  removeReceivingItem(idx) {
    this.newReceiptItems.splice(idx, 1);
    this.renderNewGoodsReceiptItems();
  },

  async submitGoodsReceipt() {
    const supplier_id = document.getElementById('ngrSupplierSelect').value;
    const invoice_no = document.getElementById('ngrInvoiceNo').value.trim();
    const notes = document.getElementById('ngrNotes').value.trim();

    if (!supplier_id) {
      App.showToast('Lütfen tedarikçi firma seçin!', true);
      return;
    }
    if (!invoice_no) {
      App.showToast('İrsaliye / Fatura No zorunludur!', true);
      return;
    }
    if (this.newReceiptItems.length === 0) {
      App.showToast('En az bir mal kabul kalemi eklemelisiniz!', true);
      return;
    }

    try {
      await Api.createGoodsReceipt({
        supplier_id,
        invoice_no,
        notes: notes || null,
        items: this.newReceiptItems.map(i => ({
          product_id: i.product_id,
          barcode: i.barcode,
          name: i.name,
          quantity: i.quantity,
          purchase_cost: i.purchase_cost
        }))
      });

      App.showToast(`✅ Mal kabulü tamamlandı! Stoklar ve maliyetler güncellendi.`);
      App.closeModal('newGoodsReceiptModal');
      this.loadReceiving();
      this.loadProducts();
    } catch (e) {
      App.showToast('Mal kabul işlenemedi: ' + e.message, true);
    }
  },

  async viewGoodsReceiptDetail(id) {
    try {
      const d = await Api.getGoodsReceiptDetail(id);
      const headEl = document.getElementById('receiptDetailHeader');
      if (headEl) {
        headEl.innerHTML = `
          <div>
            <div><b>İrsaliye:</b> <span style="font-family:monospace; color:var(--apple-blue);">${d.invoice_no}</span></div>
            <div style="color:var(--text-muted); font-size:12px;">Tedarikçi: ${d.supplier_name}</div>
          </div>
          <div style="text-align:right;">
            <div><b>Toplam Tutar:</b> <span style="color:var(--apple-green); font-weight:800;">${d.grand_total.toFixed(2)} ₺</span></div>
            <div style="color:var(--text-muted); font-size:12px;">${new Date(d.created_at).toLocaleString('tr-TR')}</div>
          </div>
        `;
      }

      const tbody = document.getElementById('receiptDetailItemsBody');
      if (tbody) {
        tbody.innerHTML = '';
        (d.items || []).forEach(i => {
          tbody.innerHTML += `
            <tr>
              <td><b>${i.name}</b></td>
              <td><span style="font-family:monospace; color:var(--text-muted);">${i.barcode}</span></td>
              <td>${i.quantity}</td>
              <td>${i.purchase_cost.toFixed(2)} ₺</td>
              <td><b>${i.total_cost.toFixed(2)} ₺</b></td>
            </tr>
          `;
        });
      }

      App.openModal('goodsReceiptDetailModal');
    } catch (e) {
      App.showToast('İrsaliye detayı alınamadı', true);
    }
  },

  // ==========================================================
  // 10. KAMPANYALAR & İNDİRİMLER
  // ==========================================================
  campaigns: [],

  async loadCampaigns() {
    try {
      this.campaigns = await Api.getCampaigns();
      this.renderCampaigns(this.campaigns);
    } catch (e) {
      App.showToast('Kampanyalar listelenemedi', true);
    }
  },

  renderCampaigns(list) {
    const tbody = document.getElementById('campaignsTableBody');
    if (!tbody) return;
    tbody.innerHTML = '';

    if (!list || list.length === 0) {
      tbody.innerHTML = `<tr><td colspan="7" style="text-align:center; padding:24px; color:var(--text-muted);">Aktif kampanya bulunamadı.</td></tr>`;
      return;
    }

    list.forEach(c => {
      const tr = document.createElement('tr');
      const typeLabel = c.campaign_type === 'PERCENT'
        ? `%${c.discount_value} Yüzde İndirimi`
        : (c.campaign_type === 'AMOUNT' ? `${c.discount_value.toFixed(2)} ₺ Sabit İndirim` : `${c.buy_x || 3} Al ${c.pay_y || 2} Öde Promosyonu`);

      const statusBadge = c.is_active
        ? '<span class="pill-badge" style="color:var(--apple-green);">🟢 Aktif</span>'
        : '<span class="pill-badge offline">⚪ Pasif</span>';

      tr.innerHTML = `
        <td><b style="font-family:monospace; color:#38bdf8; font-size:14px;">${c.code}</b></td>
        <td><b>${c.name}</b></td>
        <td>${typeLabel}</td>
        <td><span style="font-weight:700;">${c.discount_value > 0 ? c.discount_value : '-'}</span></td>
        <td>${c.min_subtotal > 0 ? c.min_subtotal.toFixed(2) + ' ₺' : 'Yok'}</td>
        <td>${statusBadge}</td>
        <td style="text-align:right;">
          <button class="btn-top-action" onclick="Views.toggleCampaign('${c.id}')">${c.is_active ? '⏸️ Durdur' : '▶️ Başlat'}</button>
          <button class="btn-top-action" style="color:var(--apple-red);" onclick="Views.deleteCampaign('${c.id}')">🗑️ Sil</button>
        </td>
      `;
      tbody.appendChild(tr);
    });
  },

  openNewCampaignModal() {
    document.getElementById('ncCode').value = '';
    document.getElementById('ncName').value = '';
    document.getElementById('ncType').value = 'PERCENT';
    document.getElementById('ncValue').value = '10.0';
    document.getElementById('ncMinSubtotal').value = '0.0';
    App.openModal('newCampaignModal');
  },

  async saveNewCampaign() {
    const code = document.getElementById('ncCode').value.trim().toUpperCase();
    const name = document.getElementById('ncName').value.trim();
    const campaign_type = document.getElementById('ncType').value;
    const discount_value = parseFloat(document.getElementById('ncValue').value) || 0;
    const min_subtotal = parseFloat(document.getElementById('ncMinSubtotal').value) || 0;

    if (!code || !name) {
      App.showToast('Kampanya kodu ve başlığı zorunludur!', true);
      return;
    }

    try {
      await Api.createCampaign({
        code,
        name,
        campaign_type,
        discount_value,
        min_subtotal,
        buy_x: campaign_type === 'BUY_X_PAY_Y' ? 3 : null,
        pay_y: campaign_type === 'BUY_X_PAY_Y' ? 2 : null,
        start_date: null,
        end_date: null
      });

      App.showToast(`🎉 Kampanya oluşturuldu: ${code}`);
      App.closeModal('newCampaignModal');
      this.loadCampaigns();
    } catch (e) {
      App.showToast('Kampanya oluşturulamadı: ' + e.message, true);
    }
  },

  async toggleCampaign(id) {
    try {
      await Api.toggleCampaign(id);
      this.loadCampaigns();
    } catch (e) {
      App.showToast('Kampanya güncellenemedi', true);
    }
  },

  async deleteCampaign(id) {
    if (!confirm('Bu kampanyayı silmek istediğinizden emin misiniz?')) return;
    try {
      await Api.deleteCampaign(id);
      App.showToast('Kampanya silindi');
      this.loadCampaigns();
    } catch (e) {
      App.showToast('Kampanya silinemedi', true);
    }
  },

  async applyCampaignCode() {
    const codeInput = document.getElementById('posCampaignCodeInput');
    const badge = document.getElementById('appliedCampaignBadge');
    if (!codeInput) return;
    const code = codeInput.value.trim().toUpperCase();
    if (!code) {
      Cart.clearCampaign();
      if (badge) { badge.style.display = 'none'; badge.innerText = ''; }
      return;
    }

    const res = await Cart.applyCampaign(code);
    if (res.success) {
      App.showToast(`🎉 Kampanya uygulandı! (${res.discount.toFixed(2)} ₺ indirim)`);
      if (badge) {
        badge.style.display = 'inline-flex';
        badge.innerText = `🎉 -${res.discount.toFixed(2)} ₺`;
      }
    } else {
      App.showToast(`⚠️ ${res.message}`, true);
      if (badge) { badge.style.display = 'none'; badge.innerText = ''; }
    }
  },

  // ==========================================================
  // 11. ŞUBELER & ŞUBELER ARASI MAL TRANSFERİ
  // ==========================================================
  branches: [],
  transfers: [],
  newTransferItems: [],

  async loadTransfers() {
    try {
      const [brs, trfs] = await Promise.all([
        Api.getBranches(),
        Api.getTransfers()
      ]);
      this.branches = brs || [];
      this.transfers = trfs || [];
      this.renderTransfers(this.transfers);
    } catch (e) {
      App.showToast('Şube transferleri alınamadı', true);
    }
  },

  renderTransfers(list) {
    const tbody = document.getElementById('transfersTableBody');
    if (!tbody) return;
    tbody.innerHTML = '';

    if (!list || list.length === 0) {
      tbody.innerHTML = `<tr><td colspan="7" style="text-align:center; padding:24px; color:var(--text-muted);">Kayıtlı transfer sevkıyatı bulunamadı.</td></tr>`;
      return;
    }

    list.forEach(t => {
      const tr = document.createElement('tr');
      const isTransit = t.status === 'IN_TRANSIT';
      const statusBadge = isTransit
        ? '<span class="pill-badge" style="color:var(--apple-amber);">🚚 Yolda / Sevk Edildi</span>'
        : '<span class="pill-badge" style="color:var(--apple-green);">✅ Teslim Alındı</span>';

      tr.innerHTML = `
        <td><b style="font-family:monospace; color:var(--apple-blue);">${t.transfer_code}</b></td>
        <td style="color:var(--text-muted);">${new Date(t.created_at).toLocaleString('tr-TR')}</td>
        <td>${t.from_branch_name}</td>
        <td><b>${t.to_branch_name}</b></td>
        <td>${t.items ? t.items.length : 0} Kalem</td>
        <td>${statusBadge}</td>
        <td style="text-align:right;">
          ${isTransit ? `<button class="btn-top-action" style="color:var(--apple-green); font-weight:700;" onclick="Views.receiveTransfer('${t.id}')">📥 Teslim Alındı Onayla</button>` : '<span style="color:var(--text-muted); font-size:12px;">Tamamlandı</span>'}
        </td>
      `;
      tbody.appendChild(tr);
    });
  },

  async openNewTransferModal() {
    if (!this.branches || this.branches.length === 0) {
      try { this.branches = await Api.getBranches(); } catch (e) {}
    }

    const sel = document.getElementById('ntToBranchSelect');
    if (sel) {
      sel.innerHTML = '';
      this.branches.forEach(b => {
        sel.innerHTML += `<option value="${b.id}">${b.name} (${b.city || 'Merkez'})</option>`;
      });
    }

    document.getElementById('ntNotes').value = '';
    document.getElementById('ntBarcodeField').value = '';
    document.getElementById('ntQtyField').value = '1';
    document.getElementById('ntLookupStatus').innerText = '';
    this.newTransferItems = [];
    this.renderNewTransferItems();
    App.openModal('newTransferModal');
  },

  addTransferItemRow() {
    const code = document.getElementById('ntBarcodeField').value.trim();
    const qty = parseFloat(document.getElementById('ntQtyField').value) || 1;
    const statusEl = document.getElementById('ntLookupStatus');

    if (!code) return;
    const prod = this.products.find(p => p.barcode === code || p.id === code || p.name.toLowerCase().includes(code.toLowerCase()));
    if (!prod) {
      if (statusEl) statusEl.innerText = `⚠️ Ürün bulunamadı: ${code}`;
      return;
    }

    this.newTransferItems.push({
      product_id: prod.id,
      barcode: prod.barcode,
      name: prod.name,
      quantity: qty
    });

    document.getElementById('ntBarcodeField').value = '';
    document.getElementById('ntQtyField').value = '1';
    if (statusEl) statusEl.innerText = `✅ Eklendi: ${prod.name} (${qty} ${prod.unit})`;
    this.renderNewTransferItems();
  },

  renderNewTransferItems() {
    const tbody = document.getElementById('ntItemsTableBody');
    if (!tbody) return;
    tbody.innerHTML = '';
    this.newTransferItems.forEach((item, idx) => {
      tbody.innerHTML += `
        <tr>
          <td><b>${item.name}</b></td>
          <td><span style="font-family:monospace; color:var(--text-muted);">${item.barcode}</span></td>
          <td><b>${item.quantity}</b></td>
          <td style="text-align:center;">
            <button class="qty-btn" style="background:transparent; border:none; color:var(--apple-red);" onclick="Views.removeTransferItemRow(${idx})">×</button>
          </td>
        </tr>
      `;
    });
  },

  removeTransferItemRow(idx) {
    this.newTransferItems.splice(idx, 1);
    this.renderNewTransferItems();
  },

  async submitTransfer() {
    const to_branch_id = document.getElementById('ntToBranchSelect').value;
    const notes = document.getElementById('ntNotes').value.trim();

    if (!to_branch_id) {
      App.showToast('Lütfen hedef şube seçin!', true);
      return;
    }
    if (this.newTransferItems.length === 0) {
      App.showToast('En az bir ürün ekleyin!', true);
      return;
    }

    try {
      await Api.createTransfer({
        to_branch_id,
        notes: notes || null,
        items: this.newTransferItems
      });

      App.showToast('🏢 Şubeler arası sevkıyat başlatıldı, stoktan düşüldü.');
      App.closeModal('newTransferModal');
      this.loadTransfers();
      this.loadProducts();
    } catch (e) {
      App.showToast('Transfer başlatılamadı: ' + e.message, true);
    }
  },

  async receiveTransfer(id) {
    if (!confirm('Bu transferdeki ürünleri şube stoğunuza kabul etmek istiyor musunuz?')) return;
    try {
      await Api.receiveTransfer(id);
      App.showToast('📦 Mallar teslim alındı ve stoğa eklendi!');
      this.loadTransfers();
      this.loadProducts();
    } catch (e) {
      App.showToast('Teslim alınamadı: ' + e.message, true);
    }
  },

  // ==========================================================
  // 12. KASA & VARDİYA / X-RAPORU / KASA SAYIMI
  // ==========================================================
  shiftData: null,

  async loadShiftStatus() {
    try {
      this.shiftData = await Api.getShiftStatus();
      this.renderShiftCard(this.shiftData);
    } catch (e) {
      App.showToast('Vardiya durumu alınamadı', true);
    }
  },

  renderShiftCard(data) {
    const container = document.getElementById('shiftStatusContainer');
    if (!container) return;

    if (!data || !data.active_shift) {
      container.innerHTML = `
        <div style="background:var(--bg-card); border:1px solid var(--border); border-radius:var(--radius-lg); padding:28px; text-align:center;">
          <div style="font-size:40px; margin-bottom:12px;">🔴</div>
          <div style="font-size:20px; font-weight:800; margin-bottom:8px;">Kasa Vardiyası Kapalı</div>
          <div style="font-size:13px; color:var(--text-muted); max-width:440px; margin:0 auto 20px auto;">
            Satış işlemlerine başlamadan önce açılış devir nakdini girerek vardiyanızı başlatabilirsiniz.
          </div>
          <button class="btn-checkout btn-cash" style="max-width:280px; margin:0 auto;" onclick="Views.openShiftModal()">
            🟢 VARDİYAYI BAŞLAT (DEVİR GİR)
          </button>
        </div>
      `;
      return;
    }

    const s = data.active_shift;
    const expected = data.expected_cash || 0;

    container.innerHTML = `
      <div style="display:grid; grid-template-columns: 1fr 1fr; gap:16px;">
        <div style="background:var(--bg-card); border:1px solid var(--border); border-radius:var(--radius-lg); padding:20px;">
          <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:16px;">
            <div style="font-size:16px; font-weight:800; color:var(--text-main);">🟢 Aktif Vardiya Durumu</div>
            <span class="pill-badge" style="color:var(--apple-green);">Açık</span>
          </div>
          <div style="display:flex; flex-direction:column; gap:8px; font-size:13px;">
            <div style="display:flex; justify-content:space-between;">
              <span style="color:var(--text-muted);">Kasiyer:</span>
              <b>${s.cashier_name}</b>
            </div>
            <div style="display:flex; justify-content:space-between;">
              <span style="color:var(--text-muted);">Kasa Kodu:</span>
              <span style="font-family:monospace; font-weight:700;">${s.register_id}</span>
            </div>
            <div style="display:flex; justify-content:space-between;">
              <span style="color:var(--text-muted);">Açılış Saati:</span>
              <span>${new Date(s.opened_at).toLocaleTimeString('tr-TR')}</span>
            </div>
            <div style="display:flex; justify-content:space-between; padding-top:8px; border-top:1px solid var(--border);">
              <span style="color:var(--text-muted);">Açılış Devir Nakdi (Float):</span>
              <b style="color:var(--apple-blue); font-size:16px;">${s.opening_float.toFixed(2)} ₺</b>
            </div>
          </div>
        </div>

        <div style="background:var(--bg-card); border:1px solid var(--border); border-radius:var(--radius-lg); padding:20px; display:flex; flex-direction:column; justify-content:space-between;">
          <div>
            <div style="font-size:12px; color:var(--text-muted); margin-bottom:4px;">Çekmecede Olması Gereken Nakit:</div>
            <div style="font-size:32px; font-weight:900; color:var(--apple-green);">${expected.toFixed(2)} ₺</div>
            <div style="font-size:11px; color:var(--text-muted); margin-top:4px;">
              (Açılış Float + Nakit Satışlar + Kasa Avansı - Masraflar)
            </div>
          </div>
          <div style="display:flex; gap:10px; margin-top:20px;">
            <button class="btn-checkout btn-card" style="flex:1; font-size:13px;" onclick="Views.showXReport()">
              📄 X-Raporu Yazdır
            </button>
            <button class="btn-checkout btn-danger" style="flex:1; font-size:13px;" onclick="Views.openCloseShiftModal()">
              🔴 Vardiyayı Kapat (Sayım)
            </button>
          </div>
        </div>
      </div>
    `;
  },

  openShiftModal() {
    document.getElementById('osOpeningFloat').value = '500.00';
    document.getElementById('osNotes').value = '';
    App.openModal('openShiftModal');
  },

  async submitOpenShift() {
    const floatVal = parseFloat(document.getElementById('osOpeningFloat').value) || 0;
    const cashier = document.getElementById('osCashierName').value.trim() || 'Kasiyer';
    const notes = document.getElementById('osNotes').value.trim();

    try {
      await Api.openShift({ opening_float: floatVal, cashier_name: cashier, notes: notes || null });
      App.showToast(`🟢 Vardiya açıldı. Devir nakit: ${floatVal.toFixed(2)} ₺`);
      App.closeModal('openShiftModal');
      this.loadShiftStatus();
    } catch (e) {
      App.showToast('Vardiya açılamadı: ' + e.message, true);
    }
  },

  openCloseShiftModal() {
    if (!this.shiftData || !this.shiftData.active_shift) return;
    const expected = this.shiftData.expected_cash || 0;
    document.getElementById('csExpectedCashDisplay').innerText = `${expected.toFixed(2)} ₺`;
    document.getElementById('csActualCashInput').value = expected.toFixed(2);
    document.getElementById('csNotesInput').value = '';
    this.updateCloseShiftCalc();
    App.openModal('closeShiftModal');
  },

  updateCloseShiftCalc() {
    const expected = this.shiftData ? (this.shiftData.expected_cash || 0) : 0;
    const actual = parseFloat(document.getElementById('csActualCashInput').value) || 0;
    const diff = actual - expected;
    const diffEl = document.getElementById('csDiscrepancyDisplay');
    if (!diffEl) return;

    if (Math.abs(diff) < 0.05) {
      diffEl.style.color = 'var(--apple-green)';
      diffEl.innerText = '0.00 ₺ (Denk)';
    } else if (diff > 0) {
      diffEl.style.color = 'var(--apple-blue)';
      diffEl.innerText = `+${diff.toFixed(2)} ₺ (Kasa Fazlası)`;
    } else {
      diffEl.style.color = 'var(--apple-red)';
      diffEl.innerText = `${diff.toFixed(2)} ₺ (Kasa Açığı)`;
    }
  },

  async submitCloseShift() {
    const actual = parseFloat(document.getElementById('csActualCashInput').value) || 0;
    const notes = document.getElementById('csNotesInput').value.trim();

    try {
      await Api.closeShift({ actual_cash: actual, notes: notes || null });
      App.showToast('🔴 Vardiya kapatıldı. Gün sonu raporu oluşturuldu.');
      App.closeModal('closeShiftModal');
      this.loadShiftStatus();
      this.fetchAndDisplayZReport();
    } catch (e) {
      App.showToast('Vardiya kapatılamadı: ' + e.message, true);
    }
  },

  async showXReport() {
    try {
      const x = await Api.getXReport();
      const area = document.getElementById('xReportReceiptArea');
      if (area) {
        area.innerHTML = `
          <div style="text-align:center; font-weight:800; font-size:14px; margin-bottom:4px;">QORDY PERAKENDE POS</div>
          <div style="text-align:center; font-size:11px; margin-bottom:12px;">GÜN İÇİ X-RAPORU (ARA KASA)</div>
          <div>Tarih / Saat : ${new Date(x.report_time).toLocaleString('tr-TR')}</div>
          <div>Kasa No      : ${x.register_id}</div>
          <div>Kasiyer      : ${x.cashier_name}</div>
          <div style="border-top:1px dashed #666; margin:8px 0;"></div>
          <div>Açılış Devir Nakdi : ${x.opening_float.toFixed(2)} ₺</div>
          <div>Nakit Satışlar     : ${x.sales_cash_total.toFixed(2)} ₺</div>
          <div>Kart Satışları     : ${x.sales_card_total.toFixed(2)} ₺</div>
          <div>Veresiye Satışlar  : ${x.sales_veresiye_total.toFixed(2)} ₺</div>
          <div>Kasa Avans Girişi  : +${x.cash_in_total.toFixed(2)} ₺</div>
          <div>Masraf Çıkışları   : -${x.cash_out_total.toFixed(2)} ₺</div>
          <div style="border-top:1px dashed #666; margin:8px 0;"></div>
          <div style="font-weight:700; font-size:13px; color:var(--apple-green);">
            KASADA OLMASI GEREKEN NAKİT : ${x.expected_drawer_cash.toFixed(2)} ₺
          </div>
          <div style="font-weight:700; color:#38bdf8;">
            TOPLAM CİRO HİTİMİ         : ${x.grand_sales_total.toFixed(2)} ₺
          </div>
          <div style="text-align:center; margin-top:14px; color:#888;">MALİ DEĞERİ YOKTUR - BİLGİ AMAÇLIDIR</div>
        `;
      }
      App.openModal('xReportModal');
    } catch (e) {
      App.showToast('X-Raporu alınamadı: ' + e.message, true);
    }
  },

  // ==========================================================
  // 13. PARÇALI & KARMA ÖDEME (SPLIT PAYMENTS)
  // ==========================================================
  splitTotalDue: 0.0,

  openSplitPayModal() {
    if (Cart.getItems().length === 0) {
      App.showToast('Sepetiniz boş!', true);
      return;
    }

    this.splitTotalDue = Cart.getTotal();
    document.getElementById('splitTotalDue').innerText = `${this.splitTotalDue.toFixed(2)} ₺`;
    document.getElementById('splitCashAmount').value = '';
    document.getElementById('splitCardAmount').value = '';
    document.getElementById('splitVeresiyeAmount').value = '';

    const sel = document.getElementById('splitCustomerSelect');
    if (sel) {
      sel.innerHTML = '<option value="">-- Müşteri / Cari Seçin --</option>';
      this.customers.forEach(c => {
        sel.innerHTML += `<option value="${c.id}">${c.name} (Borç: ${c.balance.toFixed(2)} ₺)</option>`;
      });
    }

    this.updateSplitCalc();
    App.openModal('splitPayModal');
  },

  updateSplitCalc() {
    const cash = parseFloat(document.getElementById('splitCashAmount').value) || 0;
    const card = parseFloat(document.getElementById('splitCardAmount').value) || 0;
    const veresiye = parseFloat(document.getElementById('splitVeresiyeAmount').value) || 0;

    const paid = cash + card + veresiye;
    const remaining = this.splitTotalDue - paid;
    const remEl = document.getElementById('splitRemainingAmount');
    const submitBtn = document.getElementById('btnSubmitSplitSale');

    if (remEl) {
      if (Math.abs(remaining) < 0.05) {
        remEl.style.color = 'var(--apple-green)';
        remEl.innerText = '0.00 ₺ (Tam)';
        if (submitBtn) submitBtn.disabled = false;
      } else if (remaining > 0) {
        remEl.style.color = 'var(--apple-amber)';
        remEl.innerText = `${remaining.toFixed(2)} ₺ (Kalan)`;
        if (submitBtn) submitBtn.disabled = true;
      } else {
        remEl.style.color = 'var(--apple-red)';
        remEl.innerText = `+${Math.abs(remaining).toFixed(2)} ₺ (Fazla)`;
        if (submitBtn) submitBtn.disabled = true;
      }
    }
  },

  async submitSplitSale() {
    const cash = parseFloat(document.getElementById('splitCashAmount').value) || 0;
    const card = parseFloat(document.getElementById('splitCardAmount').value) || 0;
    const veresiye = parseFloat(document.getElementById('splitVeresiyeAmount').value) || 0;
    const customer_id = document.getElementById('splitCustomerSelect').value || null;

    if (veresiye > 0 && !customer_id) {
      App.showToast('Veresiye ödeme için lütfen bir müşteri seçiniz!', true);
      return;
    }

    const splits = [];
    if (cash > 0) splits.push({ method: 'CASH', amount: cash });
    if (card > 0) splits.push({ method: 'CREDIT_CARD', amount: card });
    if (veresiye > 0) splits.push({ method: 'VERESIYE', amount: veresiye });

    if (splits.length === 0) {
      App.showToast('Lütfen geçerli ödeme tutarı giriniz!', true);
      return;
    }

    try {
      const sale = await Api.checkout({
        items: Cart.getItems(),
        payment_method: 'SPLIT',
        split_payments: splits,
        customer_id: customer_id,
        tender_amount: cash > 0 ? cash : null,
        discount_amount: Cart.discountAmount > 0 ? Cart.discountAmount : null,
        campaign_code: Cart.appliedCampaignCode || null,
        notes: 'Parçalı / Karma Ödeme'
      });

      App.showToast(`✅ Parçalı Satış Onaylandı! Fiş No: ${sale.receipt_no}`);
      App.closeModal('splitPayModal');
      Cart.clear();
      this.loadTransactions();
    } catch (e) {
      App.showToast('Satış işlenemedi: ' + e.message, true);
    }
  },

  // ==========================================================
  // 14. 4.13 MİLYON ULUSAL BARKOD KATALOĞU DESTEĞİ
  // ==========================================================
  async lookupNationalCatalog() {
    const code = document.getElementById('npBarcode').value.trim();
    const badge = document.getElementById('npCatalogMatchBadge');
    if (!code) return;

    try {
      const res = await Api.searchNationalCatalog(code);
      if (res && res.found) {
        if (res.name) document.getElementById('npName').value = res.name;
        if (res.category) document.getElementById('npCategory').value = res.category;
        if (res.vat_rate) document.getElementById('npVat').value = res.vat_rate.toFixed(1);
        if (res.unit) document.getElementById('npUnit').value = res.unit;
        if (res.suggested_price && res.suggested_price > 0) {
          document.getElementById('npPrice').value = res.suggested_price.toFixed(2);
        }
        if (badge) {
          badge.style.display = 'block';
          badge.innerText = `✨ 4.13M Ulusal Katalog: ${res.brand || 'Yerli Üretim'} (${res.name})`;
        }
        App.showToast(`🌟 Ulusal Katalogda Bulundu: ${res.name}`);
      } else {
        if (badge) badge.style.display = 'none';
        App.showToast('Ulusal katalogda bulunamadı, elle girebilirsiniz.');
      }
    } catch (e) {
      if (badge) badge.style.display = 'none';
    }
  }
};
