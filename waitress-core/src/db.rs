//! SQLite data access layer — a faithful Rust port of
//! `waitress/shared/database.py`. One SQLite file per device, WAL mode,
//! same table/column names so the schema is a 1:1 mirror of the Python app.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use chrono::Local;
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::models::*;

fn now() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Database { conn: Mutex::new(conn) };
        db.create_schema()?;
        Ok(db)
    }

    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let db = Database { conn: Mutex::new(conn) };
        db.create_schema()?;
        Ok(db)
    }

    // ------------------------------------------------------------ schema
    fn create_schema(&self) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS restaurant (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                name TEXT NOT NULL DEFAULT 'Your Restaurant',
                tagline TEXT DEFAULT '',
                logo_path TEXT DEFAULT '',
                active_background TEXT DEFAULT '',
                currency TEXT DEFAULT 'USD',
                currency_symbol TEXT DEFAULT '$',
                tax_rate REAL DEFAULT 0,
                printing_enabled INTEGER DEFAULT 0,
                kitchen_display_mode TEXT DEFAULT 'tablet',
                voice_greeting_enabled INTEGER DEFAULT 1,
                notify_on_payment INTEGER DEFAULT 1,
                payment_gateways TEXT DEFAULT 'cash',
                updated_at TEXT
            );

            CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                sort_order INTEGER DEFAULT 0,
                is_starter_group INTEGER DEFAULT 0,
                updated_at TEXT
            );

            CREATE TABLE IF NOT EXISTS menu_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                description TEXT DEFAULT '',
                price REAL NOT NULL DEFAULT 0,
                image_path TEXT DEFAULT '',
                available INTEGER DEFAULT 1,
                sort_order INTEGER DEFAULT 0,
                updated_at TEXT
            );

            CREATE TABLE IF NOT EXISTS daily_specials (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                menu_item_id INTEGER REFERENCES menu_items(id) ON DELETE SET NULL,
                title TEXT NOT NULL,
                description TEXT DEFAULT '',
                price REAL DEFAULT 0,
                image_path TEXT DEFAULT '',
                active INTEGER DEFAULT 1,
                active_date TEXT DEFAULT '',
                updated_at TEXT
            );

            CREATE TABLE IF NOT EXISTS orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                table_id TEXT DEFAULT '',
                device_id TEXT DEFAULT '',
                status TEXT DEFAULT 'NEW',
                payment_status TEXT DEFAULT 'UNPAID',
                payment_method TEXT DEFAULT '',
                subtotal REAL DEFAULT 0,
                tax REAL DEFAULT 0,
                total REAL DEFAULT 0,
                kitchen_message TEXT DEFAULT '',
                created_at TEXT,
                updated_at TEXT
            );

            CREATE TABLE IF NOT EXISTS order_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_id INTEGER NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
                menu_item_id INTEGER,
                name TEXT DEFAULT '',
                unit_price REAL DEFAULT 0,
                quantity INTEGER DEFAULT 1,
                notes TEXT DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS admin_users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                role TEXT DEFAULT 'admin',
                active INTEGER DEFAULT 1,
                created_at TEXT,
                last_login TEXT,
                updated_at TEXT
            );

            CREATE TABLE IF NOT EXISTS license (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL,
                device_id TEXT NOT NULL,
                license_id INTEGER DEFAULT 0,
                status TEXT DEFAULT 'ACTIVE',
                activated_at TEXT,
                updated_at TEXT
            );

            CREATE TABLE IF NOT EXISTS notifications (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                kind TEXT DEFAULT 'payment',
                title TEXT DEFAULT '',
                body TEXT DEFAULT '',
                order_id INTEGER,
                read INTEGER DEFAULT 0,
                created_at TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_items_category ON menu_items(category_id);
            CREATE INDEX IF NOT EXISTS idx_orderitems_order ON order_items(order_id);
            CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
            CREATE INDEX IF NOT EXISTS idx_orders_created ON orders(created_at);
            CREATE INDEX IF NOT EXISTS idx_notifications_read ON notifications(read);
            "#,
        )?;

        self.migrate(&conn)?;

        let exists: Option<i64> = conn
            .query_row("SELECT 1 FROM restaurant WHERE id = 1", [], |r| r.get(0))
            .optional()?;
        if exists.is_none() {
            conn.execute(
                "INSERT INTO restaurant (id, updated_at) VALUES (1, ?1)",
                params![now()],
            )?;
        }
        Ok(())
    }

    fn migrate(&self, conn: &Connection) -> rusqlite::Result<()> {
        Self::ensure_column(conn, "restaurant", "kitchen_display_mode", "TEXT DEFAULT 'tablet'")?;
        Self::ensure_column(conn, "restaurant", "address", "TEXT DEFAULT ''")?;
        Self::ensure_column(conn, "restaurant", "phone", "TEXT DEFAULT ''")?;
        Self::ensure_column(conn, "restaurant", "email", "TEXT DEFAULT ''")?;
        Self::ensure_column(conn, "restaurant", "payment_gateways_all", "TEXT DEFAULT ''")?;
        Self::ensure_column(conn, "restaurant", "payment_gateway_config", "TEXT DEFAULT '{}'")?;

        conn.execute(
            "UPDATE restaurant SET payment_gateways_all = payment_gateways \
             WHERE COALESCE(payment_gateways_all,'') = ''",
            [],
        )?;
        conn.execute(
            "UPDATE restaurant SET kitchen_display_mode='both' \
             WHERE printing_enabled=1 AND kitchen_display_mode='tablet'",
            [],
        )?;
        Ok(())
    }

    fn ensure_column(conn: &Connection, table: &str, column: &str, decl: &str) -> rusqlite::Result<()> {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
        let cols: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .collect();
        if !cols.iter().any(|c| c == column) {
            conn.execute(&format!("ALTER TABLE {table} ADD COLUMN {column} {decl}"), [])?;
        }
        Ok(())
    }

    // --------------------------------------------------------- restaurant
    pub fn get_restaurant(&self) -> rusqlite::Result<RestaurantSettings> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT * FROM restaurant WHERE id = 1", [], |row| {
            Ok(RestaurantSettings {
                name: row.get("name")?,
                tagline: row.get("tagline")?,
                address: row.get("address")?,
                phone: row.get("phone")?,
                email: row.get("email")?,
                logo_path: row.get("logo_path")?,
                active_background: row.get("active_background")?,
                currency: row.get("currency")?,
                currency_symbol: row.get("currency_symbol")?,
                tax_rate: row.get("tax_rate")?,
                printing_enabled: row.get::<_, i64>("printing_enabled")? != 0,
                kitchen_display_mode: row.get("kitchen_display_mode")?,
                voice_greeting_enabled: row.get::<_, i64>("voice_greeting_enabled")? != 0,
                notify_on_payment: row.get::<_, i64>("notify_on_payment")? != 0,
                payment_gateways: row.get("payment_gateways")?,
                payment_gateways_all: row.get("payment_gateways_all")?,
                payment_gateway_config: row.get("payment_gateway_config")?,
            })
        })
    }

    pub fn save_restaurant(&self, s: &RestaurantSettings) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE restaurant SET
                name=?1, tagline=?2, address=?3, phone=?4, email=?5, logo_path=?6,
                active_background=?7, currency=?8, currency_symbol=?9, tax_rate=?10,
                printing_enabled=?11, kitchen_display_mode=?12, voice_greeting_enabled=?13,
                notify_on_payment=?14, payment_gateways=?15, payment_gateways_all=?16,
                payment_gateway_config=?17, updated_at=?18
             WHERE id = 1",
            params![
                s.name, s.tagline, s.address, s.phone, s.email, s.logo_path,
                s.active_background, s.currency, s.currency_symbol, s.tax_rate,
                s.printing_enabled as i64, s.kitchen_display_mode,
                s.voice_greeting_enabled as i64, s.notify_on_payment as i64,
                s.payment_gateways, s.payment_gateways_all,
                s.payment_gateway_config, now(),
            ],
        )?;
        Ok(())
    }

    // --------------------------------------------------------- categories
    pub fn get_categories(&self) -> rusqlite::Result<Vec<MenuCategory>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM categories ORDER BY sort_order, name")?;
        let rows = stmt.query_map([], Self::row_to_category)?;
        rows.collect()
    }

    fn row_to_category(r: &Row) -> rusqlite::Result<MenuCategory> {
        Ok(MenuCategory {
            id: r.get("id")?,
            name: r.get("name")?,
            sort_order: r.get("sort_order")?,
            is_starter_group: r.get::<_, i64>("is_starter_group")? != 0,
        })
    }

    pub fn add_category(&self, cat: &MenuCategory) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO categories (name, sort_order, is_starter_group, updated_at) VALUES (?1,?2,?3,?4)",
            params![cat.name, cat.sort_order, cat.is_starter_group as i64, now()],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_category(&self, cat: &MenuCategory) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE categories SET name=?1, sort_order=?2, is_starter_group=?3, updated_at=?4 WHERE id=?5",
            params![cat.name, cat.sort_order, cat.is_starter_group as i64, now(), cat.id],
        )?;
        Ok(())
    }

    pub fn delete_category(&self, id: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM categories WHERE id=?1", params![id])?;
        Ok(())
    }

    // --------------------------------------------------------- menu items
    fn row_to_item(r: &Row) -> rusqlite::Result<MenuItem> {
        Ok(MenuItem {
            id: r.get("id")?,
            category_id: r.get("category_id")?,
            name: r.get("name")?,
            description: r.get("description")?,
            price: r.get("price")?,
            image_path: r.get("image_path")?,
            available: r.get::<_, i64>("available")? != 0,
            sort_order: r.get("sort_order")?,
        })
    }

    pub fn get_menu_items(&self, category_id: Option<i64>, available_only: bool) -> rusqlite::Result<Vec<MenuItem>> {
        let conn = self.conn.lock().unwrap();
        let mut sql = String::from("SELECT * FROM menu_items");
        let mut clauses: Vec<String> = Vec::new();
        if category_id.is_some() {
            clauses.push("category_id = ?1".into());
        }
        if available_only {
            clauses.push("available = 1".into());
        }
        if !clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&clauses.join(" AND "));
        }
        sql.push_str(" ORDER BY sort_order, name");
        let mut stmt = conn.prepare(&sql)?;
        let rows = if let Some(cid) = category_id {
            stmt.query_map(params![cid], Self::row_to_item)?.collect()
        } else {
            stmt.query_map([], Self::row_to_item)?.collect()
        };
        rows
    }

    pub fn get_menu_item(&self, id: i64) -> rusqlite::Result<Option<MenuItem>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT * FROM menu_items WHERE id=?1", params![id], Self::row_to_item)
            .optional()
    }

    pub fn add_menu_item(&self, item: &MenuItem) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO menu_items (category_id, name, description, price, image_path, available, sort_order, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![item.category_id, item.name, item.description, item.price,
                    item.image_path, item.available as i64, item.sort_order, now()],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_menu_item(&self, item: &MenuItem) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE menu_items SET category_id=?1, name=?2, description=?3, price=?4,
             image_path=?5, available=?6, sort_order=?7, updated_at=?8 WHERE id=?9",
            params![item.category_id, item.name, item.description, item.price,
                    item.image_path, item.available as i64, item.sort_order, now(), item.id],
        )?;
        Ok(())
    }

    pub fn set_item_availability(&self, id: i64, available: bool) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE menu_items SET available=?1, updated_at=?2 WHERE id=?3",
            params![available as i64, now(), id],
        )?;
        Ok(())
    }

    pub fn delete_menu_item(&self, id: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM menu_items WHERE id=?1", params![id])?;
        Ok(())
    }

    // ----------------------------------------------------- daily specials
    fn row_to_special(r: &Row) -> rusqlite::Result<DailySpecial> {
        Ok(DailySpecial {
            id: r.get("id")?,
            menu_item_id: r.get("menu_item_id")?,
            title: r.get("title")?,
            description: r.get("description")?,
            price: r.get("price")?,
            image_path: r.get("image_path")?,
            active: r.get::<_, i64>("active")? != 0,
            active_date: r.get("active_date")?,
        })
    }

    pub fn get_active_specials(&self, on_date: &str) -> rusqlite::Result<Vec<DailySpecial>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT * FROM daily_specials WHERE active = 1
             AND (active_date = '' OR active_date = ?1) ORDER BY id",
        )?;
        let rows = stmt.query_map(params![on_date], Self::row_to_special)?;
        rows.collect()
    }

    pub fn get_all_specials(&self) -> rusqlite::Result<Vec<DailySpecial>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM daily_specials ORDER BY id")?;
        let rows = stmt.query_map([], Self::row_to_special)?;
        rows.collect()
    }

    pub fn add_special(&self, sp: &DailySpecial) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO daily_specials (menu_item_id, title, description, price, image_path, active, active_date, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![sp.menu_item_id, sp.title, sp.description, sp.price, sp.image_path,
                    sp.active as i64, sp.active_date, now()],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_special(&self, sp: &DailySpecial) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE daily_specials SET menu_item_id=?1, title=?2, description=?3, price=?4,
             image_path=?5, active=?6, active_date=?7, updated_at=?8 WHERE id=?9",
            params![sp.menu_item_id, sp.title, sp.description, sp.price, sp.image_path,
                    sp.active as i64, sp.active_date, now(), sp.id],
        )?;
        Ok(())
    }

    pub fn delete_special(&self, id: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM daily_specials WHERE id=?1", params![id])?;
        Ok(())
    }

    // --------------------------------------------------------------admins
    fn row_to_admin(r: &Row) -> rusqlite::Result<AdminUser> {
        Ok(AdminUser {
            id: r.get("id")?,
            username: r.get("username")?,
            password_hash: r.get("password_hash")?,
            role: r.get("role")?,
            active: r.get::<_, i64>("active")? != 0,
            created_at: r.get("created_at")?,
            last_login: r.get::<_, Option<String>>("last_login")?.unwrap_or_default(),
        })
    }

    pub fn add_admin(&self, username: &str, password_hash: &str, role: &str) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO admin_users (username, password_hash, role, active, created_at, updated_at)
             VALUES (?1,?2,?3,1,?4,?4)",
            params![username, password_hash, role, now()],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_admins(&self) -> rusqlite::Result<Vec<AdminUser>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM admin_users ORDER BY role, username")?;
        let rows = stmt.query_map([], Self::row_to_admin)?;
        rows.collect()
    }

    pub fn get_admin_by_username(&self, username: &str) -> rusqlite::Result<Option<AdminUser>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT * FROM admin_users WHERE username=?1", params![username], Self::row_to_admin)
            .optional()
    }

    pub fn set_admin_password(&self, id: i64, hash: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE admin_users SET password_hash=?1, updated_at=?2 WHERE id=?3",
                     params![hash, now(), id])?;
        Ok(())
    }

    pub fn set_admin_active(&self, id: i64, active: bool) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE admin_users SET active=?1, updated_at=?2 WHERE id=?3",
                     params![active as i64, now(), id])?;
        Ok(())
    }

    /// Rename an admin and/or change their role. Guarded in commands.rs
    /// (username-uniqueness and last-superadmin checks live at that layer,
    /// same as the other admin-management commands).
    pub fn update_admin(&self, id: i64, username: &str, role: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE admin_users SET username=?1, role=?2, updated_at=?3 WHERE id=?4",
            params![username, role, now(), id],
        )?;
        Ok(())
    }

    pub fn delete_admin(&self, id: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM admin_users WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn touch_admin_login(&self, id: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE admin_users SET last_login=?1 WHERE id=?2", params![now(), id])?;
        Ok(())
    }

    pub fn count_superadmins(&self, active_only: bool) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        let sql = if active_only {
            "SELECT COUNT(*) FROM admin_users WHERE role='superadmin' AND active=1"
        } else {
            "SELECT COUNT(*) FROM admin_users WHERE role='superadmin'"
        };
        conn.query_row(sql, [], |r| r.get(0))
    }

    // -------------------------------------------------------------license
    pub fn get_license(&self, device_id: Option<&str>) -> rusqlite::Result<Option<License>> {
        let conn = self.conn.lock().unwrap();
        let map = |r: &Row| {
            Ok(License {
                id: r.get("id")?,
                code: r.get("code")?,
                device_id: r.get("device_id")?,
                license_id: r.get("license_id")?,
                status: r.get("status")?,
                activated_at: r.get("activated_at")?,
            })
        };
        if let Some(dev) = device_id {
            conn.query_row(
                "SELECT * FROM license WHERE device_id=?1 AND status='ACTIVE' ORDER BY id DESC LIMIT 1",
                params![dev], map,
            ).optional()
        } else {
            conn.query_row(
                "SELECT * FROM license WHERE status='ACTIVE' ORDER BY id DESC LIMIT 1", [], map,
            ).optional()
        }
    }

    pub fn activate_license(&self, code: &str, device_id: &str, license_id: i64) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO license (code, device_id, license_id, status, activated_at, updated_at)
             VALUES (?1,?2,?3,'ACTIVE',?4,?4)",
            params![code, device_id, license_id, now()],
        )?;
        Ok(conn.last_insert_rowid())
    }

    // ------------------------------------------------------- notifications
    fn row_to_notification(r: &Row) -> rusqlite::Result<Notification> {
        Ok(Notification {
            id: r.get("id")?,
            kind: r.get("kind")?,
            title: r.get("title")?,
            body: r.get("body")?,
            order_id: r.get("order_id")?,
            read: r.get::<_, i64>("read")? != 0,
            created_at: r.get("created_at")?,
        })
    }

    pub fn add_notification(&self, note: &Notification) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO notifications (kind, title, body, order_id, read, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![note.kind, note.title, note.body, note.order_id, note.read as i64, now()],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_notifications(&self, unread_only: bool, limit: i64) -> rusqlite::Result<Vec<Notification>> {
        let conn = self.conn.lock().unwrap();
        let sql = if unread_only {
            "SELECT * FROM notifications WHERE read=0 ORDER BY id DESC LIMIT ?1"
        } else {
            "SELECT * FROM notifications ORDER BY id DESC LIMIT ?1"
        };
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params![limit], Self::row_to_notification)?;
        rows.collect()
    }

    pub fn unread_count(&self) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM notifications WHERE read=0", [], |r| r.get(0))
    }

    pub fn mark_notification_read(&self, id: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE notifications SET read=1 WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn mark_all_notifications_read(&self) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE notifications SET read=1 WHERE read=0", [])?;
        Ok(())
    }

    // -------------------------------------------------------------- orders
    pub fn create_order(&self, order: &Order) -> rusqlite::Result<i64> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let created_at = if order.created_at.is_empty() { now() } else { order.created_at.clone() };
        tx.execute(
            "INSERT INTO orders (table_id, device_id, status, payment_status, payment_method,
             subtotal, tax, total, kitchen_message, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![order.table_id, order.device_id, order.status, order.payment_status,
                    order.payment_method, order.subtotal, order.tax, order.total,
                    order.kitchen_message, created_at, now()],
        )?;
        let order_id = tx.last_insert_rowid();
        for it in &order.items {
            tx.execute(
                "INSERT INTO order_items (order_id, menu_item_id, name, unit_price, quantity, notes)
                 VALUES (?1,?2,?3,?4,?5,?6)",
                params![order_id, it.menu_item_id, it.name, it.unit_price, it.quantity, it.notes],
            )?;
        }
        tx.commit()?;
        Ok(order_id)
    }

    fn row_to_order(r: &Row) -> rusqlite::Result<Order> {
        Ok(Order {
            id: r.get("id")?,
            table_id: r.get("table_id")?,
            device_id: r.get("device_id")?,
            status: r.get("status")?,
            payment_status: r.get("payment_status")?,
            payment_method: r.get("payment_method")?,
            subtotal: r.get("subtotal")?,
            tax: r.get("tax")?,
            total: r.get("total")?,
            kitchen_message: r.get("kitchen_message")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
            items: Vec::new(),
        })
    }

    fn order_items(conn: &Connection, order_id: i64) -> rusqlite::Result<Vec<OrderItem>> {
        let mut stmt = conn.prepare("SELECT * FROM order_items WHERE order_id=?1 ORDER BY id")?;
        let rows = stmt.query_map(params![order_id], |r| {
            Ok(OrderItem {
                id: r.get("id")?,
                order_id: r.get("order_id")?,
                menu_item_id: r.get("menu_item_id")?,
                name: r.get("name")?,
                unit_price: r.get("unit_price")?,
                quantity: r.get("quantity")?,
                notes: r.get("notes")?,
            })
        })?;
        rows.collect()
    }

    pub fn get_order(&self, id: i64) -> rusqlite::Result<Option<Order>> {
        let conn = self.conn.lock().unwrap();
        let order = conn.query_row("SELECT * FROM orders WHERE id=?1", params![id], Self::row_to_order).optional()?;
        match order {
            Some(mut o) => {
                o.items = Self::order_items(&conn, id)?;
                Ok(Some(o))
            }
            None => Ok(None),
        }
    }

    pub fn get_orders(&self, status: Option<&str>, table_id: Option<&str>) -> rusqlite::Result<Vec<Order>> {
        let conn = self.conn.lock().unwrap();
        let mut sql = String::from("SELECT * FROM orders");
        let mut clauses: Vec<&str> = Vec::new();
        if status.is_some() {
            clauses.push("status = ?1");
        }
        if table_id.is_some() {
            clauses.push(if status.is_some() { "table_id = ?2" } else { "table_id = ?1" });
        }
        if !clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&clauses.join(" AND "));
        }
        sql.push_str(" ORDER BY created_at DESC");
        let mut stmt = conn.prepare(&sql)?;
        let mut orders: Vec<Order> = match (status, table_id) {
            (Some(s), Some(t)) => stmt.query_map(params![s, t], Self::row_to_order)?.collect::<Result<_, _>>()?,
            (Some(s), None) => stmt.query_map(params![s], Self::row_to_order)?.collect::<Result<_, _>>()?,
            (None, Some(t)) => stmt.query_map(params![t], Self::row_to_order)?.collect::<Result<_, _>>()?,
            (None, None) => stmt.query_map([], Self::row_to_order)?.collect::<Result<_, _>>()?,
        };
        for o in orders.iter_mut() {
            o.items = Self::order_items(&conn, o.id.unwrap())?;
        }
        Ok(orders)
    }

    pub fn update_order_status(&self, id: i64, status: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE orders SET status=?1, updated_at=?2 WHERE id=?3", params![status, now(), id])?;
        Ok(())
    }

    pub fn set_payment(&self, id: i64, method: &str, status: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE orders SET payment_method=?1, payment_status=?2, updated_at=?3 WHERE id=?4",
            params![method, status, now(), id],
        )?;
        Ok(())
    }

    pub fn set_kitchen_message(&self, id: i64, message: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE orders SET kitchen_message=?1, updated_at=?2 WHERE id=?3",
                     params![message, now(), id])?;
        Ok(())
    }

    // ----------------------------------------------------------- reporting
    const NOT_CANCELLED: &'static str = "status != 'CANCELLED'";

    pub fn sales_summary(&self, since_iso: Option<&str>) -> rusqlite::Result<HashMap<String, f64>> {
        let conn = self.conn.lock().unwrap();
        let sql = format!(
            "SELECT COUNT(*) n, COALESCE(SUM(total),0) rev, COALESCE(AVG(total),0) avg \
             FROM orders WHERE {}{}",
            Self::NOT_CANCELLED,
            if since_iso.is_some() { " AND created_at >= ?1" } else { "" }
        );
        let (n, rev, avg): (i64, f64, f64) = if let Some(s) = since_iso {
            conn.query_row(&sql, params![s], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        } else {
            conn.query_row(&sql, [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        };
        let mut out = HashMap::new();
        out.insert("orders".to_string(), n as f64);
        out.insert("revenue".to_string(), (rev * 100.0).round() / 100.0);
        out.insert("avg_order".to_string(), (avg * 100.0).round() / 100.0);
        Ok(out)
    }

    pub fn dashboard_summary(&self) -> rusqlite::Result<HashMap<String, HashMap<String, f64>>> {
        let today = Local::now().date_naive();
        let start_today = today.format("%Y-%m-%d").to_string();
        let weekday = today.format("%u").to_string().parse::<i64>().unwrap_or(1) - 1; // Mon=0
        let start_week = (today - chrono::Duration::days(weekday)).format("%Y-%m-%d").to_string();
        let start_month = today.format("%Y-%m-01").to_string();

        let mut out = HashMap::new();
        out.insert("today".to_string(), self.sales_summary(Some(&start_today))?);
        out.insert("week".to_string(), self.sales_summary(Some(&start_week))?);
        out.insert("month".to_string(), self.sales_summary(Some(&start_month))?);
        out.insert("all".to_string(), self.sales_summary(None)?);
        Ok(out)
    }

    pub fn sales_by_day(&self, days: i64) -> rusqlite::Result<Vec<HashMap<String, serde_json::Value>>> {
        let conn = self.conn.lock().unwrap();
        let sql = format!(
            "SELECT substr(created_at,1,10) d, COUNT(*) n, COALESCE(SUM(total),0) rev
             FROM orders WHERE {} GROUP BY d ORDER BY d DESC LIMIT ?1",
            Self::NOT_CANCELLED
        );
        let mut stmt = conn.prepare(&sql)?;
        let mut by_day: HashMap<String, (i64, f64)> = HashMap::new();
        let rows = stmt.query_map(params![days], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, f64>(2)?))
        })?;
        for row in rows {
            let (d, n, rev) = row?;
            by_day.insert(d, (n, rev));
        }
        let today = Local::now().date_naive();
        let mut out = Vec::new();
        for i in (0..days).rev() {
            let d = (today - chrono::Duration::days(i)).format("%Y-%m-%d").to_string();
            let (n, rev) = by_day.get(&d).cloned().unwrap_or((0, 0.0));
            let mut m = HashMap::new();
            m.insert("label".to_string(), serde_json::Value::String(d));
            m.insert("orders".to_string(), serde_json::json!(n));
            m.insert("revenue".to_string(), serde_json::json!((rev * 100.0).round() / 100.0));
            out.push(m);
        }
        Ok(out)
    }

    pub fn sales_by_table(&self) -> rusqlite::Result<Vec<HashMap<String, serde_json::Value>>> {
        let conn = self.conn.lock().unwrap();
        let sql = format!(
            "SELECT COALESCE(NULLIF(table_id,''),'(unknown)') t, COUNT(*) n, COALESCE(SUM(total),0) rev
             FROM orders WHERE {} GROUP BY t ORDER BY rev DESC",
            Self::NOT_CANCELLED
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, f64>(2)?))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (t, n, rev) = row?;
            let mut m = HashMap::new();
            m.insert("label".to_string(), serde_json::Value::String(t));
            m.insert("orders".to_string(), serde_json::json!(n));
            m.insert("revenue".to_string(), serde_json::json!((rev * 100.0).round() / 100.0));
            out.push(m);
        }
        Ok(out)
    }

    pub fn sales_by_period(&self, period: &str, buckets: i64) -> rusqlite::Result<Vec<HashMap<String, serde_json::Value>>> {
        let conn = self.conn.lock().unwrap();
        let fmt = if period == "month" { "%Y-%m" } else { "%Y-W%W" };
        let sql = format!(
            "SELECT strftime('{}', created_at) p, COUNT(*) n, COALESCE(SUM(total),0) rev
             FROM orders WHERE {} GROUP BY p ORDER BY p DESC LIMIT ?1",
            fmt, Self::NOT_CANCELLED
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![buckets], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, f64>(2)?))
        })?;
        let mut out: Vec<HashMap<String, serde_json::Value>> = Vec::new();
        for row in rows {
            let (p, n, rev) = row?;
            let mut m = HashMap::new();
            m.insert("label".to_string(), serde_json::Value::String(p));
            m.insert("orders".to_string(), serde_json::json!(n));
            m.insert("revenue".to_string(), serde_json::json!((rev * 100.0).round() / 100.0));
            out.push(m);
        }
        out.reverse();
        Ok(out)
    }
}
