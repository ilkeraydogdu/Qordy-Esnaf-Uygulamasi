/**
 * QORDY POS - Enterprise Authentication & Role-Based Access Control (RBAC)
 * Esnaf Perakende Mağaza Aktivasyonu & Hızlı Kasiyer PIN Sistemi
 * 
 * Özellikler:
 * 1. Esnaf Mağaza Aktivasyonu: İlk kurulumda bir kez E-Posta & Şifre ile yetkilendirme.
 * 2. Kalıcı Cihaz Hafızası: Bir kez aktive edildikten sonra tekrar tekrar e-posta/şifre istemez.
 * 3. Hızlı Kasa PIN Girişi: Kasiyer ve Esnaf için 4 haneli dokunmatik/klavye PIN ile 1 saniyede açılış.
 * 4. Rol Bazlı Yetki Sınırlandırması (RBAC):
 *    - Kasiyer (CASHIER): Sadece Kasa Satış (POS) ve İşlemler/Fiş İade erişimi. Diğer sayfalar kilitli.
 *    - Esnaf / Yönetici (MANAGER): Z-Raporu, Stok, Mal Kabul, Fiyatlar, Personel yönetimi tam açık.
 * 5. Kasa Başında Hızlı Kasiyer / Vardiya Değişimi
 * 6. PIN Belirleme & Yeni Kasiyer Ekleme Yönetimi
 */

