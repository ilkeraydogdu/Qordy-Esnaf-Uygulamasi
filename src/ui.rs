//! QORDY POS - Embedded UI & Static Asset Loader
//! Decoupled from application binary and compiled directly via include_str! macro.

pub fn get_pos_html() -> &'static str {
    include_str!("../assets/index.html")
}

pub fn get_customer_display_html() -> &'static str {
    include_str!("../assets/customer.html")
}

#[allow(dead_code)]
pub fn get_pos_css() -> &'static str {
    include_str!("../assets/css/style.css")
}

#[allow(dead_code)]
pub fn get_customer_css() -> &'static str {
    include_str!("../assets/css/customer.css")
}

pub fn get_static_asset(path: &str) -> Option<(&'static str, &'static str)> {
    let clean_path = path.split('?').next().unwrap_or(path);
    match clean_path {
        "/assets/css/style.css" => Some((include_str!("../assets/css/style.css"), "text/css; charset=utf-8")),
        "/assets/css/customer.css" => Some((include_str!("../assets/css/customer.css"), "text/css; charset=utf-8")),
        "/assets/js/api.js" => Some((include_str!("../assets/js/api.js"), "application/javascript; charset=utf-8")),
        "/assets/js/cart.js" => Some((include_str!("../assets/js/cart.js"), "application/javascript; charset=utf-8")),
        "/assets/js/terminal.js" => Some((include_str!("../assets/js/terminal.js"), "application/javascript; charset=utf-8")),
        "/assets/js/views.js" => Some((include_str!("../assets/js/views.js"), "application/javascript; charset=utf-8")),
        "/assets/js/app.js" => Some((include_str!("../assets/js/app.js"), "application/javascript; charset=utf-8")),
        "/assets/js/customer.js" => Some((include_str!("../assets/js/customer.js"), "application/javascript; charset=utf-8")),
        "/assets/images/pofuduk-logo.svg" => Some((include_str!("../assets/images/pofuduk-logo.svg"), "image/svg+xml; charset=utf-8")),
        _ => None,
    }
}

pub fn get_static_binary_asset(path: &str) -> Option<(&'static [u8], &'static str)> {
    let clean_path = path.split('?').next().unwrap_or(path);
    match clean_path {
        "/assets/images/caddecafe_logo.webp" => Some((include_bytes!("../assets/images/caddecafe_logo.webp"), "image/webp")),
        "/assets/images/logo.webp" => Some((include_bytes!("../assets/images/logo.webp"), "image/webp")),
        "/assets/images/logo_light.webp" => Some((include_bytes!("../assets/images/logo_light.webp"), "image/webp")),
        "/assets/images/qordy_app_icon.png" => Some((include_bytes!("../assets/images/qordy_app_icon.png"), "image/png")),
        "/assets/images/favicon.png" => Some((include_bytes!("../assets/images/favicon.png"), "image/png")),
        _ => None,
    }
}
