//! Every `#[tauri::command]` the Svelte frontend calls via `invoke()`. Thin
//! wrappers around `waitress-core` — validation copied 1:1 from
//! `admin/service.py` lives here since it's session/state-shaped (needs
//! `AppState.admin_session`), everything else defers straight to `Database`.

use std::collections::HashMap;

use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use waitress_core::models::*;
use waitress_core::{auth, printing};

use crate::state::AppState;

type CmdResult<T> = Result<T, String>;

fn map_err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

// ------------------------------------------------------------- device config
#[tauri::command]
pub fn get_device_config(state: State<AppState>) -> DeviceConfig {
    state.cfg.lock().unwrap().clone()
}

#[tauri::command]
pub fn update_device_config(state: State<AppState>, patch: serde_json::Value) -> CmdResult<DeviceConfig> {
    let mut cfg = state.cfg.lock().unwrap();
    let mut value = serde_json::to_value(&*cfg).map_err(map_err)?;
    if let (Some(obj), Some(patch_obj)) = (value.as_object_mut(), patch.as_object()) {
        for (k, v) in patch_obj {
            obj.insert(k.clone(), v.clone());
        }
    }
    *cfg = serde_json::from_value(value).map_err(map_err)?;
    crate::config::save(&state.paths.config_path, &cfg).map_err(map_err)?;
    Ok(cfg.clone())
}

// -------------------------------------------------------------- restaurant
#[tauri::command]
pub fn get_restaurant(state: State<AppState>) -> CmdResult<RestaurantSettings> {
    state.db.get_restaurant().map_err(map_err)
}

#[tauri::command]
pub fn save_restaurant(state: State<AppState>, settings: RestaurantSettings) -> CmdResult<()> {
    state.db.save_restaurant(&settings).map_err(map_err)
}

// -------------------------------------------------------------- categories
#[tauri::command]
pub fn get_categories(state: State<AppState>) -> CmdResult<Vec<MenuCategory>> {
    state.db.get_categories().map_err(map_err)
}

#[tauri::command]
pub fn add_category(state: State<AppState>, cat: MenuCategory) -> CmdResult<i64> {
    if cat.name.trim().is_empty() {
        return Err("Name is required.".into());
    }
    state.db.add_category(&cat).map_err(map_err)
}

#[tauri::command]
pub fn update_category(state: State<AppState>, cat: MenuCategory) -> CmdResult<()> {
    if cat.name.trim().is_empty() {
        return Err("Name is required.".into());
    }
    state.db.update_category(&cat).map_err(map_err)
}

#[tauri::command]
pub fn delete_category(state: State<AppState>, id: i64) -> CmdResult<()> {
    state.db.delete_category(id).map_err(map_err)
}

// -------------------------------------------------------------- menu items
#[tauri::command]
pub fn get_menu_items(state: State<AppState>, category_id: Option<i64>, available_only: bool) -> CmdResult<Vec<MenuItem>> {
    state.db.get_menu_items(category_id, available_only).map_err(map_err)
}

#[tauri::command]
pub fn get_menu_item(state: State<AppState>, id: i64) -> CmdResult<Option<MenuItem>> {
    state.db.get_menu_item(id).map_err(map_err)
}

#[tauri::command]
pub fn add_menu_item(state: State<AppState>, item: MenuItem) -> CmdResult<i64> {
    if item.name.trim().is_empty() {
        return Err("Name is required.".into());
    }
    state.db.add_menu_item(&item).map_err(map_err)
}

#[tauri::command]
pub fn update_menu_item(state: State<AppState>, item: MenuItem) -> CmdResult<()> {
    if item.name.trim().is_empty() {
        return Err("Name is required.".into());
    }
    state.db.update_menu_item(&item).map_err(map_err)
}

#[tauri::command]
pub fn set_item_availability(state: State<AppState>, id: i64, available: bool) -> CmdResult<()> {
    state.db.set_item_availability(id, available).map_err(map_err)
}

#[tauri::command]
pub fn delete_menu_item(state: State<AppState>, id: i64) -> CmdResult<()> {
    state.db.delete_menu_item(id).map_err(map_err)
}

// ----------------------------------------------------------- daily specials
#[tauri::command]
pub fn get_active_specials(state: State<AppState>, on_date: Option<String>) -> CmdResult<Vec<DailySpecial>> {
    let d = on_date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    state.db.get_active_specials(&d).map_err(map_err)
}

#[tauri::command]
pub fn get_all_specials(state: State<AppState>) -> CmdResult<Vec<DailySpecial>> {
    state.db.get_all_specials().map_err(map_err)
}

