/**
 * QORDY POS - Enterprise Authentication & Session Management
 * Apple HIG Lock Screen & Cashier PIN / Password Authentication
 * Features:
 *  - Persistent sessions via SQLite & LocalStorage (Mac & Windows)
 *  - Cryptographic salted verification & rate-limiting protection
 *  - Interactive PIN keypad with instant tactile feedback
 *  - Keyboard shortcuts (0-9, Backspace, Enter, F12 to lock)
 *  - Cashier quick avatar picker
 *  - Manager password login fallback
 */

const Auth = {
    tokenKey: 'qordy_pos_token',
    currentSession: null,
    currentUser: null,
    usersList: [],
    enteredPin: '',
    lockoutTimer: null,

    // Initialize Auth module
    async init() {
        this.bindEvents();
        await this.loadUsers();
        await this.checkCurrentSession();
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

    // Load available active cashiers/managers for avatar picker
    async loadUsers() {
        try {
            const res = await fetch('/api/auth/users');
            if (res.ok) {
                this.usersList = await res.json();
                this.renderUserAvatars();
            }
        } catch (e) {
            console.warn('Kullanıcı listesi yüklenemedi:', e);
        }
    },

    renderUserAvatars() {
        const container = document.getElementById('authCashierList');
        if (!container) return;

        if (!this.usersList || this.usersList.length === 0) {
            container.innerHTML = '<div style="color:var(--text-tertiary);font-size:13px;">Kayıtlı kasiyer bulunamadı</div>';
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
                    <div class="auth-avatar-role">${isManager ? 'Yönetici' : 'Kasiyer'}</div>
                </div>
            `;
        }).join('');

        // Click on avatar
        container.querySelectorAll('.auth-avatar-card').forEach(card => {
            card.addEventListener('click', () => {
                container.querySelectorAll('.auth-avatar-card').forEach(c => c.classList.remove('selected'));
                card.classList.add('selected');
                const name = card.dataset.userName;
                const role = card.dataset.role;
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

    // Check existing session
    async checkCurrentSession() {
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
                    }
                    return;
                }
            }
        } catch (e) {
            console.error('Oturum doğrulama hatası:', e);
        }

        // Invalid or expired token
        this.setToken(null);
        this.showLockScreen('login');
    },

    updateActiveUserDisplay(name, role) {
        const nameEl = document.getElementById('hdrCashierName');
        if (nameEl) nameEl.textContent = name || 'Kasiyer';

        const roleBadge = document.getElementById('hdrRoleBadge');
        if (roleBadge) {
            roleBadge.textContent = role === 'MANAGER' ? 'Yönetici' : 'Kasiyer';
            roleBadge.style.display = 'inline-block';
        }

        const avatarInitial = document.getElementById('hdrAvatarInitial');
        if (avatarInitial && name) {
            avatarInitial.textContent = name.substring(0, 1).toUpperCase();
        }
    },

    showLockScreen(mode = 'login') {
        const overlay = document.getElementById('authLockScreen');
        if (!overlay) return;

        overlay.classList.add('active');
        this.clearPin();

        const unlockView = document.getElementById('authUnlockView');
        const loginView = document.getElementById('authLoginView');
        const passView = document.getElementById('authPasswordView');

        if (mode === 'unlock' && this.currentSession) {
            if (unlockView) unlockView.style.display = 'block';
            if (loginView) loginView.style.display = 'none';
            if (passView) passView.style.display = 'none';

            const unlockName = document.getElementById('authUnlockUserName');
            if (unlockName) unlockName.textContent = this.currentSession.user_name;
            const unlockRole = document.getElementById('authUnlockUserRole');
            if (unlockRole) unlockRole.textContent = this.currentSession.role === 'MANAGER' ? 'Yönetici Oturumu' : 'Kasiyer Oturumu';
        } else {
            if (unlockView) unlockView.style.display = 'none';
            if (loginView) loginView.style.display = 'block';
            if (passView) passView.style.display = 'none';
        }
    },

    hideLockScreen() {
        const overlay = document.getElementById('authLockScreen');
        if (overlay) overlay.classList.remove('active');
        this.clearPin();
    },

    appendPinDigit(digit) {
        if (this.enteredPin.length >= 6) return;
        this.enteredPin += digit;
        this.updatePinDots();

        // 4 veya daha fazla hane girildiğinde otomatik doğrulama yapabilir
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

    async submitPin() {
        if (this.enteredPin.length < 4) {
            this.showError('PIN en az 4 haneli olmalıdır.');
            return;
        }

        const pin = this.enteredPin;

        // Is it unlock mode?
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
                    }
                    if (window.showToast) window.showToast('Kilit açıldı', 'success');
                } else {
                    this.showError(data.message || 'Hatalı PIN kodu.');
                }
            } catch (e) {
                this.showError('Bağlantı hatası.');
            }
            return;
        }

        // Login Mode
        try {
            const rememberMe = document.getElementById('authRememberMe')?.checked ?? true;
            const res = await fetch('/api/auth/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ pin, remember_me: rememberMe })
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
                this.hideLockScreen();
                if (window.showToast) window.showToast(`Hoş geldiniz, ${data.user.name}`, 'success');
            } else {
                this.showError(data.error || 'Giriş başarısız. Lütfen tekrar deneyin.');
            }
        } catch (e) {
            this.showError('Sunucu bağlantı hatası.');
        }
    },

    async submitPassword() {
        const usernameInput = document.getElementById('authUsernameInput');
        const passwordInput = document.getElementById('authPasswordInput');
        const username = usernameInput ? usernameInput.value.trim() : '';
        const password = passwordInput ? passwordInput.value : '';

        if (!username || !password) {
            this.showError('Kullanıcı adı ve şifre zorunludur.');
            return;
        }

        try {
            const rememberMe = document.getElementById('authRememberMe')?.checked ?? true;
            const res = await fetch('/api/auth/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password, remember_me: rememberMe })
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
                this.hideLockScreen();
                if (window.showToast) window.showToast(`Yönetici girişi başarılı: ${data.user.name}`, 'success');
            } else {
                this.showError(data.error || 'Geçersiz kullanıcı adı veya şifre.');
            }
        } catch (e) {
            this.showError('Giriş yapılamadı.');
        }
    },

    async lockSession() {
        const token = this.getToken();
        if (token) {
            try {
                await fetch('/api/auth/lock', {
                    method: 'POST',
                    headers: this.getAuthHeaders()
                });
            } catch (e) {
                console.warn('Lock request error:', e);
            }
        }
        if (this.currentSession) {
            this.currentSession.is_locked = true;
        }
        this.showLockScreen('unlock');
        if (window.showToast) window.showToast('Ekran kilitlendi [F12]', 'info');
    },

    async logout() {
        const token = this.getToken();
        if (token) {
            try {
                await fetch('/api/auth/logout', {
                    method: 'POST',
                    headers: this.getAuthHeaders()
                });
            } catch (e) {
                console.warn('Logout error:', e);
            }
        }
        this.setToken(null);
        this.currentSession = null;
        this.updateActiveUserDisplay('Giriş Yapılmadı', '');
        this.showLockScreen('login');
        if (window.showToast) window.showToast('Oturum kapatıldı', 'info');
    },

    bindEvents() {
        // Numpad key clicks
        document.querySelectorAll('.auth-numpad-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const val = btn.dataset.val;
                if (val === 'backspace') {
                    this.backspacePin();
                } else if (val === 'clear') {
                    this.clearPin();
                } else if (val === 'enter') {
                    this.submitPin();
                } else if (val) {
                    this.appendPinDigit(val);
                }
            });
        });

        // Global Keyboard Event
        window.addEventListener('keydown', (e) => {
            // F12 to Lock
            if (e.key === 'F12') {
                e.preventDefault();
                const overlay = document.getElementById('authLockScreen');
                if (overlay && overlay.classList.contains('active')) {
                    // Already locked
                } else {
                    this.lockSession();
                }
                return;
            }

            const overlay = document.getElementById('authLockScreen');
            if (!overlay || !overlay.classList.contains('active')) return;

            // Password view has regular form inputs, don't intercept unless enter
            const passView = document.getElementById('authPasswordView');
            if (passView && passView.style.display !== 'none') {
                if (e.key === 'Enter') {
                    this.submitPassword();
                }
                return;
            }

            // PIN View Key Handlers
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

        // Lock button in UI
        const btnLock = document.getElementById('btnLockScreen');
        if (btnLock) {
            btnLock.addEventListener('click', () => this.lockSession());
        }

        // Logout buttons
        const btnLogout = document.getElementById('btnLogout');
        if (btnLogout) {
            btnLogout.addEventListener('click', () => this.logout());
        }

        const btnSwitchUser = document.getElementById('btnSwitchUser');
        if (btnSwitchUser) {
            btnSwitchUser.addEventListener('click', () => {
                this.showLockScreen('login');
            });
        }

        // Toggle to Password Mode
        const btnTogglePass = document.getElementById('btnTogglePasswordMode');
        if (btnTogglePass) {
            btnTogglePass.addEventListener('click', () => {
                const loginView = document.getElementById('authLoginView');
                const passView = document.getElementById('authPasswordView');
                if (loginView) loginView.style.display = 'none';
                if (passView) passView.style.display = 'block';
                this.clearPin();
            });
        }

        // Back to PIN Mode
        const btnBackToPin = document.getElementById('btnBackToPinMode');
        if (btnBackToPin) {
            btnBackToPin.addEventListener('click', () => {
                const loginView = document.getElementById('authLoginView');
                const passView = document.getElementById('authPasswordView');
                if (loginView) loginView.style.display = 'block';
                if (passView) passView.style.display = 'none';
                this.clearPin();
            });
        }

        // Password submit button
        const btnSubmitPassword = document.getElementById('btnSubmitPassword');
        if (btnSubmitPassword) {
            btnSubmitPassword.addEventListener('click', () => this.submitPassword());
        }
    }
};

window.Auth = Auth;
document.addEventListener('DOMContentLoaded', () => {
    Auth.init();
});