const Auth = {
    tokenKey: 'qordy_pos_token',
    activatedKey: 'qordy_esnaf_activated',
    storeNameKey: 'qordy_store_name',

    currentSession: null,
    currentUser: null,
    usersList: [],
    enteredPin: '',
    pendingRestrictedScreen: null,

    // 1. Modül Başlatma
    async init() {
        this.bindEvents();
        try {
            const cloudRes = await fetch('/api/cloud/status');
            if (cloudRes.ok) {
                const cloudData = await cloudRes.json();
                if (cloudData.is_connected && cloudData.business_name) {
                    this.setActivated(true, cloudData.business_name);
                }
            }
        } catch (e) {
            console.warn('Bulut durum kontrolü yapılamadı:', e);
        }
        await this.loadUsers();
        await this.checkCurrentSession();
    },

    isActivated() {
        return localStorage.getItem(this.activatedKey) === 'true';
    },

    setActivated(val, storeName) {
        if (val) {
            localStorage.setItem(this.activatedKey, 'true');
            if (storeName) localStorage.setItem(this.storeNameKey, storeName);
        } else {
            localStorage.removeItem(this.activatedKey);
            localStorage.removeItem(this.storeNameKey);
        }
    },

    getStoreName() {
        return localStorage.getItem(this.storeNameKey) || 'KMK Market';
    },

    getToken() {
        return localStorage.getItem(this.tokenKey);
    },

    setToken(token) {
        if (token) {
            localStorage.setItem(this.tokenKey, token);
        } else {
            localStorage.removeItem(this.tokenKey);
        }
    },

    getAuthHeaders() {
        const token = this.getToken();
        return token ? { 'Authorization': 'Bearer ' + token, 'X-Session-Token': token } : {};
    },

    // 2. Kullanıcıları Yükle
    async loadUsers() {
        try {
            const res = await fetch('/api/auth/users');
            if (res.ok) {
                this.usersList = await res.json();
                this.renderUserAvatars();
                this.populateChangePinSelect();
            }
        } catch (e) {
            console.warn('Kullanıcı listesi çekilemedi:', e);
        }
    },

    renderUserAvatars() {
        const container = document.getElementById('authCashierList');
        if (!container) return;

        if (!this.usersList || this.usersList.length === 0) {
            container.innerHTML = '<div style="color:var(--text-tertiary);font-size:13px;">Kayıtlı personel bulunamadı</div>';
            return;
        }

        container.innerHTML = this.usersList.map((u, idx) => {
            const initials = u.name.split(' ').map(n => n[0]).join('').substring(0, 2).toUpperCase();
            const isManager = u.role === 'MANAGER' || u.role === 'ADMIN';
            const badgeColor = isManager ? 'var(--accent-orange)' : 'var(--apple-blue)';
            return `
                <div class="auth-avatar-card ${idx === 0 ? 'selected' : ''}" data-user-id="${u.id}" data-user-name="${u.name}" data-role="${u.role}">
                    <div class="auth-avatar-circle" style="background: linear-gradient(135deg, ${badgeColor}, var(--system-indigo));">
                        <span>${initials}</span>
                    </div>
                    <div class="auth-avatar-name">${u.name}</div>
                    <div class="auth-avatar-role">${isManager ? 'Esnaf (Yönetici)' : 'Kasiyer'}</div>
                </div>
            `;
        }).join('');

        // Tıklama ile avatar seçimi
        container.querySelectorAll('.auth-avatar-card').forEach(card => {
            card.addEventListener('click', () => {
                container.querySelectorAll('.auth-avatar-card').forEach(c => c.classList.remove('selected'));
                card.classList.add('selected');
                const name = card.dataset.userName;
                const targetNameEl = document.getElementById('authTargetUser');
                if (targetNameEl) targetNameEl.textContent = name;
                this.clearPin();
            });
        });

        if (this.usersList.length > 0) {
            const targetNameEl = document.getElementById('authTargetUser');
            if (targetNameEl) targetNameEl.textContent = this.usersList[0].name;
        }
    },

    populateChangePinSelect() {
        const select = document.getElementById('changePinUserSelect');
        if (!select) return;
        select.innerHTML = this.usersList.map(u => `
            <option value="${u.id}">${u.name} (${u.role === 'MANAGER' ? 'Esnaf/Yönetici' : 'Kasiyer'})</option>
        `).join('');
    },

    // 3. Mevcut Oturum Kontrolü
    async checkCurrentSession() {
        // Cihaz hiç aktive edilmemişse ilk aktivasyon ekranını göster
        if (!this.isActivated()) {
            this.showLockScreen('activation');
            return;
        }

        const token = this.getToken();
        if (!token) {
            this.showLockScreen('login');
            return;
        }

        try {
            const res = await fetch('/api/auth/session?token=' + encodeURIComponent(token), {
                headers: this.getAuthHeaders()
            });

            if (res.ok) {
                const data = await res.json();
                if (data.valid && data.session) {
                    this.currentSession = data.session;
                    if (data.session.is_locked) {
                        this.showLockScreen('unlock');
                    } else {
                        this.hideLockScreen();
                        this.updateActiveUserDisplay(data.session.user_name, data.session.role);
                        this.applyRolePermissions(data.session.role);
                    }
                    return;
                }
            }
        } catch (e) {
            console.error('Oturum doğrulama hatası:', e);
        }

        this.setToken(null);
        this.showLockScreen('login');
    },

    updateActiveUserDisplay(name, role) {
        const nameEl = document.getElementById('hdrCashierName');
        if (nameEl) nameEl.textContent = name || 'Kasiyer';

        const roleBadge = document.getElementById('hdrRoleBadge');
        if (roleBadge) {
            roleBadge.textContent = role === 'MANAGER' ? 'Esnaf / Yönetici' : 'Kasiyer';
            roleBadge.style.background = role === 'MANAGER' ? 'rgba(255, 149, 0, 0.15)' : 'rgba(0, 122, 255, 0.12)';
            roleBadge.style.color = role === 'MANAGER' ? 'var(--accent-orange)' : 'var(--apple-blue)';
            roleBadge.style.display = 'inline-block';
        }

        const avatarInitial = document.getElementById('hdrAvatarInitial');
        if (avatarInitial && name) {
            avatarInitial.textContent = name.substring(0, 1).toUpperCase();
        }

        const storeBadge = document.getElementById('authStoreBadge');
        if (storeBadge) {
            storeBadge.textContent = `${this.getStoreName()} • KASA-01`;
        }
    },

    // 4. Rol Bazlı Yetkilendirme (RBAC - Kasiyer Sınırlandırması)
    applyRolePermissions(role) {
        const isManager = role === 'MANAGER' || role === 'ADMIN';

        // Kasiyer için sınırlandırılan ekranlar:
        // Yalnızca 'pos' ve 'transactions' sayfalarına serbestçe girebilir!
        const restrictedScreens = [
            'registers', 'products', 'receiving', 'stock',
            'transfers', 'wastes', 'labels', 'customers',
            'campaigns', 'reports', 'cash'
        ];

        document.querySelectorAll('.nav-item').forEach(item => {
            const clickAttr = item.getAttribute('onclick') || '';
            const match = clickAttr.match(/switchMainScreen\('([^']+)'\)/);
            if (match && match[1]) {
                const screenId = match[1];
                if (restrictedScreens.includes(screenId)) {
                    if (isManager) {
                        item.classList.remove('cashier-restricted');
                        item.removeAttribute('title');
                    } else {
                        item.classList.add('cashier-restricted');
                        item.setAttribute('title', 'Bu işlem için Yönetici / Esnaf yetkisi gereklidir 🔒');
                    }
                }
            }
        });

        // Eğer şu an açık olan ekran kilitliyse ve giriş yapan kasiyerse hemen Kasa Satışına yönlendir!
        if (!isManager) {
            const activeScreen = document.querySelector('.screen-view.active');
            if (activeScreen && activeScreen.id) {
                const currentId = activeScreen.id.replace('screen-', '');
                if (restrictedScreens.includes(currentId)) {
                    if (window.App && window.App.switchMainScreen) {
                        window.App.switchMainScreen('pos');
                    }
                }
            }
        }
    },

    canAccessScreen(screenId) {
        if (!this.currentSession) return false;
        const role = this.currentSession.role;
        if (role === 'MANAGER' || role === 'ADMIN') return true;

        // Kasiyer yalnızca Kasa Satışı ve İşlemler sayfasına doğrudan girebilir
        if (screenId === 'pos' || screenId === 'transactions') return true;

        return false;
    },

    requestManagerAccess(screenId) {
        this.pendingRestrictedScreen = screenId;
        const screenNames = {
            'registers': 'Kasa & Vardiya / X-Raporu',
            'products': 'Ürün & Fiyat Kataloğu',
            'receiving': 'Mal Kabul & İrsaliye',
            'stock': 'Stok Sayımı',
            'transfers': 'Şubeler Arası Transfer',
            'wastes': 'Fire & Zayi Takibi',
            'labels': 'Raf Etiketi Basımı',
            'customers': 'Veresiye Defteri (Cari)',
            'campaigns': 'Kampanyalar & Kuponlar',
            'reports': 'Gün Sonu Z-Raporu & Mali Özet',
            'cash': 'Kasa Avans / Masraf'
        };

        const targetName = screenNames[screenId] || screenId;
        const labelEl = document.getElementById('managerPromptScreenName');
        if (labelEl) labelEl.textContent = targetName;

        const pinInput = document.getElementById('managerAuthPinInput');
        if (pinInput) {
            pinInput.value = '';
            setTimeout(() => pinInput.focus(), 150);
        }

        const errEl = document.getElementById('managerAuthError');
        if (errEl) errEl.style.display = 'none';

        if (window.openModal) window.openModal('managerPinAuthModal');
    },

    async confirmManagerAuth() {
        const pinInput = document.getElementById('managerAuthPinInput');
        const pin = pinInput ? pinInput.value.trim() : '';
        const errEl = document.getElementById('managerAuthError');

        if (!pin || pin.length < 4) {
            if (errEl) {
                errEl.textContent = 'Lütfen en az 4 haneli Yönetici PIN kodunu girin.';
                errEl.style.display = 'block';
            }
            return;
        }

        try {
            const res = await fetch('/api/auth/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ pin, remember_me: false })
            });

            const data = await res.json();
            if (data.success && data.user && (data.user.role === 'MANAGER' || data.user.role === 'ADMIN')) {
                if (window.closeModal) window.closeModal('managerPinAuthModal');
                if (window.showToast) window.showToast(`Yönetici onayı sağlandı: ${data.user.name}`, 'success');
                const target = this.pendingRestrictedScreen;
                this.pendingRestrictedScreen = null;
                if (target && window.App && window.App.switchMainScreen) {
                    // Geçici erişim ver ve ekranı aç
                    window.App.switchMainScreen(target);
                }
            } else {
                if (errEl) {
                    errEl.textContent = 'Hatalı PIN veya bu kullanıcı Yönetici yetkisine sahip değil!';
                    errEl.style.display = 'block';
                }
            }
        } catch (e) {
            if (errEl) {
                errEl.textContent = 'Bağlantı hatası.';
                errEl.style.display = 'block';
            }
        }
    },

    // 5. Ekran Gösterim Modları
    showLockScreen(mode = 'login') {
        const overlay = document.getElementById('authLockScreen');
        if (!overlay) return;

        overlay.classList.add('active');
        this.clearPin();

        const actView = document.getElementById('authActivationView');
        const loginView = document.getElementById('authLoginView');
        const unlockView = document.getElementById('authUnlockView');

        if (mode === 'activation') {
            if (actView) actView.style.display = 'block';
            if (loginView) loginView.style.display = 'none';
            if (unlockView) unlockView.style.display = 'none';
        } else if (mode === 'unlock' && this.currentSession) {
            if (actView) actView.style.display = 'none';
            if (loginView) loginView.style.display = 'none';
            if (unlockView) unlockView.style.display = 'block';

            const unlockName = document.getElementById('authUnlockUserName');
            if (unlockName) unlockName.textContent = this.currentSession.user_name;
            const unlockRole = document.getElementById('authUnlockUserRole');
            if (unlockRole) unlockRole.textContent = this.currentSession.role === 'MANAGER' ? 'Esnaf / Yönetici Kilitlendi' : 'Kasiyer Kilitlendi';
        } else {
            if (actView) actView.style.display = 'none';
            if (loginView) loginView.style.display = 'block';
            if (unlockView) unlockView.style.display = 'none';
            const storeBadge = document.getElementById('authStoreBadge');
            if (storeBadge) storeBadge.textContent = `${this.getStoreName()} • KASA-01`;
        }
    },

    hideLockScreen() {
        const overlay = document.getElementById('authLockScreen');
        if (overlay) overlay.classList.remove('active');
        this.clearPin();
    },

    // 6. PIN Numpad Mantığı
    appendPinDigit(digit) {
        if (this.enteredPin.length >= 6) return;
        this.enteredPin += digit;
        this.updatePinDots();

        // 4 hane dolduğunda otomatik doğrula
        if (this.enteredPin.length === 4) {
            setTimeout(() => this.submitPin(), 120);
        }
    },

    backspacePin() {
        if (this.enteredPin.length > 0) {
            this.enteredPin = this.enteredPin.slice(0, -1);
            this.updatePinDots();
        }
    },

    clearPin() {
        this.enteredPin = '';
        this.updatePinDots();
        const errEl = document.getElementById('authErrorMessage');
        if (errEl) {
            errEl.textContent = '';
            errEl.style.display = 'none';
        }
    },

    updatePinDots() {
        const dots = document.querySelectorAll('.auth-pin-dot');
        dots.forEach((dot, index) => {
            if (index < this.enteredPin.length) {
                dot.classList.add('filled');
            } else {
                dot.classList.remove('filled');
            }
        });
    },

    showError(msg) {
        const errEl = document.getElementById('authErrorMessage');
        if (errEl) {
            errEl.textContent = msg;
            errEl.style.display = 'block';
        }

        const pinBox = document.querySelector('.auth-pin-dots-container');
        if (pinBox) {
            pinBox.classList.add('shake');
            setTimeout(() => pinBox.classList.remove('shake'), 500);
        }
        this.clearPin();
    },

    // 7. PIN ile Giriş / Kilit Açma
    async submitPin() {
        if (this.enteredPin.length < 4) {
            this.showError('PIN en az 4 haneli olmalıdır.');
            return;
        }

        const pin = this.enteredPin;

        // Kilit açma modu
        const unlockView = document.getElementById('authUnlockView');
        if (unlockView && unlockView.style.display !== 'none' && this.getToken()) {
            try {
                const res = await fetch('/api/auth/unlock', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json', ...this.getAuthHeaders() },
                    body: JSON.stringify({ pin })
                });
                const data = await res.json();
                if (data.success) {
                    this.hideLockScreen();
                    if (this.currentSession) {
                        this.currentSession.is_locked = false;
                        this.updateActiveUserDisplay(this.currentSession.user_name, this.currentSession.role);
                        this.applyRolePermissions(this.currentSession.role);
                    }
                    if (window.showToast) window.showToast('Kasa kilidi açıldı', 'success');
                } else {
                    this.showError(data.message || 'Hatalı PIN kodu.');
                }
            } catch (e) {
                this.showError('Bağlantı hatası.');
            }
            return;
        }

        // Günlük Kasa PIN Girişi
        try {
            const res = await fetch('/api/auth/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ pin, remember_me: true })
            });

            const data = await res.json();
            if (data.success && data.token && data.user) {
                this.setToken(data.token);
                this.currentSession = {
                    token: data.token,
                    user_id: data.user.id,
                    user_name: data.user.name,
                    role: data.user.role,
                    is_locked: false
                };
                this.updateActiveUserDisplay(data.user.name, data.user.role);
                this.applyRolePermissions(data.user.role);
                this.hideLockScreen();
                const roleTitle = data.user.role === 'MANAGER' ? 'Esnaf / Yönetici' : 'Kasiyer';
                if (window.showToast) window.showToast(`Hoş geldiniz, ${data.user.name} (${roleTitle})`, 'success');
            } else {
                this.showError(data.error || 'Hatalı PIN kodu. Lütfen tekrar deneyin.');
            }
        } catch (e) {
            this.showError('Sunucu bağlantı hatası.');
        }
    },

    // 8. İlk Esnaf Mağaza Aktivasyonu (E-Posta & Şifre)
    async submitActivation() {
        const emailInput = document.getElementById('activationEmail');
        const passInput = document.getElementById('activationPassword');
        const storeInput = document.getElementById('activationStoreName');
        const errEl = document.getElementById('activationError');

        const username = emailInput ? emailInput.value.trim() : '';
        const password = passInput ? passInput.value : '';
        const storeName = storeInput ? storeInput.value.trim() : 'QORDY Kadıköy Çarşı Esnafı';

        if (!username || !password) {
            if (errEl) {
                errEl.textContent = 'Lütfen e-posta ve şifrenizi giriniz.';
                errEl.style.display = 'block';
            }
            return;
        }

        try {
            const res = await fetch('/api/auth/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password, remember_me: true })
            });

            const data = await res.json();
            if (data.success && data.token && data.user) {
                // Buluttan gelen gerçek mağaza adını al
                let finalStoreName = storeName;
                try {
                    const stRes = await fetch('/api/cloud/status');
                    if (stRes.ok) {
                        const stData = await stRes.json();
                        if (stData.business_name) finalStoreName = stData.business_name;
                    }
                } catch(e) {}

                this.setActivated(true, finalStoreName);
                this.setToken(data.token);
                this.currentSession = {
                    token: data.token,
                    user_id: data.user.id,
                    user_name: data.user.name,
                    role: data.user.role,
                    is_locked: false
                };

                // Başlığı da mağaza adına göre güncelle
                const headerTitle = document.querySelector('.brand-name');
                if (headerTitle) headerTitle.textContent = finalStoreName;

                this.updateActiveUserDisplay(data.user.name, data.user.role);
                this.applyRolePermissions(data.user.role);
                this.hideLockScreen();
                await this.loadUsers();

                if (window.App) {
                    if (window.App.loadCatalog) window.App.loadCatalog();
                    if (window.App.fetchStatus) window.App.fetchStatus();
                }
                if (window.Views) {
                    if (window.Views.loadProducts) window.Views.loadProducts();
                    if (window.Views.loadCustomers) window.Views.loadCustomers();
                }

                if (window.showToast) window.showToast(`'${finalStoreName}' Mağazası Başarıyla Aktive Edildi!`, 'success');
            } else {
                if (errEl) {
                    errEl.textContent = data.error || 'Geçersiz e-posta veya şifre.';
                    errEl.style.display = 'block';
                }
            }
        } catch (e) {
            if (errEl) {
                errEl.textContent = 'Aktivasyon sunucusuna bağlanılamadı.';
                errEl.style.display = 'block';
            }
        }
    },

    // 9. PIN Değiştirme
    async submitChangePin() {
        const userSelect = document.getElementById('changePinUserSelect');
        const newPinInput = document.getElementById('newPinInput');
        const confirmPinInput = document.getElementById('newPinConfirmInput');
        const errEl = document.getElementById('changePinError');

        const userId = userSelect ? userSelect.value : null;
        const newPin = newPinInput ? newPinInput.value.trim() : '';
        const confirmPin = confirmPinInput ? confirmPinInput.value.trim() : '';

        if (!newPin || newPin.length < 4) {
            if (errEl) {
                errEl.textContent = 'Yeni PIN en az 4 haneli olmalıdır.';
                errEl.style.display = 'block';
            }
            return;
        }

        if (newPin !== confirmPin) {
            if (errEl) {
                errEl.textContent = 'Girdiğiniz PIN kodları birbiriyle eşleşmiyor.';
                errEl.style.display = 'block';
            }
            return;
        }

        try {
            const res = await fetch('/api/auth/change-pin', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json', ...this.getAuthHeaders() },
                body: JSON.stringify({ user_id: userId, new_pin: newPin })
            });

            const data = await res.json();
            if (data.success) {
                if (window.closeModal) window.closeModal('changePinModal');
                if (window.showToast) window.showToast('PIN kodu başarıyla güncellendi.', 'success');
                newPinInput.value = '';
                confirmPinInput.value = '';
                await this.loadUsers();
            } else {
                if (errEl) {
                    errEl.textContent = data.message || 'PIN güncellenemedi.';
                    errEl.style.display = 'block';
                }
            }
        } catch (e) {
            if (errEl) {
                errEl.textContent = 'Bağlantı hatası.';
                errEl.style.display = 'block';
            }
        }
    },

    // 10. Yeni Kasiyer Ekleme
    async submitNewCashier() {
        const nameInput = document.getElementById('newCashierName');
        const usernameInput = document.getElementById('newCashierUsername');
        const pinInput = document.getElementById('newCashierPin');
        const roleSelect = document.getElementById('newCashierRole');
        const errEl = document.getElementById('newCashierError');

        const name = nameInput ? nameInput.value.trim() : '';
        const username = usernameInput ? usernameInput.value.trim() : '';
        const pin = pinInput ? pinInput.value.trim() : '';
        const role = roleSelect ? roleSelect.value : 'CASHIER';

        if (!name || !username || !pin || pin.length < 4) {
            if (errEl) {
                errEl.textContent = 'Lütfen tüm alanları doldurun ve en az 4 haneli PIN girin.';
                errEl.style.display = 'block';
            }
            return;
        }

        try {
            const res = await fetch('/api/auth/register-cashier', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json', ...this.getAuthHeaders() },
                body: JSON.stringify({ name, username, pin, role })
            });

            const data = await res.json();
            if (data.success) {
                if (window.closeModal) window.closeModal('newCashierModal');
                if (window.showToast) window.showToast(`Yeni personel eklendi: ${name}`, 'success');
                nameInput.value = '';
                usernameInput.value = '';
                pinInput.value = '';
                await this.loadUsers();
            } else {
                if (errEl) {
                    errEl.textContent = data.message || 'Kasiyer eklenemedi.';
                    errEl.style.display = 'block';
                }
            }
        } catch (e) {
            if (errEl) {
                errEl.textContent = 'Bağlantı hatası.';
                errEl.style.display = 'block';
            }
        }
    },

    // 11. Oturum Kilitleme & Çıkış
    async lockSession() {
        const token = this.getToken();
        if (token) {
            try {
                await fetch('/api/auth/lock', {
                    method: 'POST',
                    headers: this.getAuthHeaders()
                });
            } catch (e) {}
        }
        if (this.currentSession) {
            this.currentSession.is_locked = true;
        }
        this.showLockScreen('unlock');
        if (window.showToast) window.showToast('Kasa kilitlendi [F12]', 'info');
    },

    async logout() {
        const token = this.getToken();
        if (token) {
            try {
                await fetch('/api/auth/logout', {
                    method: 'POST',
                    headers: this.getAuthHeaders()
                });
            } catch (e) {}
        }
        this.setToken(null);
        this.currentSession = null;
        this.updateActiveUserDisplay('Giriş Yapılmadı', '');
        this.showLockScreen('login');
        if (window.showToast) window.showToast('Oturum kapatıldı', 'info');
    },

    resetActivation() {
        if (confirm('Cihaz aktivasyonunu sıfırlayıp farklı bir esnaf hesabı ile giriş yapmak istiyor musunuz?')) {
            this.setActivated(false);
            this.setToken(null);
            this.currentSession = null;
            this.showLockScreen('activation');
            if (window.showToast) window.showToast('Cihaz sıfırlandı. Lütfen e-posta ile giriş yapın.', 'info');
        }
    },

    // 12. Olay Dinleyicileri
    bindEvents() {
        // Numpad butonları
        document.querySelectorAll('.auth-numpad-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const val = btn.dataset.val;
                if (val === 'backspace') {
                    this.backspacePin();
                } else if (val === 'clear') {
                    this.clearPin();
                } else if (val) {
                    this.appendPinDigit(val);
                }
            });
        });

        // Klavye kısayolları
        window.addEventListener('keydown', (e) => {
            if (e.key === 'F12') {
                e.preventDefault();
                const overlay = document.getElementById('authLockScreen');
                if (!overlay || !overlay.classList.contains('active')) {
                    this.lockSession();
                }
                return;
            }

            const overlay = document.getElementById('authLockScreen');
            if (!overlay || !overlay.classList.contains('active')) return;

            // Aktivasyon ekranında input alanları serbest olsun
            const actView = document.getElementById('authActivationView');
            if (actView && actView.style.display !== 'none') {
                if (e.key === 'Enter') this.submitActivation();
                return;
            }

            // PIN ekranı tuşları
            if (e.key >= '0' && e.key <= '9') {
                this.appendPinDigit(e.key);
            } else if (e.key === 'Backspace') {
                this.backspacePin();
            } else if (e.key === 'Escape') {
                this.clearPin();
            } else if (e.key === 'Enter') {
                this.submitPin();
            }
        });

        // Üst bar butonları
        document.getElementById('btnLockScreen')?.addEventListener('click', () => this.lockSession());
        document.getElementById('btnLogout')?.addEventListener('click', () => this.logout());
        document.getElementById('btnSidebarSwitchUser')?.addEventListener('click', () => {
            this.showLockScreen('login');
        });

        // Kilit ekranı "Kullanıcı Değiştir"
        document.getElementById('btnSwitchUser')?.addEventListener('click', () => {
            this.showLockScreen('login');
        });

        // Aktivasyon butonu
        document.getElementById('btnSubmitActivation')?.addEventListener('click', () => this.submitActivation());

        // Cihaz Sıfırlama
        document.getElementById('btnResetActivation')?.addEventListener('click', () => this.resetActivation());

        // PIN Değiştirme Modalı aç/kapat
        document.getElementById('btnOpenChangePinModal')?.addEventListener('click', () => {
            if (window.openModal) window.openModal('changePinModal');
        });
        document.getElementById('btnSubmitChangePin')?.addEventListener('click', () => this.submitChangePin());

        // Yeni Kasiyer Modalı aç/kapat
        document.getElementById('btnOpenNewCashierModal')?.addEventListener('click', () => {
            if (window.openModal) window.openModal('newCashierModal');
        });
        document.getElementById('btnSubmitNewCashier')?.addEventListener('click', () => this.submitNewCashier());

        // Yönetici PIN Onay butonu
        document.getElementById('btnConfirmManagerAuth')?.addEventListener('click', () => this.confirmManagerAuth());
    }
};

window.Auth = Auth;
document.addEventListener('DOMContentLoaded', () => {
    Auth.init();
});
