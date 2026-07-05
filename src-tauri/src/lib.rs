mod commands;
mod config;
mod paths;
mod state;

use std::sync::Mutex;

use tauri::Manager;

use crate::paths::Paths;
use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let resolved_paths = Paths::resolve(&app_handle)?;
            let db = waitress_core::Database::open(&resolved_paths.db_path)?;
            waitress_core::seed::seed(&db, "The Copper Table", false)?;
            let cfg = config::load(&resolved_paths.config_path);

            app.manage(AppState {
                db,
                paths: resolved_paths,
                cfg: Mutex::new(cfg),
                admin_session: Mutex::new(None),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_device_config,
            commands::update_device_config,
            commands::get_restaurant,
            commands::save_restaurant,
            commands::get_categories,
            commands::add_category,
            commands::update_category,
            commands::delete_category,
            commands::get_menu_items,
            commands::get_menu_item,
            commands::add_menu_item,
            commands::update_menu_item,
            commands::set_item_availability,
            commands::delete_menu_item,
            commands::get_active_specials,
            commands::get_all_specials,
            commands::add_special,
            commands::update_special,
            commands::delete_special,
            commands::is_activated,
            commands::device_id,
            commands::activate,
            commands::login,
            commands::logout,
            commands::current_admin,
            commands::get_admins,
            commands::create_admin,
            commands::update_admin,
            commands::set_admin_active,
            commands::delete_admin,
            commands::reset_admin_password,
            commands::get_notifications,
            commands::unread_notification_count,
            commands::mark_notification_read,
            commands::mark_all_notifications_read,
            commands::create_order,
            commands::get_order,
            commands::get_orders,
            commands::update_order_status,
            commands::set_payment,
            commands::set_kitchen_message,
            commands::dashboard_summary,
            commands::sales_by_day,
            commands::sales_by_table,
            commands::sales_by_period,
            commands::pick_image_file,
            commands::copy_logo,
            commands::copy_background,
            commands::copy_menu_image,
            commands::list_gallery_images,
            commands::list_background_images,
            commands::print_order_ticket,
            commands::ensure_seeded,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
