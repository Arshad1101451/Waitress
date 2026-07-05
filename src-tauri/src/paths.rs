//! Filesystem layout — port of `waitress/shared/paths.py`. Uses Tauri's own
//! per-OS app-data resolution (equivalent to the Python app's `platformdirs`
//! dependency) so this behaves the same on Windows/macOS/Linux and, once
//! Tauri mobile targets are built, Android/iOS sandboxed storage too.

use std::fs;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

pub struct Paths {
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
    pub config_path: PathBuf,
    pub assets_dir: PathBuf,
    pub backgrounds_dir: PathBuf,
    pub menu_images_dir: PathBuf,
    pub logo_dir: PathBuf,
    pub tickets_dir: PathBuf,
    pub gallery_dir: PathBuf,
}

impl Paths {
    pub fn resolve(app: &AppHandle) -> tauri::Result<Self> {
        let data_dir = app.path().app_data_dir()?;
        let assets_dir = data_dir.join("assets");
        let paths = Paths {
            db_path: data_dir.join("waitress.db"),
            config_path: data_dir.join("config.json"),
            backgrounds_dir: assets_dir.join("backgrounds"),
            menu_images_dir: assets_dir.join("menu_images"),
            logo_dir: assets_dir.join("logo"),
            tickets_dir: data_dir.join("tickets"),
            // Bundled read-only gallery of AI dining-hall backgrounds, shipped
            // as a Tauri resource (see tauri.conf.json `bundle.resources`).
            gallery_dir: app
                .path()
                .resource_dir()
                .map(|r| r.join("gallery"))
                .unwrap_or_else(|_| assets_dir.join("gallery")),
            assets_dir,
            data_dir,
        };
        paths.ensure_dirs();
        Ok(paths)
    }

    pub fn ensure_dirs(&self) {
        for d in [
            &self.data_dir,
            &self.assets_dir,
            &self.backgrounds_dir,
            &self.menu_images_dir,
            &self.logo_dir,
            &self.tickets_dir,
        ] {
            let _ = fs::create_dir_all(d);
        }
    }

    pub fn gallery_files(&self) -> Vec<PathBuf> {
        let exts = ["png", "jpg", "jpeg", "webp", "svg"];
        let mut out: Vec<PathBuf> = fs::read_dir(&self.gallery_dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| {
                        p.extension()
                            .and_then(|e| e.to_str())
                            .map(|e| exts.contains(&e.to_lowercase().as_str()))
                            .unwrap_or(false)
                    })
                    .collect()
            })
            .unwrap_or_default();
        out.sort();
        out
    }

    pub fn copy_into(&self, dest_dir: &Path, src: &Path) -> std::io::Result<PathBuf> {
        fs::create_dir_all(dest_dir)?;
        let filename = src
            .file_name()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "no filename"))?;
        let dest = dest_dir.join(filename);
        fs::copy(src, &dest)?;
        Ok(dest)
    }
}
