// QORDY POS - Reactive Shopping Cart Manager
const Cart = {
  items: [],

  init() {
    this.render();
  },

  getItems() {
    return this.items;
  },

  getTotal() {
    return this.items.reduce((acc, i) => acc + i.total, 0);
  },

  addItem(product) {
    if (product.is_weighed) {
      const weightInput = prompt(`${product.name} tartım gramajını girin (Örn: 1.450 veya 1450g):`, "1.000");
      if (!weightInput) return;
      let qty = parseFloat(weightInput.replace(',', '.'));
      if (qty > 50) qty = qty / 1000;

      this.pushItem({
        product_id: product.id,
        barcode: product.barcode,
        name: product.name,
        unit_price: product.price,
        quantity: qty,
        unit: 'kg',
        vat_rate: product.vat_rate,
        discount: 0.0,
        total: qty * product.price,
        is_weighed: true,
        notes: null
      });
    } else {
      const existing = this.items.find(i => i.product_id === product.id && !i.is_weighed);
      if (existing) {
        existing.quantity += 1;
        existing.total = existing.quantity * existing.unit_price;
        this.render();
      } else {
        this.pushItem({
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
  },

  pushItem(item) {
    this.items.push(item);
    this.render();
  },

  changeQty(idx, delta) {
    const item = this.items[idx];
    if (!item) return;

    if (item.is_weighed) item.quantity += delta * 0.100;
    else item.quantity += delta;

    if (item.quantity <= 0.001) this.items.splice(idx, 1);
    else item.total = item.quantity * item.unit_price;

    this.render();
  },

  removeItem(idx) {
    this.items.splice(idx, 1);
    this.render();
  },

  clear() {
    this.items = [];
    this.render();
    this.broadcastCfd('IDLE');
    const field = document.getElementById('posBarcodeField');
    if (field) field.focus();
  },

  render() {
    const tbody = document.getElementById('cartTableBody');
    const emptyMsg = document.getElementById('cartEmptyMsg');
    const countBadge = document.getElementById('cartCountBadge');
    if (!tbody) return;

    tbody.innerHTML = '';
    if (this.items.length === 0) {
      if (emptyMsg) emptyMsg.style.display = 'block';
      if (countBadge) countBadge.innerText = '(0 Kalem)';
    } else {
      if (emptyMsg) emptyMsg.style.display = 'none';
      if (countBadge) countBadge.innerText = `(${this.items.length} Kalem)`;
    }

    let subtotal = 0;
    let vatTotal = 0;

    this.items.forEach((item, idx) => {
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
            <button class="qty-btn" onclick="Cart.changeQty(${idx}, -1)">-</button>
            <span style="min-width:40px; text-align:center; font-weight:600;">${qtyTxt}</span>
            <button class="qty-btn" onclick="Cart.changeQty(${idx}, 1)">+</button>
          </div>
        </td>
        <td style="text-align:right;">${item.unit_price.toFixed(2)}</td>
        <td style="text-align:right; font-weight:700; color:#38bdf8;">${item.total.toFixed(2)}</td>
        <td style="text-align:center;">
          <button class="qty-btn" style="background:transparent; border:none; color:#ef4444;" onclick="Cart.removeItem(${idx})">×</button>
        </td>
      `;
      tbody.appendChild(tr);
    });

    const elSub = document.getElementById('posSubtotal');
    const elVat = document.getElementById('posVat');
    const elGrand = document.getElementById('posGrandTotal');
    if (elSub) elSub.innerText = `${subtotal.toFixed(2)} ₺`;
    if (elVat) elVat.innerText = `${vatTotal.toFixed(2)} ₺`;
    if (elGrand) elGrand.innerText = `${subtotal.toFixed(2)} ₺`;

    this.broadcastCfd(this.items.length === 0 ? 'IDLE' : 'ACTIVE_SALE');
  },

  broadcastCfd(stateType = 'ACTIVE_SALE') {
    const subtotal = this.getTotal();
    const cfdData = {
      state: this.items.length === 0 ? 'IDLE' : stateType,
      items: this.items,
      subtotal: subtotal,
      discount: 0.0,
      grand_total: subtotal,
      tender: 0.0,
      change: 0.0,
      payment_method: null,
      customer_name: null,
      updated_at: new Date().toISOString()
    };
    Api.updateCfd(cfdData).catch(() => {});
  }
};
