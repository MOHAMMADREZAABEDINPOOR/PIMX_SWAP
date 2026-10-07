# PIMXSWAP architecture

PIMXSWAP is a Windows 10 (1607+) / 11 desktop utility. All text stays on the device. Node is build tooling only; Rust owns conversion and Windows integration, and Tauri hosts a React settings/editor window on demand.

## Repository and dependencies

`src-tauri/src/{layout,detection,clipboard,hotkeys,config,app}.rs` separates Windows mechanisms from policy. `src/` contains a strict TypeScript UI, localized JSON dictionaries and CSS tokens. `language_profiles/` contains small curated linguistic hints. `assets/` contains the original vector brand mark; `scripts/` contains build and verification helpers.

Runtime dependencies: Tauri 2, serde/serde_json, windows (Win32 bindings), unicode-normalization (dead-key composition). React and Tauri's JavaScript IPC are the only UI runtime dependencies. No animation framework, network client, telemetry, cloud model or translation service is used.

## Windows mechanisms

* Layout discovery: GetKeyboardLayoutList, GetKeyboardLayout, GetWindowThreadProcessId, GetLocaleInfoW. HKLs remain internal integer handles, validated against loaded layouts on every request.
* Physical-key mapping: enumerate scan codes; MapVirtualKeyExW maps each scan to a layout-specific virtual key. ToUnicodeEx with flag 4 probes Shift, Caps Lock and AltGr combinations without mutating the Windows keyboard buffer. Reverse longest-match lookup reconstructs physical keys; target characters follow those same scans. Dead keys are represented explicitly and composed using Unicode normalization. Unmappable text is preserved and counted. Original keystrokes cannot always be uniquely recovered from text; ambiguities are deterministic and reported.
* Shortcuts: RegisterHotKey with MOD_NOREPEAT on a dedicated GetMessageW thread; configuration changes arrive via a thread message, not polling. Registration failures are surfaced and previous bindings restored.
* Selection replacement: bounded modifier-release wait, foreground HWND checks, SendInput Ctrl+C/Ctrl+V, clipboard sequence checks. A transaction snapshots supported HGLOBAL clipboard formats before touching the clipboard. Unsupported handle formats cause a safe refusal when restoration is enabled. Snapshot capture, read and write use RAII OpenClipboard/CloseClipboard and bounded retries. Clipboard changes from another app abort replacement/restoration instead of overwriting newer data. Clipboard snapshots and selected text never persist to disk.
* Startup: HKCU Run value with a quoted executable and --background. No elevation required. Single-instance named mutex prevents duplicate hotkeys.
* Tray: Tauri native menu; closing the editor destroys its WebView. Background launch creates no WebView. Tray clicks and ambiguous selections build the editor lazily. Selection candidates retain a bounded in-memory context; application requires a deliberate click and foreground verification.

## Detection policy

Local profiles score script validity, frequent characters, common words, bigrams and trigrams. Scores are heuristic evidence, not statistically calibrated probabilities. Direct mode is deterministic. Auto mode compares conversions with the unchanged input, preserves recognized correct words in mixed text, requires lexical evidence, a minimum score and a separation margin. Unknown languages and short ambiguous inputs go to the picker. Correct known input is left untouched. Preferred loaded layouts limit candidate generation.

## IPC and performance

Typed Tauri commands return serializable models or stable error codes. The UI initializes settings and loaded layouts once, explicitly refreshes layouts, and runs conversion only on user action. Notifications are short native tray balloons. No idle timers, keyboard hooks, continuous text analysis or network access. Pointer-driven CSS highlights run only during interaction; reduced-motion and hidden windows stop effects. Release uses LTO, one codegen unit, symbol stripping and size optimization.

## Design

An original pair of interlocking directional forms represents two physical key paths exchanging layouts. Mint is the brand accent; neutral glass surfaces and restrained metallic highlights support dark, light and system themes. UI language switches without restart, using logical CSS properties and bidi-isolated shortcuts. Editor text uses auto direction independently of interface language.

## Validation limits

Clipboard and SendInput behavior depends on the target application's copy/paste support, focus and Windows integrity level. Password fields, IMEs, secure desktops and elevated apps are not guaranteed. Test against Notepad and real target apps before distribution. Performance budgets must be measured on the built executable; no unmeasured RAM/CPU claims are made.
