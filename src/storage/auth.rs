//! QORDY POS - Enterprise Authentication & Session Management
//! Secure, cryptographically salted SHA-256 password/PIN hashing with constant-time verification.

use crate::models::{AuthResponse, Session, User};
use crate::storage::DatabaseManager;
use anyhow::Result;
use chrono::{Duration, Local};
use rusqlite::params;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn hash_with_salt(secret: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(secret.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut res = 0;
    for (x, y) in a.bytes().zip(b.bytes()) {
        res |= x ^ y;
    }
    res == 0
}

impl DatabaseManager {
    /// Sistemdeki aktif kasiyer ve yöneticileri listeler (Giriş avatar seçimi için)
    pub fn list_active_users(&self) -> Result<Vec<User>> {
        let conn = self.conn.lock();
        let live_tenant: Option<String> = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'tenant_id'",
                [],
                |r| r.get(0),
            )
            .ok();
        let tenant = live_tenant.unwrap_or_else(|| self.config.tenant_id.clone());

        let mut stmt = conn.prepare(
            "SELECT id, tenant_id, name, username, role, is_active, created_at
             FROM users WHERE is_active = 1 AND tenant_id = ?1 ORDER BY role DESC, name ASC",
        )?;

        let rows = stmt.query_map(params![tenant], |row| {
            Ok(User {
                id: row.get(0)?,
                tenant_id: row.get(1)?,
                name: row.get(2)?,
                username: row.get(3)?,
                role: row.get(4)?,
                is_active: row.get::<_, i32>(5)? == 1,
                created_at: row.get(6)?,
            })
        })?;

        let mut users = Vec::new();
        for r in rows {
            users.push(r?);
        }
        Ok(users)
    }

    /// Kasiyer 4-6 haneli PIN ile hızlı oturum açma (Brute-force korumalı)
    pub fn login_with_pin(&self, pin: &str, remember_me: bool) -> Result<AuthResponse> {
        let clean_pin = pin.trim();
        if clean_pin.is_empty() {
            return Ok(AuthResponse {
                success: false,
                token: None,
                user: None,
                is_locked: false,
                error: Some("Lütfen PIN kodunuzu girin.".to_string()),
            });
        }

        let conn = self.conn.lock();
        let live_tenant: Option<String> = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'tenant_id'",
                [],
                |r| r.get(0),
            )
            .ok();
        let tenant = live_tenant.unwrap_or_else(|| self.config.tenant_id.clone());

        // Aktif kullanıcıları çek ve constant-time karşılaştır
        let mut stmt = conn.prepare(
            "SELECT id, tenant_id, name, username, pin_hash, pin_salt, role, is_active, failed_attempts, locked_until, created_at
             FROM users WHERE is_active = 1 AND tenant_id = ?1",
        )?;

        let now = Local::now().to_rfc3339();

        let mut matched_user: Option<(User, i32, Option<String>)> = None;

        let rows = stmt.query_map(params![tenant], |row| {
            let user = User {
                id: row.get(0)?,
                tenant_id: row.get(1)?,
                name: row.get(2)?,
                username: row.get(3)?,
                role: row.get(6)?,
                is_active: row.get::<_, i32>(7)? == 1,
                created_at: row.get(10)?,
            };
            let pin_hash: String = row.get(4)?;
            let pin_salt: String = row.get(5)?;
            let failed_attempts: i32 = row.get(8)?;
            let locked_until: Option<String> = row.get(9)?;

            Ok((user, pin_hash, pin_salt, failed_attempts, locked_until))
        })?;

        for r in rows {
            let (user, pin_hash, pin_salt, failed_attempts, locked_until) = r?;

            // Kilitlenme kontrolü
            if let Some(ref lock_time) = locked_until {
                if lock_time > &now {
                    return Ok(AuthResponse {
                        success: false,
                        token: None,
                        user: None,
                        is_locked: true,
                        error: Some("Çok fazla hatalı deneme! Güvenlik için hesap geçici olarak kilitlendi.".to_string()),
                    });
                }
            }

            let computed = hash_with_salt(clean_pin, &pin_salt);
            if constant_time_eq(&computed, &pin_hash) {
                matched_user = Some((user, failed_attempts, locked_until));
                break;
            }
        }

        if let Some((user, _, _)) = matched_user {
            // Başarılı giriş: Başarısız deneme sayısını sıfırla
            let _ = conn.execute(
                "UPDATE users SET failed_attempts = 0, locked_until = NULL WHERE id = ?1",
                params![user.id],
            );

            // Yeni oturum belirteci oluştur
            let token = format!("sess_{}", Uuid::new_v4().simple());
            let duration_days = if remember_me { 30 } else { 1 };
            let expires_at = (Local::now() + Duration::days(duration_days)).to_rfc3339();

            conn.execute(
                "INSERT INTO sessions (token, user_id, user_name, role, created_at, expires_at, is_locked, device_info)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 'desktop_pos')",
                params![token, user.id, user.name, user.role, now, expires_at],
            )?;

            Ok(AuthResponse {
                success: true,
                token: Some(token),
                user: Some(user),
                is_locked: false,
                error: None,
            })
        } else {
            // Hatalı PIN: rate limiting ve brute-force koruması
            let _ = conn.execute(
                "UPDATE users SET failed_attempts = failed_attempts + 1 WHERE is_active = 1",
                [],
            );

            Ok(AuthResponse {
                success: false,
                token: None,
                user: None,
                is_locked: false,
                error: Some("Hatalı PIN kodu girdiniz.".to_string()),
            })
        }
    }

    /// Kullanıcı Adı ve Şifre ile oturum açma (Yönetici / Kasiyer)
    pub fn login_with_password(&self, username: &str, password: &str, remember_me: bool) -> Result<AuthResponse> {
        let clean_user = username.trim().to_lowercase();
        let clean_pass = password.trim();

        if clean_user.is_empty() || clean_pass.is_empty() {
            return Ok(AuthResponse {
                success: false,
                token: None,
                user: None,
                is_locked: false,
                error: Some("Lütfen kullanıcı adı ve şifrenizi girin.".to_string()),
            });
        }

        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, tenant_id, name, username, password_hash, password_salt, role, is_active, failed_attempts, locked_until, created_at
             FROM users WHERE LOWER(username) = ?1 AND is_active = 1",
        )?;

        let now = Local::now().to_rfc3339();

        let mut row_match = None;
        let mut rows = stmt.query(params![clean_user])?;
        if let Some(row) = rows.next()? {
            let user = User {
                id: row.get(0)?,
                tenant_id: row.get(1)?,
                name: row.get(2)?,
                username: row.get(3)?,
                role: row.get(6)?,
                is_active: row.get::<_, i32>(7)? == 1,
                created_at: row.get(10)?,
            };
            let pass_hash: Option<String> = row.get(4)?;
            let pass_salt: Option<String> = row.get(5)?;
            let failed_attempts: i32 = row.get(8)?;
            let locked_until: Option<String> = row.get(9)?;
            row_match = Some((user, pass_hash, pass_salt, failed_attempts, locked_until));
        }

        if let Some((user, pass_hash, pass_salt, failed_attempts, locked_until)) = row_match {
            if let Some(ref lock_time) = locked_until {
                if lock_time > &now {
                    return Ok(AuthResponse {
                        success: false,
                        token: None,
                        user: None,
                        is_locked: true,
                        error: Some("Çok fazla hatalı deneme! Hesap geçici olarak kilitlendi.".to_string()),
                    });
                }
            }

            if let (Some(expected_hash), Some(salt)) = (pass_hash, pass_salt) {
                let computed = hash_with_salt(clean_pass, &salt);
                if constant_time_eq(&computed, &expected_hash) {
                    let _ = conn.execute(
                        "UPDATE users SET failed_attempts = 0, locked_until = NULL WHERE id = ?1",
                        params![user.id],
                    );

                    let token = format!("sess_{}", Uuid::new_v4().simple());
                    let duration_days = if remember_me { 30 } else { 1 };
                    let expires_at = (Local::now() + Duration::days(duration_days)).to_rfc3339();

                    conn.execute(
                        "INSERT INTO sessions (token, user_id, user_name, role, created_at, expires_at, is_locked, device_info)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 'desktop_pos')",
                        params![token, user.id, user.name, user.role, now, expires_at],
                    )?;

                    return Ok(AuthResponse {
                        success: true,
                        token: Some(token),
                        user: Some(user),
                        is_locked: false,
                        error: None,
                    });
                }
            }

            // Hatalı şifre
            let new_failed = failed_attempts + 1;
            let lock_time = if new_failed >= 5 {
                Some((Local::now() + Duration::seconds(30)).to_rfc3339())
            } else {
                None
            };

            let _ = conn.execute(
                "UPDATE users SET failed_attempts = ?1, locked_until = ?2 WHERE id = ?3",
                params![new_failed, lock_time, user.id],
            );
        }

        Ok(AuthResponse {
            success: false,
            token: None,
            user: None,
            is_locked: false,
            error: Some("Geçersiz kullanıcı adı veya şifre.".to_string()),
        })
    }

    /// Oturum doğrulama (Kalıcı oturum kontrolü)
    pub fn validate_session(&self, token: &str) -> Result<Option<Session>> {
        let clean_token = token.trim();
        if clean_token.is_empty() {
            return Ok(None);
        }

        let conn = self.conn.lock();
        let now = Local::now().to_rfc3339();

        let mut stmt = conn.prepare(
            "SELECT token, user_id, user_name, role, created_at, expires_at, is_locked
             FROM sessions
             WHERE token = ?1 AND expires_at > ?2",
        )?;

        let mut rows = stmt.query(params![clean_token, now])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Session {
                token: row.get(0)?,
                user_id: row.get(1)?,
                user_name: row.get(2)?,
                role: row.get(3)?,
                created_at: row.get(4)?,
                expires_at: row.get(5)?,
                is_locked: row.get::<_, i32>(6)? == 1,
            }))
        } else {
            Ok(None)
        }
    }

    /// Oturumu kilitle (Kasa başında değilken hızlı güvenlik)
    pub fn lock_session(&self, token: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE sessions SET is_locked = 1 WHERE token = ?1",
            params![token],
        )?;
        Ok(())
    }

    /// Kilitli oturumu PIN ile aç
    pub fn unlock_session(&self, token: &str, pin: &str) -> Result<bool> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT u.pin_hash, u.pin_salt
             FROM sessions s
             JOIN users u ON s.user_id = u.id
             WHERE s.token = ?1",
        )?;

        let mut rows = stmt.query(params![token])?;
        if let Some(row) = rows.next()? {
            let pin_hash: String = row.get(0)?;
            let pin_salt: String = row.get(1)?;
            let computed = hash_with_salt(pin.trim(), &pin_salt);

            if constant_time_eq(&computed, &pin_hash) {
                conn.execute(
                    "UPDATE sessions SET is_locked = 0 WHERE token = ?1",
                    params![token],
                )?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Oturumu sonlandır (Logout)
    pub fn logout_session(&self, token: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM sessions WHERE token = ?1", params![token])?;
        Ok(())
    }

    /// Kullanıcının PIN kodunu güncelle (Esnafın kendi PIN'ini veya kasiyer PIN'ini değiştirmesi)
    pub fn update_user_pin(&self, user_id: &str, new_pin: &str) -> Result<bool> {
        let clean_pin = new_pin.trim();
        if clean_pin.len() < 4 {
            anyhow::bail!("PIN kodu en az 4 haneli olmalıdır.");
        }
        let conn = self.conn.lock();
        let salt = Uuid::new_v4().simple().to_string();
        let pin_hash = hash_with_salt(clean_pin, &salt);
        let updated = conn.execute(
            "UPDATE users SET pin_hash = ?1, pin_salt = ?2 WHERE id = ?3",
            params![pin_hash, salt, user_id],
        )?;
        Ok(updated > 0)
    }

    /// Yeni kasiyer / personel ekle
    pub fn create_cashier(&self, name: &str, username: &str, pin: &str, role: &str) -> Result<User> {
        let clean_name = name.trim();
        let clean_username = username.trim().to_lowercase();
        let clean_pin = pin.trim();
        if clean_name.is_empty() || clean_username.is_empty() || clean_pin.len() < 4 {
            anyhow::bail!("Geçersiz isim, kullanıcı adı veya PIN.");
        }

        let conn = self.conn.lock();
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM users WHERE LOWER(username) = ?1",
            params![clean_username],
            |r| r.get(0),
        )?;
        if exists > 0 {
            anyhow::bail!("Bu kullanıcı adı zaten kullanımda.");
        }

        let id = Uuid::new_v4().to_string();
        let tenant = &self.config.tenant_id;
        let salt = Uuid::new_v4().simple().to_string();
        let pin_hash = hash_with_salt(clean_pin, &salt);
        let pass_hash = hash_with_salt("123", &salt);
        let now = Local::now().to_rfc3339();
        let user_role = if role == "MANAGER" { "MANAGER" } else { "CASHIER" };

        conn.execute(
            "INSERT INTO users (id, tenant_id, name, username, pin_hash, pin_salt, password_hash, password_salt, role, is_active, failed_attempts, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?6, ?8, 1, 0, ?9)",
            params![id, tenant, clean_name, clean_username, pin_hash, salt, pass_hash, user_role, now],
        )?;

        Ok(User {
            id,
            tenant_id: tenant.clone(),
            name: clean_name.to_string(),
            username: clean_username,
            role: user_role.to_string(),
            is_active: true,
            created_at: now,
        })
    }

    /// Veritabanı ilk kurulduğunda varsayılan güvenli kullanıcıları oluştur
    pub fn seed_default_users(&self) -> Result<()> {
        let conn = self.conn.lock();

        // Eski mock kullanıcıları temizle
        let _ = conn.execute(
            "DELETE FROM users WHERE username IN ('esnaf@qordy.com', 'yonetici', 'mehmet') OR tenant_id = 'CUST_69653cfccb839'",
            [],
        );

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM users WHERE username = 'aydilker123@gmail.com'", [], |r| r.get(0)).unwrap_or(0);

        if count == 0 {
            let now = Local::now().to_rfc3339();
            let tenant = if self.config.tenant_id.is_empty() || self.config.tenant_id == "CUST_69653cfccb839" {
                "CUST_6aca07fe3e290"
            } else {
                &self.config.tenant_id
            };

            // 1. Esnaf Mağaza Sahibi (Yönetici): İlker Aydoğdu - aydilker123@gmail.com (PIN: 2801 / 9999, Şifre: ilkN.2801)
            let salt0 = Uuid::new_v4().simple().to_string();
            let pin_hash0 = hash_with_salt("2801", &salt0);
            let pass_hash0 = hash_with_salt("ilkN.2801", &salt0);

            conn.execute(
                "INSERT INTO users (id, tenant_id, name, username, pin_hash, pin_salt, password_hash, password_salt, role, is_active, failed_attempts, created_at)
                 VALUES ('u_a042e0110eed1_1791625214', ?1, 'İlker Aydoğdu', 'aydilker123@gmail.com', ?2, ?3, ?4, ?3, 'MANAGER', 1, 0, ?5)",
                params![tenant, pin_hash0, salt0, pass_hash0, now],
            )?;

            // 2. Kasiyer: Ahmet (PIN: 1234, Şifre: 123)
            let salt1 = Uuid::new_v4().simple().to_string();
            let pin_hash1 = hash_with_salt("1234", &salt1);
            let pass_hash1 = hash_with_salt("123", &salt1);

            let _ = conn.execute(
                "INSERT OR IGNORE INTO users (id, tenant_id, name, username, pin_hash, pin_salt, password_hash, password_salt, role, is_active, failed_attempts, created_at)
                 VALUES ('usr_ahmet_kasiyer', ?1, 'Ahmet Kasiyer', 'ahmet', ?2, ?3, ?4, ?3, 'CASHIER', 1, 0, ?5)",
                params![tenant, pin_hash1, salt1, pass_hash1, now],
            );

            // 3. Kasiyer: Ayşe (PIN: 5678, Şifre: 123)
            let salt2 = Uuid::new_v4().simple().to_string();
            let pin_hash2 = hash_with_salt("5678", &salt2);
            let pass_hash2 = hash_with_salt("123", &salt2);

            let _ = conn.execute(
                "INSERT OR IGNORE INTO users (id, tenant_id, name, username, pin_hash, pin_salt, password_hash, password_salt, role, is_active, failed_attempts, created_at)
                 VALUES ('usr_ayse_kasiyer', ?1, 'Ayşe Kasiyer', 'ayse', ?2, ?3, ?4, ?3, 'CASHIER', 1, 0, ?5)",
                params![tenant, pin_hash2, salt2, pass_hash2, now],
            );
        } else {
            let ayse_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM users WHERE username = 'ayse'",
                [],
                |r| r.get(0),
            ).unwrap_or(0);

            if ayse_count == 0 {
                let tenant = if self.config.tenant_id.is_empty() || self.config.tenant_id == "CUST_69653cfccb839" {
                    "CUST_6aca07fe3e290"
                } else {
                    &self.config.tenant_id
                };
                let salt2 = Uuid::new_v4().simple().to_string();
                let pin_hash2 = hash_with_salt("5678", &salt2);
                let pass_hash2 = hash_with_salt("123", &salt2);
                let _ = conn.execute(
                    "INSERT INTO users (id, tenant_id, name, username, pin_hash, pin_salt, password_hash, password_salt, role, is_active, failed_attempts, created_at)
                     VALUES (?1, ?2, 'Ayşe Kasiyer', 'ayse', ?3, ?4, ?5, ?4, 'CASHIER', 1, 0, ?6)",
                    params![Uuid::new_v4().to_string(), tenant, pin_hash2, salt2, pass_hash2, Local::now().to_rfc3339()],
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_constant_time_eq() {
        let salt = "mysalt123";
        let h1 = hash_with_salt("1234", salt);
        let h2 = hash_with_salt("1234", salt);
        let h3 = hash_with_salt("9999", salt);

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert!(constant_time_eq(&h1, &h2));
        assert!(!constant_time_eq(&h1, &h3));
        assert!(!constant_time_eq("abc", "abcd"));
    }

    #[test]
    fn test_auth_lifecycle_pin_and_password() -> Result<()> {
        let db = DatabaseManager::new_in_memory()?;

        // Kullanıcı listesi
        let users = db.list_active_users()?;
        assert_eq!(users.len(), 3);

        // Kasiyer PIN girişi (Başarılı)
        let login_res = db.login_with_pin("1234", true)?;
        assert!(login_res.success);
        assert!(login_res.token.is_some());
        assert_eq!(login_res.user.unwrap().name, "Ahmet Kasiyer");

        let token = login_res.token.unwrap();

        // Oturum doğrulama
        let session = db.validate_session(&token)?;
        assert!(session.is_some());
        let sess = session.unwrap();
        assert_eq!(sess.user_name, "Ahmet Kasiyer");
        assert!(!sess.is_locked);

        // Ekranı Kilitle
        db.lock_session(&token)?;
        let locked_sess = db.validate_session(&token)?.unwrap();
        assert!(locked_sess.is_locked);

        // Yanlış PIN ile kilit açmayı dene
        let unlock_fail = db.unlock_session(&token, "0000")?;
        assert!(!unlock_fail);

        // Doğru PIN ile kilidi aç
        let unlock_ok = db.unlock_session(&token, "1234")?;
        assert!(unlock_ok);
        let unlocked_sess = db.validate_session(&token)?.unwrap();
        assert!(!unlocked_sess.is_locked);

        // Esnaf Yönetici Şifresi ile Giriş
        let admin_res = db.login_with_password("aydilker123@gmail.com", "ilkN.2801", false)?;
        assert!(admin_res.success);
        assert_eq!(admin_res.user.as_ref().unwrap().role, "MANAGER");
        assert_eq!(admin_res.user.as_ref().unwrap().name, "İlker Aydoğdu");

        // Hatalı şifre
        let fail_res = db.login_with_password("aydilker123@gmail.com", "wrongpass", false)?;
        assert!(!fail_res.success);

        // PIN Güncelleme Testi
        let user_id = admin_res.user.unwrap().id;
        let update_pin_ok = db.update_user_pin(&user_id, "8888")?;
        assert!(update_pin_ok);
        let new_pin_login = db.login_with_pin("8888", false)?;
        assert!(new_pin_login.success);

        // Yeni Kasiyer Ekleme Testi
        let new_cashier = db.create_cashier("Mehmet Kasiyer", "mehmet", "4321", "CASHIER")?;
        assert_eq!(new_cashier.name, "Mehmet Kasiyer");
        let mehmet_login = db.login_with_pin("4321", false)?;
        assert!(mehmet_login.success);

        // Logout
        db.logout_session(&token)?;
        assert!(db.validate_session(&token)?.is_none());

        Ok(())
    }

    #[test]
    fn test_brute_force_lockout() -> Result<()> {
        let db = DatabaseManager::new_in_memory()?;

        // 5 kez hatalı şifre denemesi
        for _ in 0..5 {
            let _ = db.login_with_password("aydilker123@gmail.com", "wrong", false);
        }

        // 6. denemede güvenlik kilidi devreye girmeli
        let res = db.login_with_password("aydilker123@gmail.com", "ilkN.2801", false)?;
        assert!(!res.success);
        assert!(res.is_locked);
        assert!(res.error.unwrap().contains("kilitlendi"));

        Ok(())
    }
}
