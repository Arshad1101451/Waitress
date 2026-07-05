# Waitress — Tauri port

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
- `src-tauri` (the Tauri glue crate): confirmed compiling and running as a
  native macOS app (`npm run tauri dev`), all 3 modules working end to end.
  Windows and mobile (Android/iOS) builds haven't been produced yet — see
  "Building for other platforms" below.

## Building for other platforms

The desktop build above only produces an app for whatever OS you ran it on
(macOS, if that's your dev machine). Getting installers for Windows, an
Android tablet, and an iPad each need a different toolchain — here's the
exact path for each.

### Windows — via GitHub Actions (no Windows PC needed)

Tauri's Windows target needs the MSVC/WebView2 toolchain, which only exists
on Windows, so this repo builds it in the cloud instead:

1. Push this repo to GitHub (one-time setup):
   ```bash
   gh repo create waitress --private --source=. --remote=origin --push
   # — or, without the GitHub CLI —
   # create an empty repo at github.com/new, then:
   git remote add origin <the URL GitHub gives you>
   git push -u origin main
   ```
2. On GitHub, open the **Actions** tab → **Windows build** → **Run workflow**
   (it also runs automatically on every future push to `main`).
3. When the run finishes (~5–10 min), scroll to the bottom of that run page
   to **Artifacts** → download `windows-installer`. Unzip it — you'll get a
   `.msi` and/or a `.exe` (NSIS) installer. Copy that to the Windows
   tablet/PC and run it.

### Android tablet

Do this once, on your Mac:

```bash
# 1. Install Android Studio (https://developer.android.com/studio), then
#    open it once and let it install the default SDK via its setup wizard.

# 2. Add the Android NDK (Android Studio → Settings → Languages & Frameworks
#    → Android SDK → SDK Tools tab → check "NDK (Side by side)" → Apply).

# 3. Point Tauri at your SDK/NDK (add to ~/.zprofile, then `exec zsh`):
export ANDROID_HOME="$HOME/Library/Android/sdk"
export NDK_HOME="$ANDROID_HOME/ndk/$(ls "$ANDROID_HOME/ndk" | head -1)"

# 4. Rust targets for Android:
rustup target add aarch64-linux-android armv7-linux-androideabi \
  i686-linux-android x86_64-linux-android

# 5. Scaffold the Android project (generates src-tauri/gen/android):
cd waitress-tauri
npm run tauri android init
```

Then, to test on your actual tablet:

```bash
# Enable Developer Options + USB debugging on the tablet, connect via USB,
# accept the "allow USB debugging" prompt on the tablet, then:
npm run tauri android dev
```

This installs and launches the app directly on the connected tablet with
hot-reload. For a standalone APK you can install without a USB connection:

```bash
npm run tauri android build
# unsigned APK lands in:
# src-tauri/gen/android/app/build/outputs/apk/universal/release/
```

(An unsigned release APK needs "Install unknown apps" enabled in Android
settings when you sideload it. Signing it properly for wider distribution is
a later step — the Tauri docs cover keystore setup if you get there.)

### iPad (iOS)

Do this once, on your Mac:

```bash
# 1. Install Xcode from the App Store, then open it once to accept the
#    license and let it install additional components.
xcode-select --install   # command line tools, if not already present

# 2. Rust targets for iOS:
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios

# 3. Scaffold the iOS project (generates src-tauri/gen/apple):
cd waitress-tauri
npm run tauri ios init
```

Then, to test on your iPad:

```bash
# Connect the iPad via USB (or same WiFi network with Xcode's wireless
# debugging), trust the Mac on the iPad if prompted, then:
npm run tauri ios dev
```

The first run opens Xcode's signing prompt — under **Signing & Capabilities**,
set **Team** to your personal Apple ID (Xcode → Settings → Accounts → add
your Apple ID if it's not there yet; a free account is enough). With a free
account, the app installs and runs immediately, but the signing certificate
expires after **7 days** — after that, just re-run `npm run tauri ios dev`
(or open `src-tauri/gen/apple/*.xcodeproj` in Xcode and hit Run) to
re-sign and reinstall it. A paid Apple Developer account ($99/yr) removes
that 7-day limit and adds TestFlight for sharing builds without a cable.

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
