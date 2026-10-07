# PIMXSWAP

A Windows keyboard layout utility built with Rust, Tauri 2, React and TypeScript. It reconstructs physical keys from one Windows layout and renders those keys in another. `sghl → سلام` is keyboard correction. All conversion and language evidence stays on your device.

## Install and use

Run `release/PIMXSWAP-Setup.exe` when available. The installer supports current-user installation, upgrade, uninstall and Windows shortcuts. Windows 10 version 1607+ or Windows 11 x64 and Microsoft WebView2 are required. If WebView2 is missing, the installer downloads Microsoft's bootstrapper; the installed application's conversion works offline.

1. Complete the three-step setup and search and choose from the Windows keyboard layout catalog.
2. Select incorrectly typed text, or place the caret at the end of a line in another app, and press **Ctrl+Shift+Space**. Auto Fix uses conservative local language evidence; uncertain results open a picker.
3. **Ctrl+Alt+Shift+Space** swaps text using your configured layout pair. **Ctrl+Alt+P** opens the editor. All shortcuts can be changed in Settings.
4. In the editor, paste text, choose Auto Fix or Direct convert, and press **Ctrl+Enter**. Copy the result, swap the layout pair, or undo/redo input changes.

Use the compact-workspace button in the custom title bar to keep just the editors and conversion controls. Switching back restores the window size and position and preserves both texts. Navigation and onboarding use smooth motion, with reduced-motion support. In-app notices dismiss after five seconds.

Closing the window normally releases the WebView and keeps the tray/core running. Choose Exit from the tray to stop PIMXSWAP. Enable Launch at Windows startup for a window-free background launch. Pause disables conversion shortcuts while keeping the Open shortcut available.

## Development

Install Node.js 24 LTS, stable Rust (MSVC), Visual Studio 2022 Build Tools with Desktop development with C++ / Windows SDK, and WebView2. Node and npm are build tools, not application runtime dependencies.

```powershell
npm ci
npm run icons
npm run tauri dev
```

If this workspace has a `.tools` directory prepared by the initial build, dot-source `scripts/env.ps1` first. Those tools are local build dependencies and excluded from the product.

```powershell
. .\scripts\env.ps1
npm run build
cargo test --locked --manifest-path src-tauri/Cargo.toml -- --test-threads=1
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

`npm run dev` serves a browser preview. Windows integrations require `tauri dev` or the compiled executable; the browser preview reports this explicitly.

## Release

```powershell
powershell -ExecutionPolicy Bypass -File scripts/release.ps1
```

This verifies Rust, builds the frontend, compiles the optimized native binary and packages an NSIS installer. Output: `release/PIMXSWAP.exe`, `release/PIMXSWAP-Setup.exe`, SHA256 manifest. The executable needs the system WebView2 runtime. A public distribution should sign both executable and installer with a publisher certificate; unsigned builds may trigger SmartScreen.

For ARM64, install the ARM64 MSVC target and compatible compiler tools, then use `npm run tauri build -- --target aarch64-pc-windows-msvc`. x64 is the primary validated target.

## Architecture

See [architecture](docs/architecture.md) and [developer guide](docs/development.md). Rust owns native layouts, mapping, detection, configuration, global shortcuts, tray lifecycle and clipboard transactions. The frontend invokes typed Tauri commands. No Electron, embedded Node runtime, cloud translation, account system or telemetry.

The keyboard engine uses `GetKeyboardLayoutList`, scan codes, `MapVirtualKeyExW`, and `ToUnicodeEx`. Shift, Caps Lock, AltGr, ligatures and common dead-key compositions are modeled. Reverse mappings use a deterministic canonical key path because text does not record the original modifier state. Unmapped characters are retained. IMEs and arbitrary custom dead-key sequences are not fully reconstructible from text.

Local profiles cover English, Persian, Arabic, Russian, German, French and Spanish. Other Windows catalog layouts load on demand and work in Direct mode and can appear in the picker. Confidence is heuristic evidence, not a calibrated probability. Common correct words are preserved in mixed input. Unknown vocabulary, ambiguous Latin-script layouts, names and short text require review; the small profiles are deliberately conservative. A configured pair with different scripts also converts unfinished text by physical keys when no recognizable correct word indicates it should be preserved. This includes incomplete endings such as `sghl lk o,` → `سلام من خو`.

## Configuration and privacy

Settings are stored in `%APPDATA%/com.pimxswap.desktop/settings.json`. Only preferences persist: UI language, theme, shortcuts, layouts, threshold, startup, tray and effects. Clipboard snapshots, input and candidate text are never saved or logged. Diagnostics exposes operational error codes only. Startup uses the current-user Windows Run key. No admin privileges are requested.

## Clipboard and compatibility

Selection conversion captures supported global-memory clipboard formats, copies the selection, validates the foreground window and clipboard sequence, pastes Unicode text, then restores the previous clipboard when safe. Bitmap, palette and metafile formats are duplicated with their native Windows APIs. Nontransferable owner-display and empty bookkeeping formats are skipped without blocking text correction. Race checks preserve newer clipboard data. An active transaction uses bounded waits, including a 350 ms paste-read grace period; idle execution does not poll.

Applications with delayed clipboard reads, unusual copy/paste handlers, protected fields or higher integrity levels may not support automatic replacement. Use the internal editor and manual paste in those cases. A candidate selection expires after two minutes and must still match the original selected text when applied.

## Tests

Rust unit/integration tests exercise real OS mappings, multilingual sentences, numbers, punctuation, Shift, AltGr, dead keys, unmapped Unicode, language evidence, false positives, config validation and shortcuts. Integration tests temporarily load keyboard layouts into the test process without switching the user's layout.

For a real clipboard round trip (preserves the initial supported clipboard formats):

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib -- --ignored --test-threads=1
```