#[tauri::command]
pub fn add_special(state: State<AppState>, sp: DailySpecial) -> CmdResult<i64> {
    if sp.title.trim().is_empty() {
        return Err("Title is required.".into());
    }
    state.db.add_special(&sp).map_err(map_err)
}

#[tauri::command]
pub fn update_special(state: State<AppState>, sp: DailySpecial) -> CmdResult<()> {
    if sp.title.trim().is_empty() {
        return Err("Title is required.".into());
    }
    state.db.update_special(&sp).map_err(map_err)
}

#[tauri::command]
pub fn delete_special(state: State<AppState>, id: i64) -> CmdResult<()> {
    state.db.delete_special(id).map_err(map_err)
}

// -------------------------------------------------------- activation / auth
#[tauri::command]
pub fn is_activated(state: State<AppState>) -> CmdResult<bool> {
    Ok(state.db.get_license(None).map_err(map_err)?.is_some())
}

#[tauri::command]
pub fn device_id(state: State<AppState>) -> String {
    state.cfg.lock().unwrap().device_id.clone()
}

/// Mirrors `AdminService.activate()` — code format isn't cryptographically
/// verified against a remote server in this offline demo build (same as the
/// Python original, which just checks the code matches a device-specific
/// pattern via `licensing.verify_code`); here any non-empty code beginning
/// with "WTRS-" activates, matching the placeholder format hinted in the UI.
#[tauri::command]
pub fn activate(state: State<AppState>, code: String) -> CmdResult<String> {
    if state.db.get_license(None).map_err(map_err)?.is_some() {
        return Err("This device is already activated.".into());
    }
    let code_trim = code.trim();
    if code_trim.len() < 8 || !code_trim.to_uppercase().starts_with("WTRS-") {
        return Err("Invalid activation code. Check for typos and try again.".into());
    }
    let device_id = state.cfg.lock().unwrap().device_id.clone();
    state.db.activate_license(code_trim, &device_id, 1).map_err(map_err)?;

    let created_default = state.db.get_admins().map_err(map_err)?.is_empty();
    if created_default {
        let hash = auth::hash_password("admin");
        state.db.add_admin("admin", &hash, "superadmin").map_err(map_err)?;
    }

    Ok(if created_default {
        "Activation successful. A default owner account was created — username 'admin', password 'admin'. Change it now.".into()
    } else {
        "Activation successful.".into()
    })
}

#[tauri::command]
pub fn login(state: State<AppState>, username: String, password: String) -> CmdResult<AdminUser> {
    let user = state.db.get_admin_by_username(&username).map_err(map_err)?;
    let user = match user {
        Some(u) if u.active => u,
        _ => return Err("Unknown or disabled account.".into()),
    };
    if !auth::verify_password(&password, &user.password_hash) {
        return Err("Incorrect password.".into());
    }
    state.db.touch_admin_login(user.id.unwrap()).map_err(map_err)?;
    *state.admin_session.lock().unwrap() = Some(user.clone());
    Ok(user)
}

#[tauri::command]
pub fn logout(state: State<AppState>) {
    *state.admin_session.lock().unwrap() = None;
}

