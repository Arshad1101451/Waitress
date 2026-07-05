# Project Waitress — Tauri port

A rewrite of the Flet/Python "Project Waitress" suite (Customer, Admin,
Kitchen) as a Tauri (Rust) + Svelte app, built for the same feature set at a
fraction of the install size.

## Why this exists

The Flet build embedded a full Python interpreter *inside* a Flutter
runtime, which is what drove the 2GB APK problem in the original project.
Tauri uses the OS's built-in WebView instead of bundling a runtime, so
desktop installers land around 3-10MB and Android/iOS builds (Tauri 2
supports mobile) land around 15-25MB — no Python, no embedded Flutter engine.

## Project layout

```
waitress-tauri/
├── src/                  Svelte frontend (Customer, Admin, Kitchen, launcher)
│   └── lib/
│       ├── api.js        Every Tauri command, wrapped
│       ├── stores.js     Toasts, confirm dialogs, shared app state
│       ├── components/   Shared UI: buttons, cards, headers, nav dock
│       ├── customer/     Customer module
│       ├── admin/        Admin module
│       └── kitchen/      Kitchen module
├── src-tauri/            Rust app shell (Tauri commands, window, plugins)
│   ├── src/
│   │   ├── commands.rs   Every #[tauri::command] the frontend calls
│   │   ├── state.rs      App-wide state (DB handle, session, config)
│   │   ├── paths.rs      Per-OS data directory resolution
│   │   └── config.rs     Device-local JSON config (table id, kiosk PIN, ...)
│   ├── gallery/          10 bundled AI dining-hall background images
│   └── tauri.conf.json
└── waitress-core/        Pure Rust library: SQLite schema, models, auth,
                           ticket printing, demo-data seeding. No GUI
                           dependency — this is the fully unit-tested half
                           of the port (`cargo test` — 8/8 passing).
```

## Prerequisites (install these on your own machine — this can't be built
## inside the sandbox this was written in, see "What's verified" below)

- [Rust](https://rustup.rs) (stable toolchain)
- [Node.js](https://nodejs.org) 18+
- Platform build tools:
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Visual Studio Build Tools (Desktop C++ workload) + WebView2 (preinstalled on Win 11/most Win 10)
  - **Linux**: `webkit2gtk-4.1`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev` (see [Tauri's Linux prerequisites](https://tauri.app/start/prerequisites/))
  - **Android** (optional, for APK/AAB): Android Studio + SDK + NDK, then `rustup target add` the relevant `*-android` targets
  - **iOS** (optional, macOS only): Xcode + `rustup target add` the `*-apple-ios` targets

## First run

```bash
npm install
npm run tauri dev
```

This launches the app in a native window with hot-reload. A SQLite database
and demo menu are created automatically on first launch (same demo data as
the original Python app: "The Copper Table", the same category/item list).

## Building an installer

```bash
npm run tauri build                 # current desktop platform
npm run tauri build -- --target aarch64-linux-android   # after adding mobile targets
```

## Before shipping

- **Icons**: `src-tauri/icons/` currently has placeholder square PNGs.
  Run `npm run tauri icon /path/to/your-logo.png` with a real 1024×1024
  logo — this generates the full icon set (including `.icns`/`.ico`) that
  `tauri.conf.json` expects.
- **App identifier**: `src-tauri/tauri.conf.json`'s `identifier` is set to
  `com.projectwaitress.suite` — change it if you want a different bundle ID.

## What's verified vs. not

- `waitress-core` (the SQLite/business-logic layer): fully compiles and
  passes 8/8 unit tests (`cargo test -p waitress-core`) — this is the part
  most likely to have subtle bugs (schema, money math, admin/session rules,
  reporting queries), and it's been exercised directly.
- The Svelte frontend (`src/`): builds cleanly with `npm run build` (Vite +
  svelte-check), all 3 modules and every shared component compile without
  errors — only cosmetic accessibility lint warnings remain (missing ARIA
  roles on a few modal backdrops), which don't affect functionality.
- `src-tauri` (the Tauri glue crate): **could not be compiled in the
  sandbox this was built in**, because Tauri's Linux build needs
  `webkit2gtk`/`gtk3` system headers that require root to install, which
  wasn't available. The Rust code was written carefully and reviewed by hand
  (correct Tauri v2 API usage, matching command signatures against
  `src/lib/api.js`), but this is the one layer that hasn't been compiler-
  checked. Run `cargo check` (or just `npm run tauri dev`) on your own
  machine first — if there's a type mismatch, the compiler error will point
  at the exact line.

## Feature parity notes

Every module was ported from a detailed behavioral spec extracted from the
original Python app (button-by-button, dialog-by-dialog). A few things
changed for the better rather than being ported 1:1:

- **Voice greeting**: now uses the browser's Web Speech API instead of
  `pyttsx3` — works identically on every OS (the Python version only worked
  on desktop).
- **File picker**: uses Tauri's native OS file dialog directly — no more
  custom fallback dialog for a missing picker (the bug that plagued the
  Flet version).
- **Kiosk mode**: uses Tauri's window API (fullscreen/frameless/no-resize)
  best-effort; if unsupported on a given build target, the app just runs in
  a normal window rather than failing.
