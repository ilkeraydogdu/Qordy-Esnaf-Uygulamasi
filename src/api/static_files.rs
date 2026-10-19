//! QORDY POS - Static File & Webview Asset Serving

use crate::ui;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

pub async fn handle_index() -> Html<&'static str> {
    Html(ui::get_pos_html())
}

pub async fn handle_customer_display() -> Html<&'static str> {
    Html(ui::get_customer_display_html())
}

pub async fn handle_static_asset(Path(path): Path<String>) -> Response {
    let full_path = format!("/assets/{}", path);
    if let Some((content, mime)) = ui::get_static_asset(&full_path) {
        (
            StatusCode::OK,
            [("content-type", mime), ("cache-control", "public, max-age=3600")],
            content,
        )
            .into_response()
    } else {
        (StatusCode::NOT_FOUND, "Static asset not found").into_response()
    }
}
