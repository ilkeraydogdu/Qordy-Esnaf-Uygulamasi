//! QORDY POS - 4.13 Milyon Ulusal Barkod Kataloğu API Handlers

use crate::api::AppState;
use crate::models::NationalProductInfo;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

#[derive(Deserialize)]
pub struct SearchCatalogQuery {
    pub q: String,
}

pub async fn handle_national_catalog_lookup(
    State(state): State<Arc<AppState>>,
    Path(barcode): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // 1. Önce yerel SQLite önbelleğini kontrol et
    if let Some(cached) = state.db.lookup_national_catalog(&barcode) {
        return Ok(Json(serde_json::json!({
            "success": true,
            "found": true,
            "product": cached,
            "source": "LOCAL_CACHE"
        })));
    }

    // 2. Yerelde yoksa ve internet bağlantısı varsa sunucunun 4.13M kataloğundan sorgula
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(2500))
        .build()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let url = format!("{}/api/esnaf/barcode/{}", state.config.cloud_url, barcode);
    info!("🔍 Ulusal Barkod Kataloğu Sorgulanıyor (4.13M Veri): {}", url);

    if let Ok(res) = client.get(&url).send().await {
        if res.status().is_success() {
            if let Ok(json_data) = res.json::<serde_json::Value>().await {
                let data = if json_data.get("data").is_some() {
                    &json_data["data"]
                } else if json_data.get("product").is_some() {
                    &json_data["product"]
                } else {
                    &json_data
                };

                if let Some(name) = data.get("name").and_then(|v| v.as_str()) {
                    let brand = data.get("brand").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let cat = data
                        .get("category_name")
                        .or_else(|| data.get("category"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let vat = data.get("vat_rate").and_then(|v| v.as_f64());
                    let unit = data
                        .get("unit")
                        .and_then(|v| v.as_str())
                        .unwrap_or("adet")
                        .to_string();
                    let is_w = data
                        .get("is_weighable")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let plu = data
                        .get("plu_code")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let price = data
                        .get("suggested_price")
                        .or_else(|| data.get("price"))
                        .and_then(|v| v.as_f64());

                    let product = NationalProductInfo {
                        barcode: barcode.clone(),
                        name: name.to_string(),
                        brand,
                        category: cat,
                        vat_rate: vat,
                        unit,
                        is_weighed: is_w,
                        plu_code: plu,
                        suggested_price: price,
                        source: "NATIONAL_CATALOG_4.13M".to_string(),
                    };

                    // Yerel SQLite'a önbellekle
                    let _ = state.db.save_national_catalog_item(&product);

                    return Ok(Json(serde_json::json!({
                        "success": true,
                        "found": true,
                        "product": product,
                        "source": "NATIONAL_CATALOG_4.13M"
                    })));
                }
            }
        }
    }

    Err((
        StatusCode::NOT_FOUND,
        "Barkod ulusal katalogda bulunamadı".to_string(),
    ))
}

pub async fn handle_national_catalog_search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchCatalogQuery>,
) -> Json<serde_json::Value> {
    let results = state.db.search_national_catalog(&query.q);
    Json(serde_json::json!({
        "success": true,
        "results": results
    }))
}