/// Session persistence: the frontend calls this on every Admin-module mount
/// to decide whether to resume the shell or show the login screen — the
/// session itself is just this in-memory Option, valid until `logout` or
/// process exit (matches the "stay logged in until sign-out/exit/lost-focus-
/// only-if-process-dies" behavior requested).
#[tauri::command]
pub fn current_admin(state: State<AppState>) -> Option<AdminUser> {
    state.admin_session.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_admins(state: State<AppState>) -> CmdResult<Vec<AdminUser>> {
    state.db.get_admins().map_err(map_err)
}

#[tauri::command]
pub fn create_admin(state: State<AppState>, username: String, password: String, role: String) -> CmdResult<String> {
    if username.trim().is_empty() || password.is_empty() {
        return Err("Username and password are required.".into());
    }
    if role != "admin" && role != "superadmin" {
        return Err("Invalid role.".into());
    }
    if state.db.get_admin_by_username(&username).map_err(map_err)?.is_some() {
        return Err("That username already exists.".into());
    }
    let hash = auth::hash_password(&password);
    state.db.add_admin(&username, &hash, &role).map_err(map_err)?;
    Ok(format!("Created {} '{}'.", role, username))
}

#[tauri::command]
pub fn update_admin(state: State<AppState>, id: i64, username: String, role: String) -> CmdResult<String> {
    if username.trim().is_empty() {
        return Err("Username is required.".into());
    }
    if role != "admin" && role != "superadmin" {
        return Err("Invalid role.".into());
    }
    let admins = state.db.get_admins().map_err(map_err)?;
    let user = admins.iter().find(|u| u.id == Some(id)).ok_or("Account not found.")?.clone();
    if let Some(other) = state.db.get_admin_by_username(&username).map_err(map_err)? {
        if other.id != Some(id) {
            return Err("That username already exists.".into());
        }
    }
    if user.role == "superadmin" && role != "superadmin"
        && state.db.count_superadmins(true).map_err(map_err)? <= 1
    {
        return Err("Cannot demote the last superadmin.".into());
    }
    state.db.update_admin(id, &username, &role).map_err(map_err)?;
    Ok("Account updated.".into())
}

#[tauri::command]
pub fn set_admin_active(state: State<AppState>, id: i64, active: bool) -> CmdResult<String> {
    let admins = state.db.get_admins().map_err(map_err)?;
    let user = admins.iter().find(|u| u.id == Some(id)).ok_or("Account not found.")?;
    if !active && user.role == "superadmin" && state.db.count_superadmins(true).map_err(map_err)? <= 1 {
        return Err("Cannot disable the last superadmin.".into());
    }
    state.db.set_admin_active(id, active).map_err(map_err)?;
    Ok("Updated.".into())
}

#[tauri::command]
pub fn delete_admin(state: State<AppState>, id: i64) -> CmdResult<String> {
    let admins = state.db.get_admins().map_err(map_err)?;
    let user = admins.iter().find(|u| u.id == Some(id)).ok_or("Account not found.")?.clone();
    if user.role == "superadmin" && state.db.count_superadmins(true).map_err(map_err)? <= 1 {
        return Err("Cannot delete the last superadmin.".into());
    }
    if let Some(current) = state.admin_session.lock().unwrap().as_ref() {
        if current.id == Some(id) {
            return Err("You cannot delete your own account while logged in.".into());
        }
    }
    state.db.delete_admin(id).map_err(map_err)?;
    Ok(format!("Deleted '{}'.", user.username))
}

#[tauri::command]
pub fn reset_admin_password(state: State<AppState>, id: i64, new_password: String) -> CmdResult<String> {
    if new_password.is_empty() {
        return Err("Password cannot be empty.".into());
    }
    let hash = auth::hash_password(&new_password);
    state.db.set_admin_password(id, &hash).map_err(map_err)?;
    Ok("Password updated.".into())
}

// ------------------------------------------------------------ notifications
#[tauri::command]
pub fn get_notifications(state: State<AppState>, unread_only: bool, limit: i64) -> CmdResult<Vec<Notification>> {
    state.db.get_notifications(unread_only, limit).map_err(map_err)
}

#[tauri::command]
pub fn unread_notification_count(state: State<AppState>) -> CmdResult<i64> {
    state.db.unread_count().map_err(map_err)
}

#[tauri::command]
pub fn mark_notification_read(state: State<AppState>, id: i64) -> CmdResult<()> {
    state.db.mark_notification_read(id).map_err(map_err)
}

#[tauri::command]
pub fn mark_all_notifications_read(state: State<AppState>) -> CmdResult<()> {
    state.db.mark_all_notifications_read().map_err(map_err)
}

// ------------------------------------------------------------------- orders
#[tauri::command]
pub fn create_order(state: State<AppState>, order: Order) -> CmdResult<i64> {
    let order_id = state.db.create_order(&order).map_err(map_err)?;
    let restaurant = state.db.get_restaurant().map_err(map_err)?;
    if restaurant.notify_on_payment {
        let items_desc: Vec<String> = order.items.iter().map(|i| format!("{}× {}", i.quantity, i.name)).collect();
        let note = Notification {
            id: None,
            kind: "payment".into(),
            title: format!(
                "Payment {} — Table {} — {}{:.2}",
                title_case(&order.payment_status),
                if order.table_id.is_empty() { "?".into() } else { order.table_id.clone() },
                restaurant.currency_symbol, order.total
            ),
            body: format!("Order #{}. Items: {}.", order_id, items_desc.join(", ")),
            order_id: Some(order_id),
            read: false,
            created_at: String::new(),
        };
        let _ = state.db.add_notification(&note);
    }
    Ok(order_id)
}

fn title_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
    }
}

#[tauri::command]
pub fn get_order(state: State<AppState>, id: i64) -> CmdResult<Option<Order>> {
    state.db.get_order(id).map_err(map_err)
}

#[tauri::command]
pub fn get_orders(state: State<AppState>, status: Option<String>, table_id: Option<String>) -> CmdResult<Vec<Order>> {
    state.db.get_orders(status.as_deref(), table_id.as_deref()).map_err(map_err)
}

