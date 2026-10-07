# Developer guide

## Core modules

| Module | Responsibility |
| --- | --- |
| `layout.rs` | Windows layout catalog, on-demand HKL loading, ISO language codes, canonical physical-key reverse tables, generic conversion |
| `detection.rs` | Embedded language profiles, script/frequency/n-gram/word evidence, candidate ranking, unchanged protection |
| `clipboard.rs` | Message-only clipboard owner, HGLOBAL snapshots, foreground checks, copy/paste transactions, timeout and race handling |
| `hotkeys.rs` | Parser, dedicated blocking message thread, collision detection and rollback |
| `config.rs` | Default/validated settings, durable atomic replacement, HKCU startup entry |
| `app.rs` | Tauri commands, synchronization, on-demand windows, pending selection, tray, notifications, single-instance guard |

Every Windows pointer crossing an unsafe boundary is limited to a documented owned buffer or validated OS handle. Clipboard allocations transfer to Windows only after successful SetClipboardData; RAII closes owners and clipboard locks. The conversion mutex serializes map-cache construction. A separate atomic guard prevents overlapping selection transactions; manual clipboard actions should be avoided while a global conversion is in progress.

## Commands

`bootstrap`, `refresh_layouts`, `convert_text`, `analyze_text`, `save_settings`, `read_clipboard`, `write_clipboard`, `set_paused`, `pending_selection`, `apply_selection`, `cancel_selection`, `window_action`, `set_compact`.

Conversion, analysis and configuration changes use blocking workers so the webview event loop remains responsive. Shortcut callbacks capture the foreground handle before spawning work. No worker polls when idle. Native notifications contain only generic status messages, never selected text.

Events `navigate`, `settings-changed`, `selection-ready`, and `operation-status` coordinate the UI. Listeners are removed on unmount. The browser preview does not implement substitute conversion tables.

## UI

`App.tsx` coordinates IPC, modes, page selection, text history, theme and notices. `Settings`, `About`, `Candidates`, `LayoutSelect`, and `Onboarding` isolate display concerns. UI string keys must exist in both dictionaries; TypeScript and the UI test check parity. Windows-provided layout names remain OS metadata. CSS uses logical padding/margins, input auto direction and isolated keyboard shortcuts. Page and onboarding transitions, dropdowns, interaction effects and five-second notices respect reduced motion. The custom title bar invokes native window controls; compact mode preserves editor state and restores previous window bounds.

`npm run test:workspace` uses mocked IPC for onboarding, keyboard search, notice expiry and compact layout checks. `scripts/test-native-ui.ps1` verifies actual native conversion, compact resizing and restoration, and the tray lifecycle. The ignored Rust `native_` clipboard tests verify selected text and a word at the caret with a temporary Windows edit control.

## Adding a language profile

Create a compact JSON profile containing the language code, Unicode script, frequent characters, common words, bigrams and trigrams; add it to `profiles()`. Add positive, negative and mixed-text tests. The threshold alone never authorizes replacement: lexical evidence, mapping coverage and candidate margin also gate Auto Fix. Do not describe the score as statistical certainty.

## Release hygiene

Keep Cargo.lock and package-lock.json. Run formatting, Clippy, Rust tests, UI smoke tests and a native clipboard check. Build using the Windows MSVC target. Validate installer/uninstaller in an isolated user account or VM, sign artifacts for public distribution, and benchmark the complete process tree. Use Windows UI Automation for native editor checks; shipped code does not enable remote debugging.