For browser UI checks, run `npx playwright install chromium`, `npm run dev`, then `npm run test:ui`. These verify localization parity, input history, themes and minimum-window RTL layout. Set `CHROME_PATH` to use an existing compatible Chrome installation. For native UI checks, use `powershell -ExecutionPolicy Bypass -File scripts/test-native-ui.ps1` with a fresh default configuration. Windows UI Automation exercises the actual editor, Tauri IPC, conversion and candidate picker. The test launches a temporary instance and restores the original settings afterward.

Measure release resources using `scripts/benchmark.ps1`. Inspect both the native process and descendant WebView2 processes. Background and editor-open results differ. Exact RAM/CPU budgets require measurements on the target system; see [validation report](docs/validation.md) for this build's evidence and remaining checks.

## Troubleshooting

* Shortcut reserved: change the shortcut in Settings; previous registered shortcuts are retained if an update fails.
* Layout missing: add the language's keyboard in Windows Settings, reload it, then Refresh in PIMXSWAP.
* Selected text unavailable: release modifier keys, keep the target focused, or manually copy into the editor.
* Clipboard cannot be preserved: save its current contents elsewhere or use the editor. Disabling restoration intentionally leaves converted text in the clipboard.
* Windows blocks input: protected/elevated applications may reject `SendInput`. Use manual copy/paste.
* A second process exits after asking the existing instance to open its editor. You can also open it from the tray or use the Open shortcut.
* Compiler/linker missing: install the C++ build tools and Windows SDK; restart your terminal after installing tools.

## فارسی

PIMXSWAP متن تایپ‌شده با چیدمان اشتباه را روی دستگاه خودتان اصلاح می‌کند. متن را انتخاب کنید و `Ctrl+Shift+Space` بزنید. برای تبدیل مستقیم، متن را داخل برنامه وارد کنید و چیدمان ورودی و مقصد را انتخاب کنید. زبان فارسی، راست‌به‌چپ، پوستهٔ تیره/روشن/سیستم و کاهش حرکت در تنظیمات در دسترس‌اند. نتیجه‌های مبهم پیش از جایگزینی به انتخاب شما نیاز دارند.
