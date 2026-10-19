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
    statusDiv.innerHTML = '<span style="color:var(--apple-blue);">🔌 Cihaz bağlantısı sınanıyor...</span>';

    try {
      const data = await Api.testTerminal();
      if (data.connected) {
        statusDiv.innerHTML = `<span style="color:var(--apple-green);">✅ BAŞARILI: ${data.mode} erişilebilir ve hazır.</span>`;
      } else {
        statusDiv.innerHTML = `<span style="color:var(--apple-red);">⚠️ BAĞLANILAMADI: ${data.mode} yanıt vermedi. IP/Portu kontrol ediniz.</span>`;
      }
    } catch (e) {
      statusDiv.innerHTML = '<span style="color:var(--apple-red);">Bağlantı testi sırasında hata oluştu.</span>';
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
      App.showToast('✅ POS Cihazı yapılandırması güncellendi!');
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
    document.getElementById('eftPosStatusText').style.color = 'var(--text-main)';
    document.getElementById('eftPosSubtext').innerText = 'Cihazdan provizyon yanıtı bekleniyor';
    document.getElementById('eftPosIcon').className = 'pulse-terminal';
    document.getElementById('eftPosIcon').innerText = '💳';
    document.getElementById('eftPosDetails').style.display = 'none';
    document.getElementById('eftPosCancelBtn').style.display = 'block';

    try {
      const cfg = await Api.getTerminalConfig();
      document.getElementById('eftPosDeviceLabel').innerText = cfg.mode === 'BEKO_TCP'
        ? `Beko GMP3 Terminali (${cfg.host}:${cfg.port})`
        : '🧪 Sanal POS Simülatörü (Test Modu)';
    } catch (e) {}

    App.openModal('eftPosPayModal');

    try {
      const payData = await Api.payTerminal({
        amount: total,
        txn_type: 'SALE',
        order_ref: `POS-${Date.now()}`,
        print_slip: true
      });

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
          App.closeModal('eftPosPayModal');
          await App.executeCheckout('CREDIT_CARD', total, null);
        }, 1200);
      } else {
        throw new Error(payData.error_message || 'İşlem reddedildi');
      }
    } catch (err) {
      document.getElementById('eftPosIcon').className = '';
      document.getElementById('eftPosIcon').innerText = '❌';
      document.getElementById('eftPosStatusText').innerText = 'İŞLEM BAŞARISIZ!';
      document.getElementById('eftPosStatusText').style.color = 'var(--apple-red)';
      document.getElementById('eftPosSubtext').innerText = err.message || 'Cihazdan provizyon alınamadı.';
    }
  }
};
