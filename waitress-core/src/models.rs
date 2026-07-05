//! Plain data structs mirroring the original Python dataclasses
//! (`waitress/shared/models.py`). No ORM — just serde-friendly holders that
//! cross the Tauri IPC boundary as JSON.

use serde::{Deserialize, Serialize};

pub const ORDER_STATUSES: [&str; 6] =
    ["NEW", "ACCEPTED", "PREPARING", "READY", "SERVED", "CANCELLED"];
pub const PAYMENT_STATUSES: [&str; 5] = ["UNPAID", "PENDING", "PAID", "FAILED", "REFUNDED"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestaurantSettings {
    pub name: String,
    pub tagline: String,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub logo_path: String,
    pub active_background: String,
    pub currency: String,
    pub currency_symbol: String,
    pub tax_rate: f64,
    pub printing_enabled: bool,
    pub kitchen_display_mode: String, // tablet | printer | both
    pub voice_greeting_enabled: bool,
    pub notify_on_payment: bool,
    pub payment_gateways: String,       // CSV, enabled/visible-to-customer
    pub payment_gateways_all: String,   // CSV, master list
    pub payment_gateway_config: String, // JSON: {gateway: {api_key, secret_key, merchant_id}}
}

impl Default for RestaurantSettings {
    fn default() -> Self {
        Self {
            name: "Your Restaurant".into(),
            tagline: "Fine dining, made simple".into(),
            address: String::new(),
            phone: String::new(),
            email: String::new(),
            logo_path: String::new(),
            active_background: String::new(),
            currency: "USD".into(),
            currency_symbol: "$".into(),
            tax_rate: 0.0,
            printing_enabled: false,
            kitchen_display_mode: "tablet".into(),
            voice_greeting_enabled: true,
            notify_on_payment: true,
            payment_gateways: "cash".into(),
            payment_gateways_all: "cash".into(),
            payment_gateway_config: "{}".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuCategory {
    pub id: Option<i64>,
    pub name: String,
    pub sort_order: i64,
    pub is_starter_group: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuItem {
    pub id: Option<i64>,
    pub category_id: i64,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub image_path: String,
    pub available: bool,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailySpecial {
    pub id: Option<i64>,
    pub menu_item_id: Option<i64>,
    pub title: String,
    pub description: String,
    pub price: f64,
    pub image_path: String,
    pub active: bool,
    pub active_date: String, // ISO date, "" = every day
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderItem {
    pub id: Option<i64>,
    pub order_id: Option<i64>,
    pub menu_item_id: Option<i64>,
    pub name: String,
    pub unit_price: f64,
    pub quantity: i64,
    pub notes: String,
}

impl OrderItem {
    pub fn line_total(&self) -> f64 {
        (self.unit_price * self.quantity as f64 * 100.0).round() / 100.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub id: Option<i64>,
    pub table_id: String,
    pub device_id: String,
    pub status: String,
    pub payment_status: String,
    pub payment_method: String,
    pub subtotal: f64,
    pub tax: f64,
    pub total: f64,
    pub kitchen_message: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUser {
    pub id: Option<i64>,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String, // superadmin | admin
    pub active: bool,
    pub created_at: String,
    pub last_login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct License {
    pub id: Option<i64>,
    pub code: String,
    pub device_id: String,
    pub license_id: i64,
    pub status: String,
    pub activated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: Option<i64>,
    pub kind: String, // payment | order | system
    pub title: String,
    pub body: String,
    pub order_id: Option<i64>,
    pub read: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConfig {
    pub device_id: String,
    pub role: String, // customer | kitchen | admin
    pub table_id: String,
    pub kiosk_enabled: bool,
    pub admin_pin: String,
    pub sync_peers: Vec<String>,
    pub language: String,
    pub printer_backend: String, // os | network
    pub printer_host: String,
    pub printer_port: u16,
    // Customer-module admin-login throttle: the padlock icon's PIN prompt
    // allows 3 failed attempts before locking out for 30 minutes. Persisted
    // here (not just in-memory in the Svelte dialog) so a device restart
    // can't be used to bypass the lockout.
    #[serde(default)]
    pub admin_login_attempts: u32,
    #[serde(default)]
    pub admin_lock_until: Option<i64>, // unix ms timestamp; None = not locked
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            device_id: String::new(),
            role: "customer".into(),
            table_id: String::new(),
            kiosk_enabled: true,
            admin_pin: "4242".into(),
            sync_peers: Vec::new(),
            language: "en".into(),
            printer_backend: "os".into(),
            printer_host: String::new(),
            printer_port: 9100,
            admin_login_attempts: 0,
            admin_lock_until: None,
        }
    }
}
