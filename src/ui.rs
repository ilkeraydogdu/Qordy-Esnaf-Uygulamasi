//! QORDY POS - Embedded UI & Static Asset Loader
//! Dynamic disk loader in development with embedded binary fallback.

use std::path::{Path, PathBuf};

fn find_asset(rel: &str) -> Option<PathBuf> {
    let clean = rel.trim_start_matches('/');
    // Check current working directory
    let p = Path::new(clean);
    if p.exists() && p.is_file() {
        return Some(p.to_path_buf());
    }
    // Check if prepending "assets/" helps if not already present
    if !clean.starts_with("assets") {
        let p_assets = Path::new("assets").join(clean);
        if p_assets.exists() && p_assets.is_file() {
            return Some(p_assets);
        }
    }
    // Check relative to executable parent
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let p_exe = parent.join(clean);
            if p_exe.exists() && p_exe.is_file() {
                return Some(p_exe);
            }
        }
    }
    None
}

pub fn get_pos_html() -> String {
    if let Some(p) = find_asset("assets/index.html") {
        if let Ok(content) = std::fs::read_to_string(p) {
            return content;
        }
    }
    include_str!("../assets/index.html").to_string()
}

pub fn get_customer_display_html() -> String {
    if let Some(p) = find_asset("assets/customer.html") {
        if let Ok(content) = std::fs::read_to_string(p) {
            return content;
        }
    }
    include_str!("../assets/customer.html").to_string()
}

#[allow(dead_code)]
pub fn get_pos_css() -> String {
    if let Some(p) = find_asset("assets/css/style.css") {
        if let Ok(content) = std::fs::read_to_string(p) {
            return content;
        }
    }
    include_str!("../assets/css/style.css").to_string()
}

#[allow(dead_code)]
pub fn get_customer_css() -> String {
    if let Some(p) = find_asset("assets/css/customer.css") {
        if let Ok(content) = std::fs::read_to_string(p) {
            return content;
        }
    }
    include_str!("../assets/css/customer.css").to_string()
}

pub fn get_static_asset(path: &str) -> Option<(String, &'static str)> {
    let clean_path = path.split('?').next().unwrap_or(path);

    // Try disk first
    if let Some(p) = find_asset(clean_path) {
        if let Ok(content) = std::fs::read_to_string(&p) {
            let mime = if clean_path.ends_with(".css") {
                "text/css; charset=utf-8"
            } else if clean_path.ends_with(".js") {
                "application/javascript; charset=utf-8"
            } else if clean_path.ends_with(".svg") {
                "image/svg+xml; charset=utf-8"
            } else {
                "text/plain; charset=utf-8"
            };
            return Some((content, mime));
        }
    }

    // Embedded fallback
    match clean_path {
        "/assets/css/style.css" => Some((include_str!("../assets/css/style.css").to_string(), "text/css; charset=utf-8")),
        "/assets/css/customer.css" => Some((include_str!("../assets/css/customer.css").to_string(), "text/css; charset=utf-8")),
        "/assets/js/api.js" => Some((include_str!("../assets/js/api.js").to_string(), "application/javascript; charset=utf-8")),
        "/assets/js/cart.js" => Some((include_str!("../assets/js/cart.js").to_string(), "application/javascript; charset=utf-8")),
        "/assets/js/terminal.js" => Some((include_str!("../assets/js/terminal.js").to_string(), "application/javascript; charset=utf-8")),
        "/assets/js/views.js" => Some((include_str!("../assets/js/views.js").to_string(), "application/javascript; charset=utf-8")),
        "/assets/js/app.js" => Some((include_str!("../assets/js/app.js").to_string(), "application/javascript; charset=utf-8")),
        "/assets/js/auth.js" => Some((include_str!("../assets/js/auth.js").to_string(), "application/javascript; charset=utf-8")),
        "/assets/js/customer.js" => Some((include_str!("../assets/js/customer.js").to_string(), "application/javascript; charset=utf-8")),
        "/assets/images/pofuduk-logo.svg" => Some((include_str!("../assets/images/pofuduk-logo.svg").to_string(), "image/svg+xml; charset=utf-8")),
        _ => None,
    }
}

pub fn get_static_binary_asset(path: &str) -> Option<(Vec<u8>, &'static str)> {
    let clean_path = path.split('?').next().unwrap_or(path);

    // Try disk first
    if let Some(p) = find_asset(clean_path) {
        if let Ok(bytes) = std::fs::read(&p) {
            let mime = if clean_path.ends_with(".webp") {
                "image/webp"
            } else if clean_path.ends_with(".png") {
                "image/png"
            } else if clean_path.ends_with(".ico") {
                "image/x-icon"
            } else {
                "application/octet-stream"
            };
            return Some((bytes, mime));
        }
    }

    // Embedded fallback
    match clean_path {
        "/assets/images/caddecafe_logo.webp" => Some((include_bytes!("../assets/images/caddecafe_logo.webp").to_vec(), "image/webp")),
        "/assets/images/logo.webp" => Some((include_bytes!("../assets/images/logo.webp").to_vec(), "image/webp")),
        "/assets/images/logo_light.webp" => Some((include_bytes!("../assets/images/logo_light.webp").to_vec(), "image/webp")),
        "/assets/images/qordy_app_icon.png" => Some((include_bytes!("../assets/images/qordy_app_icon.png").to_vec(), "image/png")),
        "/assets/images/favicon.png" => Some((include_bytes!("../assets/images/favicon.png").to_vec(), "image/png")),
        "/assets/images/app.ico" => Some((include_bytes!("../assets/images/app.ico").to_vec(), "image/x-icon")),
        "/assets/images/favicon.ico" => Some((include_bytes!("../assets/images/favicon.ico").to_vec(), "image/x-icon")),
        _ => None,
    }
}
