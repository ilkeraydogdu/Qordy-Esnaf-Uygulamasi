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
  }
};
