//! Demo-data seeding — port of `waitress/shared/seed.py`. No-op if a menu
//! already exists (idempotent), so it's safe to call on every app boot.

use crate::db::Database;
use crate::models::{DailySpecial, MenuCategory, MenuItem, RestaurantSettings};

struct Item {
    name: &'static str,
    desc: &'static str,
    price: f64,
}

fn menu() -> Vec<(&'static str, bool, Vec<Item>)> {
    vec![
        ("Starters", true, vec![
            Item { name: "Bruschetta", desc: "Grilled sourdough, vine tomatoes, basil, olive oil", price: 7.5 },
            Item { name: "Calamari", desc: "Lightly fried squid, lemon aioli", price: 9.0 },
            Item { name: "Garlic Prawns", desc: "Sizzling prawns in garlic butter", price: 11.0 },
        ]),
        ("Soups", true, vec![
            Item { name: "Tomato Basil", desc: "Slow-roasted tomato, cream, fresh basil", price: 6.5 },
            Item { name: "French Onion", desc: "Caramelised onion, gruyère crouton", price: 7.0 },
        ]),
        ("Appetizers", true, vec![
            Item { name: "Stuffed Mushrooms", desc: "Herbed cheese, breadcrumb crust", price: 8.0 },
            Item { name: "Chicken Wings", desc: "Buffalo glaze, blue cheese dip", price: 9.5 },
            Item { name: "Nachos Supreme", desc: "Cheese, jalapeños, guacamole, salsa", price: 8.5 },
        ]),
        ("Mains", false, vec![
            Item { name: "Ribeye Steak", desc: "300g ribeye, peppercorn sauce, fries", price: 26.0 },
            Item { name: "Grilled Salmon", desc: "Atlantic salmon, lemon butter, greens", price: 22.0 },
            Item { name: "Margherita Pizza", desc: "San Marzano tomato, mozzarella, basil", price: 15.0 },
            Item { name: "Mushroom Risotto", desc: "Arborio rice, wild mushrooms, parmesan", price: 17.0 },
        ]),
        ("Desserts", false, vec![
            Item { name: "Tiramisu", desc: "Espresso-soaked sponge, mascarpone", price: 7.5 },
            Item { name: "Lava Cake", desc: "Warm chocolate, vanilla ice cream", price: 8.0 },
        ]),
        ("Drinks", false, vec![
            Item { name: "Fresh Lemonade", desc: "House-made, lightly sparkling", price: 4.0 },
            Item { name: "Espresso", desc: "Double shot, single origin", price: 3.0 },
        ]),
    ]
}

pub fn seed(db: &Database, restaurant_name: &str, force: bool) -> rusqlite::Result<()> {
    let existing: i64 = {
        let conn = db.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM menu_items", [], |r| r.get(0))?
    };
    if existing > 0 && !force {
        return Ok(());
    }
    if force {
        let conn = db.conn.lock().unwrap();
        conn.execute_batch(
            "DELETE FROM order_items; DELETE FROM orders;
             DELETE FROM daily_specials; DELETE FROM menu_items;
             DELETE FROM categories;",
        )?;
    }

    let settings = RestaurantSettings {
        name: restaurant_name.to_string(),
        tagline: "Comfort classics, beautifully done".into(),
        currency: "USD".into(),
        currency_symbol: "$".into(),
        tax_rate: 0.08,
        voice_greeting_enabled: true,
        notify_on_payment: true,
        payment_gateways: "cash,card".into(),
        payment_gateways_all: "cash,card,stripe,Credit,Debit,PayPal".into(),
        ..RestaurantSettings::default()
    };
    db.save_restaurant(&settings)?;

    let mut ribeye_id: Option<i64> = None;
    let mut salmon_id: Option<i64> = None;

    for (order, (cat_name, is_starter, items)) in menu().into_iter().enumerate() {
        let cat_id = db.add_category(&MenuCategory {
            id: None,
            name: cat_name.to_string(),
            sort_order: order as i64,
            is_starter_group: is_starter,
        })?;
        for (i, item) in items.into_iter().enumerate() {
            let id = db.add_menu_item(&MenuItem {
                id: None,
                category_id: cat_id,
                name: item.name.to_string(),
                description: item.desc.to_string(),
                price: item.price,
                image_path: String::new(),
                available: true,
                sort_order: i as i64,
            })?;
            if item.name == "Ribeye Steak" {
                ribeye_id = Some(id);
            } else if item.name == "Grilled Salmon" {
                salmon_id = Some(id);
            }
        }
    }

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    db.add_special(&DailySpecial {
        id: None,
        menu_item_id: ribeye_id,
        title: "Chef's Ribeye".into(),
        description: "300g ribeye with peppercorn sauce & truffle fries".into(),
        price: 24.0,
        image_path: String::new(),
        active: true,
        active_date: today,
    })?;
    db.add_special(&DailySpecial {
        id: None,
        menu_item_id: salmon_id,
        title: "Catch of the Day".into(),
        description: "Atlantic salmon, lemon butter, seasonal greens".into(),
        price: 19.5,
        image_path: String::new(),
        active: true,
        active_date: String::new(),
    })?;

    Ok(())
}
