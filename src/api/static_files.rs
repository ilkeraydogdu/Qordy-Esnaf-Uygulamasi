//! QORDY POS - Static File & Webview Asset Serving

use crate::ui;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub async fn handle_index() -> Response {
    (
        StatusCode::OK,
        [
            ("content-type", "text/html; charset=utf-8"),
            ("cache-control", "no-cache, no-store, must-revalidate"),
        ],
        ui::get_pos_html(),
    )
        .into_response()
}

pub async fn handle_customer_display() -> Response {
    (
        StatusCode::OK,
        [
            ("content-type", "text/html; charset=utf-8"),
            ("cache-control", "no-cache, no-store, must-revalidate"),
        ],
        ui::get_customer_display_html(),
    )
        .into_response()
}

pub async fn handle_static_asset(Path(path): Path<String>) -> Response {
    let full_path = format!("/assets/{}", path);
    if let Some((content, mime)) = ui::get_static_asset(&full_path) {
        (
            StatusCode::OK,
            [("content-type", mime), ("cache-control", "no-cache, no-store, must-revalidate")],
            content,
        )
            .into_response()
    } else {
        (StatusCode::NOT_FOUND, "Static asset not found").into_response()
    }
}
