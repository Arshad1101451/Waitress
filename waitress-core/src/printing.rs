//! Kitchen ticket formatting + printing backends — port of
//! `waitress/shared/printing.py`. Every print attempt first writes a durable
//! timestamped ticket file, then best-effort dispatches to a printer; failures
//! never propagate as errors (matches the original "never block the kitchen
//! board on a printer problem" behavior).

use std::fs;
use std::io::Write as _;
use std::net::TcpStream;
use std::path::Path;
use std::process::Command;

use chrono::Local;

use crate::models::{DeviceConfig, Order, RestaurantSettings};

const WIDTH: usize = 40;

fn center(s: &str) -> String {
    if s.len() >= WIDTH {
        return s.to_string();
    }
    let pad = (WIDTH - s.len()) / 2;
    format!("{}{}", " ".repeat(pad), s)
}

fn justify(left: &str, right: &str) -> String {
    let gap = WIDTH.saturating_sub(left.len() + right.len()).max(1);
    format!("{}{}{}", left, " ".repeat(gap), right)
}

pub fn should_print(r: &RestaurantSettings) -> bool {
    r.kitchen_display_mode == "printer" || r.kitchen_display_mode == "both"
}

pub fn shows_on_tablet(r: &RestaurantSettings) -> bool {
    r.kitchen_display_mode == "tablet" || r.kitchen_display_mode == "both"
}

pub fn format_ticket(order: &Order, restaurant: &RestaurantSettings) -> String {
    let mut lines = Vec::new();
    lines.push(center(&restaurant.name.to_uppercase()));
    lines.push(center("KITCHEN TICKET"));
    lines.push("=".repeat(WIDTH));
    lines.push(justify(
        &format!("Order #{}", order.id.unwrap_or(0)),
        &format!("Table {}", if order.table_id.is_empty() { "?".into() } else { order.table_id.clone() }),
    ));
    lines.push(justify(
        &Local::now().format("%Y-%m-%d %H:%M").to_string(),
        &order.payment_status,
    ));
    lines.push("-".repeat(WIDTH));
    for it in &order.items {
        let left = format!("{} x {}", it.quantity, it.name);
        let right = format!("{}{:.2}", restaurant.currency_symbol, it.line_total());
        lines.push(justify(&left, &right));
        if !it.notes.is_empty() {
            lines.push(format!("   >> {}", it.notes));
        }
    }
    lines.push("-".repeat(WIDTH));
    lines.push(justify("Subtotal", &format!("{}{:.2}", restaurant.currency_symbol, order.subtotal)));
    lines.push(justify("Tax", &format!("{}{:.2}", restaurant.currency_symbol, order.tax)));
    lines.push(justify("TOTAL", &format!("{}{:.2}", restaurant.currency_symbol, order.total)));
    lines.push("=".repeat(WIDTH));
    lines.push(center("*** PREPARE ***"));
    lines.push(String::new());
    lines.join("\n")
}

pub struct PrintResult {
    pub saved_path: String,
    pub printed: bool,
    pub message: String,
}

pub fn print_ticket(
    order: &Order,
    restaurant: &RestaurantSettings,
    to_printer: bool,
    cfg: &DeviceConfig,
    tickets_dir: &Path,
) -> PrintResult {
    let text = format_ticket(order, restaurant);
    let _ = fs::create_dir_all(tickets_dir);
    let filename = format!(
        "ticket-{}-{}.txt",
        order.id.unwrap_or(0),
        Local::now().format("%Y%m%d-%H%M%S")
    );
    let path = tickets_dir.join(filename);
    if let Err(e) = fs::write(&path, &text) {
        return PrintResult {
            saved_path: String::new(),
            printed: false,
            message: format!("Could not save ticket: {e}"),
        };
    }
    let saved_path = path.to_string_lossy().to_string();

    if !to_printer {
        return PrintResult { saved_path, printed: false, message: "Ticket saved (printing off).".into() };
    }

    if cfg.printer_backend == "network" && !cfg.printer_host.is_empty() {
        match send_escpos(&cfg.printer_host, cfg.printer_port, &text) {
            Ok(()) => PrintResult { saved_path, printed: true, message: format!("Sent to {}:{}", cfg.printer_host, cfg.printer_port) },
            Err(e) => PrintResult { saved_path, printed: false, message: format!("Network printer error: {e}") },
        }
    } else {
        print_via_os(&saved_path)
    }
}

fn send_escpos(host: &str, port: u16, text: &str) -> std::io::Result<()> {
    let mut stream = TcpStream::connect((host, port))?;
    stream.write_all(&[0x1B, 0x40])?; // ESC @ (init)
    let ascii: String = text.chars().map(|c| if c.is_ascii() { c } else { '?' }).collect();
    stream.write_all(ascii.as_bytes())?;
    stream.write_all(b"\n\n\n")?;
    stream.write_all(&[0x1D, 0x56, 0x00])?; // GS V 0 (full cut)
    Ok(())
}

#[cfg(target_os = "windows")]
fn print_via_os(path: &str) -> PrintResult {
    match Command::new("cmd").args(["/C", "start", "", "/MIN", path]).spawn() {
        Ok(_) => PrintResult { saved_path: path.into(), printed: true, message: "Sent to default printer.".into() },
        Err(e) => PrintResult { saved_path: path.into(), printed: false, message: format!("Print failed: {e}") },
    }
}

#[cfg(not(target_os = "windows"))]
fn print_via_os(path: &str) -> PrintResult {
    for cmd in ["lp", "lpr"] {
        if Command::new(cmd).arg(path).output().is_ok() {
            return PrintResult { saved_path: path.into(), printed: true, message: format!("Sent via {cmd}.") };
        }
    }
    PrintResult {
        saved_path: path.into(),
        printed: false,
        message: "No printer command (lp/lpr) found — ticket saved to file.".into(),
    }
}
