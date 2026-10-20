// QORDY POS - POS Terminal & Beko GMP3 Hardware Controller
const Terminal = {
  async openConfigModal() {
    try {
      const cfg = await Api.getTerminalConfig();
      document.getElementById('terminalMode').value = cfg.mode || 'MOCK';
      document.getElementById('terminalHost').value = cfg.host || '192.168.1.150';
      document.getElementById('terminalPort').value = cfg.port || 9000;
      document.getElementById('terminalTimeout').value = cfg.timeout_seconds || 45;
      this.updateFieldsVisibility();
    } catch (e) {}

    document.getElementById('terminalTestStatus').innerText = '';
    App.openModal('terminalConfigModal');
  },

  updateFieldsVisibility() {
    const mode = document.getElementById('terminalMode').value;
    const netFields = document.getElementById('terminalNetworkFields');
    netFields.style.display = mode === 'BEKO_TCP' ? 'flex' : 'none';
  },

  async testConnection() {
    const statusDiv = document.getElementById('terminalTestStatus');
    statusDiv.innerHTML = '<span style="color:var(--accent);">Cihaz bağlantısı sınanıyor...</span>';

    try {
      const data = await Api.testTerminal();
      if (data.connected) {
        statusDiv.innerHTML = `<span style="color:var(--green); font-weight:700;">BAŞARILI: ${data.mode} erişilebilir ve hazır.</span>`;
      } else {
        statusDiv.innerHTML = `<span style="color:var(--red); font-weight:700;">BAĞLANILAMADI: ${data.mode} yanıt vermedi. IP/Portu kontrol ediniz.</span>`;
      }
    } catch (e) {
      statusDiv.innerHTML = '<span style="color:var(--red);">Bağlantı testi sırasında hata oluştu.</span>';
    }
  },

  async saveConfig() {
    const mode = document.getElementById('terminalMode').value;
    const host = document.getElementById('terminalHost').value.trim() || '192.168.1.150';
    const port = parseInt(document.getElementById('terminalPort').value) || 9000;
    const timeout = parseInt(document.getElementById('terminalTimeout').value) || 45;

    try {
      await Api.saveTerminalConfig({
        mode: mode,
        host: host,
        port: port,
        timeout_seconds: timeout
      });
      App.showToast('POS Cihazı yapılandırması güncellendi');
      App.closeModal('terminalConfigModal');
    } catch (e) {
      App.showToast('Ayarlar kaydedilemedi', true);
    }
  },

  async checkoutCard() {
    if (Cart.getItems().length === 0) {
      App.showToast('Sepetiniz boş!', true);
      return;
    }
    const total = Cart.getTotal();

    document.getElementById('eftPosAmountText').innerText = `${total.toFixed(2)} ₺`;
    document.getElementById('eftPosStatusText').innerText = 'Kartınızı Okutunuz veya Şifrenizi Giriniz...';
    document.getElementById('eftPosStatusText').style.color = 'var(--text-primary)';
    document.getElementById('eftPosSubtext').innerText = 'Cihazdan provizyon yanıtı bekleniyor';
    document.getElementById('eftPosIcon').className = 'pulse-terminal';
    document.getElementById('eftPosIcon').innerHTML = '<svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="var(--accent)" stroke-width="1.8"><rect x="1" y="4" width="22" height="16" rx="2" ry="2"></rect><line x1="1" y1="10" x2="23" y2="10"></line></svg>';
    document.getElementById('eftPosDetails').style.display = 'none';
    document.getElementById('eftPosCancelBtn').style.display = 'block';

    try {
      const cfg = await Api.getTerminalConfig();
      document.getElementById('eftPosDeviceLabel').innerText = cfg.mode === 'BEKO_TCP'
        ? `Beko GMP3 Terminali (${cfg.host}:${cfg.port})`
        : 'Sanal POS Simülatörü (Test Modu)';
    } catch (e) {}

    App.openModal('eftPosModal');

    try {
      const payData = await Api.payTerminal({
        amount: total,
        txn_type: 'SALE',
        order_ref: `POS-${Date.now()}`,
        print_slip: true
      });

      if (payData.is_approved) {
        document.getElementById('eftPosIcon').className = '';
        document.getElementById('eftPosIcon').innerHTML = '<svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="var(--green)" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><polyline points="16 9 10 15 7 12"></polyline></svg>';
        document.getElementById('eftPosStatusText').innerText = 'ÖDEME ONAYLANDI!';
        document.getElementById('eftPosStatusText').style.color = 'var(--green)';
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
          App.closeModal('eftPosModal');
          await App.executeCheckout('CREDIT_CARD', total, null);
        }, 1200);
      } else {
        throw new Error(payData.error_message || 'İşlem reddedildi');
      }
    } catch (err) {
      document.getElementById('eftPosIcon').className = '';
      document.getElementById('eftPosIcon').innerHTML = '<svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="var(--red)" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="15" y1="9" x2="9" y2="15"></line><line x1="9" y1="9" x2="15" y2="15"></line></svg>';
      document.getElementById('eftPosStatusText').innerText = 'İŞLEM BAŞARISIZ!';
      document.getElementById('eftPosStatusText').style.color = 'var(--red)';
      document.getElementById('eftPosSubtext').innerText = err.message || 'Cihazdan provizyon alınamadı.';
    }
  }
};
