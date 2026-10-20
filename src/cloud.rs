//! QORDY POS - Cloud REST API Integration
//! Connects to https://qordy.com/api/mobile for real-time authentication,
//! product catalog synchronization, customers, suppliers, and retail checkouts.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

pub const QORDY_DEFAULT_MOBILE_API: &str = "https://qordy.com/api/mobile";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudLoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudUser {
    pub id: String,
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub is_manager: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudBusiness {
    pub id: String,
    pub name: String,
    pub subdomain: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    #[serde(default)]
    pub business_type: Option<String>,
    #[serde(default)]
    pub is_retail: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudLoginData {
    pub token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    pub user: CloudUser,
    pub business: CloudBusiness,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudLoginResponse {
    pub success: bool,
    pub data: Option<CloudLoginData>,
    pub error: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProductItem {
    #[serde(alias = "product_id")]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub brand: Option<String>,
    #[serde(default)]
    pub barcode: Option<String>,
    #[serde(default)]
    pub price: Option<serde_json::Value>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub vat_rate: Option<serde_json::Value>,
    #[serde(default)]
    pub category_id: Option<String>,
    #[serde(default)]
    pub stock_qty: Option<serde_json::Value>,
    #[serde(default)]
    pub is_weighable: Option<serde_json::Value>,
    #[serde(default)]
    pub image_url: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProductsPageData {
    pub items: Vec<CloudProductItem>,
    pub total: usize,
    pub page: usize,
    pub limit: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProductsResponse {
    pub success: bool,
    pub data: Option<CloudProductsPageData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCustomerItem {
    pub customer_id: String,
    pub full_name: String,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub balance: Option<serde_json::Value>,
    #[serde(default)]
    pub credit_limit: Option<serde_json::Value>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCustomersData {
    pub customers: Vec<CloudCustomerItem>,
    #[serde(default)]
    pub total: Option<usize>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCustomersResponse {
    pub success: bool,
    pub data: Option<CloudCustomersData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSupplierItem {
    pub id: Option<i64>,
    pub supplier_id: String,
    pub name: String,
    #[serde(default)]
    pub contact: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub balance: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSuppliersResponse {
    pub success: bool,
    #[serde(default)]
    pub suppliers: Vec<CloudSupplierItem>,
}

#[derive(Debug, Clone)]
pub struct CloudSyncBundle {
    pub token: String,
    pub user: CloudUser,
    pub business: CloudBusiness,
    pub products: Vec<CloudProductItem>,
    pub customers: Vec<CloudCustomerItem>,
    pub suppliers: Vec<CloudSupplierItem>,
}

#[derive(Clone)]
pub struct QordyCloudClient {
    client: Client,
    base_url: String,
}

impl Default for QordyCloudClient {
    fn default() -> Self {
        Self::new(QORDY_DEFAULT_MOBILE_API)
    }
}

impl QordyCloudClient {
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(25))
            .connect_timeout(Duration::from_secs(10))
            .user_agent("QordyPos/3.0.0 (Desktop; Offline-First)")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Gerçek Qordy Mobil/Esnaf API'sine giriş yap
    pub async fn login(&self, email: &str, password: &str) -> Result<CloudLoginData> {
        let url = format!("{}/auth/login", self.base_url);
        info!("Qordy Bulut Girişi deneniyor: {} -> {}", email, url);

        let res = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Bulut sunucusuna bağlanılamadı: {:#}", e))?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            anyhow::bail!("Bulut giriş başarısız (HTTP {}): {}", status, body);
        }

        let resp: CloudLoginResponse = res.json().await.context("Giriş yanıtı çözümlenemedi.")?;
        if let Some(data) = resp.data {
            info!(
                "✅ Bulut girişi başarılı! Mağaza: '{}' ({}) - Kullanıcı: '{}'",
                data.business.name, data.business.id, data.user.name
            );
            Ok(data)
        } else {
            anyhow::bail!(resp.error.or(resp.message).unwrap_or_else(|| "Bilinmeyen giriş hatası".to_string()))
        }
    }

    /// Mağazaya ait tüm perakende ürünleri sayfalar halinde buluttan çek (Limit 100)
    pub async fn fetch_all_retail_products(&self, token: &str) -> Result<Vec<CloudProductItem>> {
        let mut all_products = Vec::new();
        let mut page = 1;
        let limit = 100;

        loop {
            let url = format!("{}/retail/products?limit={}&page={}", self.base_url, limit, page);
            let res = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("Accept", "application/json")
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("Ürün kataloğu sayfası {} alınamadı: {:#}", page, e))?;

            if !res.status().is_success() {
                warn!("Bulut ürün sayfası {} HTTP hatası: {}", page, res.status());
                break;
            }

            let body_val: serde_json::Value = res
                .json()
                .await
                .map_err(|e| anyhow::anyhow!("Ürün listesi JSON ayrıştırılamadı (sayfa {}): {:#}", page, e))?;

            let data_obj = body_val.get("data").and_then(|d| d.as_object());
            if let Some(data) = data_obj {
                let total = data.get("total").and_then(|t| t.as_u64()).unwrap_or(0) as usize;
                let items_arr = data.get("items").and_then(|i| i.as_array());

                let mut fetched_count = 0;
                if let Some(items) = items_arr {
                    fetched_count = items.len();
                    for item in items {
                        let id = item["id"]
                            .as_str()
                            .or_else(|| item["product_id"].as_str())
                            .unwrap_or("")
                            .to_string();
                        if id.is_empty() {
                            continue;
                        }
                        let name = item["name"].as_str().unwrap_or("İsimsiz Ürün").to_string();
                        let brand = item["brand"].as_str().map(|s| s.to_string());
                        let barcode = item["barcode"].as_str().map(|s| s.to_string());
                        let price = item.get("price").cloned();
                        let unit = item["unit"].as_str().map(|s| s.to_string());
                        let vat_rate = item.get("vat_rate").cloned();
                        let category_id = item["category_id"].as_str().map(|s| s.to_string());
                        let stock_qty = item.get("stock_qty").cloned();
                        let is_weighable = item.get("is_weighable").cloned();
                        let image_url = item["image_url"].as_str().map(|s| s.to_string());

                        all_products.push(CloudProductItem {
                            id,
                            name,
                            brand,
                            barcode,
                            price,
                            unit,
                            vat_rate,
                            category_id,
                            stock_qty,
                            is_weighable,
                            image_url,
                        });
                    }
                }

                info!(
                    "Buluttan ürün sayfası {} yüklendi: {} adet (Toplam: {}/{})",
                    page,
                    fetched_count,
                    all_products.len(),
                    total
                );

                if all_products.len() >= total || fetched_count == 0 {
                    break;
                }
                page += 1;
            } else {
                break;
            }
        }

        info!("Toplam {} adet gerçek ürün buluttan başarıyla çekildi.", all_products.len());
        Ok(all_products)
    }

    /// Veresiye müşterilerini çek
    pub async fn fetch_retail_customers(&self, token: &str) -> Result<Vec<CloudCustomerItem>> {
        let url = format!("{}/retail/customers", self.base_url);
        let res = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Müşteri listesi alınamadı.")?;

        if !res.status().is_success() {
            anyhow::bail!("Müşteriler HTTP hatası: {}", res.status());
        }

        let body_val: serde_json::Value = res.json().await.context("Müşteri JSON ayrıştırılamadı.")?;
        let mut list = Vec::new();
        if let Some(arr) = body_val
            .get("data")
            .and_then(|d| d.get("customers"))
            .and_then(|c| c.as_array())
        {
            for c in arr {
                let customer_id = c["customer_id"].as_str().unwrap_or("").to_string();
                if customer_id.is_empty() {
                    continue;
                }
                let full_name = c["full_name"].as_str().unwrap_or("Müşteri").to_string();
                let phone = c["phone"].as_str().map(|s| s.to_string());
                let balance = c.get("balance").cloned();
                let credit_limit = c.get("credit_limit").cloned();
                let note = c["note"].as_str().map(|s| s.to_string());
                let updated_at = c["updated_at"].as_str().map(|s| s.to_string());

                list.push(CloudCustomerItem {
                    customer_id,
                    full_name,
                    phone,
                    balance,
                    credit_limit,
                    note,
                    updated_at,
                });
            }
        }
        info!("Toplam {} adet veresiye müşterisi buluttan çekildi.", list.len());
        Ok(list)
    }

    /// Tedarikçileri çek
    pub async fn fetch_suppliers(&self, token: &str) -> Result<Vec<CloudSupplierItem>> {
        let url = format!("{}/finance/suppliers", self.base_url);
        let res = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Tedarikçi listesi alınamadı.")?;

        if !res.status().is_success() {
            return Ok(Vec::new());
        }

        let body_val: serde_json::Value = res.json().await.unwrap_or(serde_json::json!({}));
        let mut list = Vec::new();
        if let Some(arr) = body_val.get("suppliers").and_then(|s| s.as_array()) {
            for s in arr {
                let supplier_id = s["supplier_id"].as_str().unwrap_or("").to_string();
                if supplier_id.is_empty() {
                    continue;
                }
                let name = s["name"].as_str().unwrap_or("Tedarikçi").to_string();
                let contact = s["contact"].as_str().map(|s| s.to_string());
                let category = s["category"].as_str().map(|s| s.to_string());
                let balance = s["balance"].as_str().map(|s| s.to_string());
                let created_at = s["created_at"].as_str().map(|s| s.to_string());

                list.push(CloudSupplierItem {
                    id: s["id"].as_i64(),
                    supplier_id,
                    name,
                    contact,
                    category,
                    balance,
                    created_at,
                });
            }
        }
        info!("Toplam {} adet tedarikçi buluttan çekildi.", list.len());
        Ok(list)
    }

    /// Tüm mağaza verisini tek seferde indirip paketle
    pub async fn fetch_full_store_bundle(&self, email: &str, password: &str) -> Result<CloudSyncBundle> {
        let login_data = self.login(email, password).await?;
        let token = login_data.token.clone();

        let products = self.fetch_all_retail_products(&token).await?;
        let customers = self.fetch_retail_customers(&token).await.unwrap_or_default();
        let suppliers = self.fetch_suppliers(&token).await.unwrap_or_default();

        Ok(CloudSyncBundle {
            token,
            user: login_data.user,
            business: login_data.business,
            products,
            customers,
            suppliers,
        })
    }

    /// Gerçek zamanlı kasa fişi satışı gönder
    #[allow(dead_code)]
    pub async fn send_retail_checkout(
        &self,
        token: &str,
        items: &[serde_json::Value],
        payment_method: &str,
        amount: f64,
        customer_id: Option<&str>,
        note: Option<&str>,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/retail/checkout", self.base_url);
        let mut body = serde_json::json!({
            "items": items,
            "payment_method": payment_method,
            "amount": amount,
        });

        if let Some(cid) = customer_id {
            if !cid.is_empty() {
                body["customer_id"] = serde_json::Value::String(cid.to_string());
            }
        }
        if let Some(n) = note {
            if !n.is_empty() {
                body["note"] = serde_json::Value::String(n.to_string());
            }
        }

        let res = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .await
            .context("Bulut checkout isteği başarısız oldu.")?;

        let resp_json: serde_json::Value = res.json().await.unwrap_or(serde_json::json!({"success": true}));
        Ok(resp_json)
    }

    /// Veresiye borç tahsilatını buluta aktar
    #[allow(dead_code)]
    pub async fn send_debt_collection(
        &self,
        token: &str,
        customer_id: &str,
        amount: f64,
        payment_method: &str,
        note: Option<&str>,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/retail/customers/collect", self.base_url);
        let mut body = serde_json::json!({
            "customer_id": customer_id,
            "amount": amount,
            "payment_method": payment_method,
        });
        if let Some(n) = note {
            body["note"] = serde_json::Value::String(n.to_string());
        }

        let res = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .await
            .context("Bulut tahsilat isteği başarısız oldu.")?;

        let resp_json: serde_json::Value = res.json().await.unwrap_or(serde_json::json!({"success": true}));
        Ok(resp_json)
    }
}
