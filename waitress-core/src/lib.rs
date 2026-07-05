pub mod auth;
pub mod db;
pub mod models;
pub mod printing;
pub mod seed;

pub use db::Database;

#[cfg(test)]
mod tests {
    use super::*;
    use models::*;

    fn fresh_db() -> Database {
        Database::open_in_memory().expect("open in-memory db")
    }

    #[test]
    fn restaurant_defaults_and_roundtrip() {
        let db = fresh_db();
        let mut r = db.get_restaurant().unwrap();
        assert_eq!(r.name, "Your Restaurant");
        r.name = "The Copper Table".into();
        r.tax_rate = 0.08;
        db.save_restaurant(&r).unwrap();
        let r2 = db.get_restaurant().unwrap();
        assert_eq!(r2.name, "The Copper Table");
        assert_eq!(r2.tax_rate, 0.08);
    }

    #[test]
    fn category_and_item_crud() {
        let db = fresh_db();
        let cat_id = db.add_category(&MenuCategory { id: None, name: "Mains".into(), sort_order: 0, is_starter_group: false }).unwrap();
        let item_id = db.add_menu_item(&MenuItem {
            id: None, category_id: cat_id, name: "Steak".into(), description: "".into(),
            price: 20.0, image_path: "".into(), available: true, sort_order: 0,
        }).unwrap();
        let items = db.get_menu_items(Some(cat_id), true).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Steak");

        db.set_item_availability(item_id, false).unwrap();
        let visible = db.get_menu_items(Some(cat_id), true).unwrap();
        assert_eq!(visible.len(), 0);

        db.delete_menu_item(item_id).unwrap();
        assert!(db.get_menu_item(item_id).unwrap().is_none());
    }

    #[test]
    fn specials_active_filtering() {
        let db = fresh_db();
        db.add_special(&DailySpecial {
            id: None, menu_item_id: None, title: "Soup of the day".into(), description: "".into(),
            price: 5.0, image_path: "".into(), active: true, active_date: "".into(),
        }).unwrap();
        let active = db.get_active_specials("2026-07-03").unwrap();
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn admin_lifecycle_and_superadmin_guard() {
        let db = fresh_db();
        let hash = auth::hash_password("admin");
        let id = db.add_admin("admin", &hash, "superadmin").unwrap();
        assert_eq!(db.count_superadmins(true).unwrap(), 1);

        let fetched = db.get_admin_by_username("admin").unwrap().unwrap();
        assert!(auth::verify_password("admin", &fetched.password_hash));
        assert!(!auth::verify_password("wrong", &fetched.password_hash));

        db.set_admin_active(id, false).unwrap();
        let fetched2 = db.get_admin_by_username("admin").unwrap().unwrap();
        assert!(!fetched2.active);
    }

    #[test]
    fn order_create_and_fetch_with_items() {
        let db = fresh_db();
        let order = Order {
            id: None, table_id: "T-01".into(), device_id: "DEV-1".into(),
            status: "NEW".into(), payment_status: "PENDING".into(), payment_method: "cash".into(),
            subtotal: 10.0, tax: 0.8, total: 10.8, kitchen_message: "".into(),
            created_at: "".into(), updated_at: "".into(),
            items: vec![OrderItem {
                id: None, order_id: None, menu_item_id: Some(1), name: "Steak".into(),
                unit_price: 10.0, quantity: 1, notes: "".into(),
            }],
        };
        let id = db.create_order(&order).unwrap();
        let fetched = db.get_order(id).unwrap().unwrap();
        assert_eq!(fetched.items.len(), 1);
        assert_eq!(fetched.status, "NEW");

        db.update_order_status(id, "ACCEPTED").unwrap();
        let fetched2 = db.get_order(id).unwrap().unwrap();
        assert_eq!(fetched2.status, "ACCEPTED");

        let by_status = db.get_orders(Some("ACCEPTED"), None).unwrap();
        assert_eq!(by_status.len(), 1);
    }

    #[test]
    fn seed_is_idempotent() {
        let db = fresh_db();
        seed::seed(&db, "The Copper Table", false).unwrap();
        let items_a = db.get_menu_items(None, false).unwrap().len();
        seed::seed(&db, "The Copper Table", false).unwrap();
        let items_b = db.get_menu_items(None, false).unwrap().len();
        assert_eq!(items_a, items_b);
        assert!(items_a > 0);
    }

    #[test]
    fn dashboard_summary_runs_without_error() {
        let db = fresh_db();
        seed::seed(&db, "The Copper Table", false).unwrap();
        let summary = db.dashboard_summary().unwrap();
        assert!(summary.contains_key("today"));
        assert!(summary.contains_key("all"));
    }
}
