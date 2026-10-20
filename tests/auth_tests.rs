#[cfg(test)]
mod auth_tests {
    use qordy_pos_rs::storage::DatabaseManager;
    use tempfile::NamedTempFile;

    #[test]
    fn test_auth_persistence_across_db_reconnect() {
        let temp_file = NamedTempFile::new().expect("Temp file failed");
        let db_path = temp_file.path().to_path_buf();

        let token: String;
        // 1. Session oluştur ve diske kaydet
        {
            let db = DatabaseManager::new(&db_path).expect("DB init failed");
            let login = db.login_with_pin("1234", true).expect("PIN login failed");
            assert!(login.success);
            token = login.token.expect("Token must exist");

            let sess = db.validate_session(&token).expect("Validate failed");
            assert!(sess.is_some());
            assert_eq!(sess.unwrap().user_name, "Ahmet Kasiyer");
        }

        // 2. Uygulama kapandı ve tekrar açıldı (Re-open DB connection from same file)
        {
            let db_reopened = DatabaseManager::new(&db_path).expect("DB re-open failed");
            let sess = db_reopened.validate_session(&token).expect("Validate after reopen failed");
            assert!(sess.is_some(), "Kalıcı oturum disk üzerinde korunmalı");
            assert_eq!(sess.unwrap().user_name, "Ahmet Kasiyer");

            // Kilit testi
            db_reopened.lock_session(&token).expect("Lock failed");
            let locked = db_reopened.validate_session(&token).expect("Validate failed").unwrap();
            assert!(locked.is_locked);

            // Yanlış PIN ile kilit açılmaz
            let fail = db_reopened.unlock_session(&token, "9999").expect("Unlock call failed");
            assert!(!fail);

            // Doğru PIN ile kilit açılır
            let ok = db_reopened.unlock_session(&token, "1234").expect("Unlock call failed");
            assert!(ok);
            let unlocked = db_reopened.validate_session(&token).expect("Validate failed").unwrap();
            assert!(!unlocked.is_locked);
        }
    }

    #[test]
    fn test_manager_password_auth_and_logout() {
        let temp_file = NamedTempFile::new().expect("Temp file failed");
        let db = DatabaseManager::new(temp_file.path()).expect("DB init failed");

        // Yönetici şifre ile giriş
        let login = db.login_with_password("yonetici", "admin", true).expect("Login failed");
        assert!(login.success);
        let user = login.user.unwrap();
        assert_eq!(user.role, "MANAGER");
        assert_eq!(user.name, "Fatma Yönetici");

        let token = login.token.unwrap();

        // Oturumu kapat
        db.logout_session(&token).expect("Logout failed");
        let sess_after = db.validate_session(&token).expect("Validate failed");
        assert!(sess_after.is_none());
    }
}
