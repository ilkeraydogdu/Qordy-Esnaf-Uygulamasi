// QORDY POS - Unified Local API Client
const Api = {
  async get(url) {
    const res = await fetch(url);
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },

  async post(url, data) {
    const res = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data)
    });
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },

  async delete(url) {
    const res = await fetch(url, { method: 'DELETE' });
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },

  // System & Status
  getStatus: () => Api.get('/api/status'),
  triggerSync: () => Api.post('/api/sync/trigger', {}),

  // Products & Barcode
  getQuickProducts: () => Api.get('/api/products/quick'),
  getProducts: () => Api.get('/api/products'),
  createProduct: (data) => Api.post('/api/products', data),
  deleteProduct: (id) => Api.delete(`/api/products/${id}`),
  scanBarcode: (barcode) => Api.post('/api/barcode/scan', { barcode }),

  // Cart & Parking
  parkCart: (data) => Api.post('/api/cart/park', data),
  getParkedCarts: () => Api.get('/api/cart/parked'),
  deleteParkedCart: (id) => Api.delete(`/api/cart/parked/${id}`),

  // Checkout & Sales
  checkout: (data) => Api.post('/api/checkout', data),
  getSalesHistory: () => Api.get('/api/sales/history'),
  refundSale: (data) => Api.post('/api/sales/refund', data),

  // Customers & Ledger
  getCustomers: () => Api.get('/api/customers'),
  createCustomer: (data) => Api.post('/api/customers', data),
  recordPayment: (data) => Api.post('/api/customers/payment', data),

  // Stock Count
  getStockSessions: () => Api.get('/api/stock/sessions'),
  createStockSession: (data) => Api.post('/api/stock/sessions', data),
  getStockSessionItems: (id) => Api.get(`/api/stock/sessions/${id}/items`),
  scanStockItem: (id, barcode, qty) => Api.post(`/api/stock/sessions/${id}/scan`, { barcode, qty }),
  commitStockSession: (id) => Api.post(`/api/stock/sessions/${id}/commit`, {}),

  // Waste / Fire
  getWasteRecords: () => Api.get('/api/waste'),
  recordWaste: (data) => Api.post('/api/waste', data),

  // Shelf Labels
  getShelfLabels: () => Api.get('/api/labels'),

  // Cash Movements & Z-Report
  getCashMovements: () => Api.get('/api/cash/movements'),
  recordCashMovement: (data) => Api.post('/api/cash/movements', data),
  getZReport: () => Api.get('/api/reports/z'),

  // Terminal & CFD
  getTerminalConfig: () => Api.get('/api/terminal/config'),
  saveTerminalConfig: (data) => Api.post('/api/terminal/config', data),
  testTerminal: () => Api.post('/api/terminal/test', {}),
  payTerminal: (data) => Api.post('/api/terminal/pay', data),
  updateCfd: (data) => Api.post('/api/cfd/update', data),

  // Mal Kabul & Alış İrsaliyesi / Tedarikçiler
  getSuppliers: () => Api.get('/api/pos/suppliers'),
  createSupplier: (data) => Api.post('/api/pos/suppliers', data),
  getGoodsReceipts: () => Api.get('/api/pos/receiving'),
  getGoodsReceiptDetail: (id) => Api.get(`/api/pos/receiving/${id}`),
  createGoodsReceipt: (data) => Api.post('/api/pos/receiving', data),

  // Kampanyalar & İndirimler
  getCampaigns: () => Api.get('/api/pos/campaigns'),
  createCampaign: (data) => Api.post('/api/pos/campaigns', data),
  toggleCampaign: (id) => Api.post(`/api/pos/campaigns/${id}/toggle`, {}),
  deleteCampaign: (id) => Api.delete(`/api/pos/campaigns/${id}`),
  calculateCampaignDiscount: (code, subtotal, items) => Api.post('/api/pos/campaigns/calculate', { code, subtotal, items }),

  // Şubeler & Şubeler Arası Transfer
  getBranches: () => Api.get('/api/pos/branches'),
  getTransfers: () => Api.get('/api/pos/transfers'),
  createTransfer: (data) => Api.post('/api/pos/transfers', data),
  receiveTransfer: (id) => Api.post(`/api/pos/transfers/${id}/receive`, {}),

  // Kasa Vardiya & X-Raporu
  getShiftStatus: () => Api.get('/api/pos/registers/shift/status'),
  openShift: (data) => Api.post('/api/pos/registers/shift/open', data),
  closeShift: (data) => Api.post('/api/pos/registers/shift/close', data),
  getXReport: () => Api.get('/api/pos/registers/x-report'),

  // 4.13 Milyon Ulusal Barkod Kataloğu
  searchNationalCatalog: (barcode) => Api.get(`/api/pos/national-catalog/${barcode}`)
};
