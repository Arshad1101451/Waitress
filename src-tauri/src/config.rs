//! Device-local JSON config — port of `waitress/shared/config.py`. Stored
//! next to the SQLite file, independent of the (shared/syncable) database.

use std::fs;
use std::path::Path;

use waitress_core::models::DeviceConfig;

fn random_device_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 4] = rng.gen();
    format!("DEV-{}", hex::encode_upper(bytes))
}

mod hex {
    pub fn encode_upper(bytes: [u8; 4]) -> String {
        bytes.iter().map(|b| format!("{:02X}", b)).collect()
    }
}

pub fn load(path: &Path) -> DeviceConfig {
    let mut cfg: DeviceConfig = fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    if cfg.device_id.is_empty() {
        cfg.device_id = random_device_id();
        let _ = save(path, &cfg);
    }
    cfg
}

pub fn save(path: &Path, cfg: &DeviceConfig) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(cfg).unwrap_or_default();
    fs::write(path, json)
}
