use std::sync::Mutex;

use waitress_core::models::{AdminUser, DeviceConfig};
use waitress_core::Database;

use crate::paths::Paths;

/// One instance lives for the whole app process. `admin_session` is a plain
/// in-memory `Option` — this is what makes an Admin login "stay valid until
/// logout / app exit / the OS reclaims the process" (there's no token/expiry
/// to check, matching the behavior ported from `admin/service.py`).
pub struct AppState {
    pub db: Database,
    pub paths: Paths,
    pub cfg: Mutex<DeviceConfig>,
    pub admin_session: Mutex<Option<AdminUser>>,
}
