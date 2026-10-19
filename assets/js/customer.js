// QORDY Customer Facing Display (CFD) Client
const CustomerDisplay = {
  init() {
    setInterval(() => {
      const el = document.getElementById('clockDisplay');
      if (el) el.innerText = new Date().toLocaleTimeString('tr-TR');
    }, 1000);

    this.connectWs();
    this.pollFallback();
  },

  connectWs() {
    const loc = window.location;
    const uri = (loc.protocol === 'https:' ? 'wss://' : 'ws://') + loc.host + '/ws/pos';
    const ws = new WebSocket(uri);

    ws.onmessage = (e) => {
      try {
        const msg = JSON.parse(e.data);
        if (msg.type === 'CFD_UPDATE') {
          this.handleState(msg.data);
        }
      } catch (err) {}
    };

    ws.onclose = () => setTimeout(() => this.connectWs(), 1500);
  },

  async pollFallback() {
    try {
      const res = await fetch('/api/cfd/state');
      if (res.ok) {
        const data = await res.json();
        this.handleState(data);
      }
    } catch (e) {}
    setTimeout(() => this.pollFallback(), 2000);
  },

  handleState(data) {
    const idle = document.getElementById('idleView');
    const success = document.getElementById('successView');
    const itemsContainer = document.getElementById('itemsContainer');
    if (!idle || !itemsContainer) return;

    if (data.state === 'IDLE' || !data.items || data.items.length === 0) {
      idle.style.display = 'flex';
      if (success) success.style.display = 'none';
      itemsContainer.innerHTML = '';
      return;
    }

    if (data.state === 'PAYMENT_COMPLETED') {
      idle.style.display = 'none';
      if (success) {
        success.style.display = 'flex';
        const changeEl = document.getElementById('successChangeText');
        if (changeEl) {
          changeEl.innerText = data.change > 0 ? `Para Üstü: ${data.change.toFixed(2)} ₺` : 'Ödeme Tamamlandı';
        }
        setTimeout(() => {
          success.style.display = 'none';
          idle.style.display = 'flex';
        }, 6000);
      }
      return;
    }

    // Active Sale
    idle.style.display = 'none';
    if (success) success.style.display = 'none';

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

    const elGrand = document.getElementById('grandTotalDisplay');
    const elSub = document.getElementById('subtotalDisplay');
    if (elGrand) elGrand.innerText = `${data.grand_total.toFixed(2)} ₺`;
    if (elSub) elSub.innerText = `${data.subtotal.toFixed(2)} ₺`;
  }
};

window.addEventListener('DOMContentLoaded', () => {
  CustomerDisplay.init();
});