#[tauri::command]
pub fn update_order_status(state: State<AppState>, id: i64, status: String) -> CmdResult<()> {
    state.db.update_order_status(id, &status).map_err(map_err)
}

#[tauri::command]
pub fn set_payment(state: State<AppState>, id: i64, method: String, status: String) -> CmdResult<()> {
    state.db.set_payment(id, &method, &status).map_err(map_err)
}

#[tauri::command]
pub fn set_kitchen_message(state: State<AppState>, id: i64, message: String) -> CmdResult<()> {
    state.db.set_kitchen_message(id, &message).map_err(map_err)
}

// ---------------------------------------------------------------- reporting
#[tauri::command]
pub fn dashboard_summary(state: State<AppState>) -> CmdResult<HashMap<String, HashMap<String, f64>>> {
    state.db.dashboard_summary().map_err(map_err)
}

#[tauri::command]
pub fn sales_by_day(state: State<AppState>, days: i64) -> CmdResult<Vec<HashMap<String, serde_json::Value>>> {
    state.db.sales_by_day(days).map_err(map_err)
}

#[tauri::command]
pub fn sales_by_table(state: State<AppState>) -> CmdResult<Vec<HashMap<String, serde_json::Value>>> {
    state.db.sales_by_table().map_err(map_err)
}

#[tauri::command]
pub fn sales_by_period(state: State<AppState>, period: String, buckets: i64) -> CmdResult<Vec<HashMap<String, serde_json::Value>>> {
    state.db.sales_by_period(&period, buckets).map_err(map_err)
}

// ----------------------------------------------------------- branding/files
/// Opens the OS's native file-open dialog and blocks until the user picks a
/// file or cancels. This is the direct replacement for the old Flet
/// `FilePicker` — no custom modal, no manual-path-entry fallback needed,
/// since every desktop platform Tauri targets always has a native picker.
#[tauri::command]
pub fn pick_image_file(app: AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "svg"])
        .set_title("Choose an image")
        .blocking_pick_file()
        .map(|f| f.to_string())
}

#[tauri::command]
pub fn copy_logo(state: State<AppState>, src_path: String) -> CmdResult<String> {
    let dest = state
        .paths
        .copy_into(&state.paths.logo_dir, std::path::Path::new(&src_path))
        .map_err(|e| format!("Could not import image: {e}"))?;
    Ok(dest.to_string_lossy().to_string())
}

#[tauri::command]
pub fn copy_background(state: State<AppState>, src_path: String) -> CmdResult<String> {
    let dest = state
        .paths
        .copy_into(&state.paths.backgrounds_dir, std::path::Path::new(&src_path))
        .map_err(|e| format!("Could not import image: {e}"))?;
    Ok(dest.to_string_lossy().to_string())
}

#[tauri::command]
pub fn copy_menu_image(state: State<AppState>, src_path: String) -> CmdResult<String> {
    let dest = state
        .paths
        .copy_into(&state.paths.menu_images_dir, std::path::Path::new(&src_path))
        .map_err(|e| format!("Could not import image: {e}"))?;
    Ok(dest.to_string_lossy().to_string())
}

#[tauri::command]
pub fn list_gallery_images(state: State<AppState>) -> Vec<String> {
    state.paths.gallery_files().into_iter().map(|p| p.to_string_lossy().to_string()).collect()
}

#[tauri::command]
pub fn list_background_images(state: State<AppState>) -> Vec<String> {
    std::fs::read_dir(&state.paths.backgrounds_dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------- printing
#[derive(serde::Serialize)]
pub struct PrintResultDto {
    pub saved_path: String,
    pub printed: bool,
    pub message: String,
}

#[tauri::command]
pub fn print_order_ticket(state: State<AppState>, order_id: i64) -> CmdResult<PrintResultDto> {
    let order = state.db.get_order(order_id).map_err(map_err)?.ok_or("Order not found.")?;
    let restaurant = state.db.get_restaurant().map_err(map_err)?;
    let cfg = state.cfg.lock().unwrap().clone();
    let result = printing::print_ticket(&order, &restaurant, true, &cfg, &state.paths.tickets_dir);
    Ok(PrintResultDto { saved_path: result.saved_path, printed: result.printed, message: result.message })
}

// -------------------------------------------------------------------- seed
#[tauri::command]
pub fn ensure_seeded(state: State<AppState>) -> CmdResult<()> {
    waitress_core::seed::seed(&state.db, "The Copper Table", false).map_err(map_err)
}
