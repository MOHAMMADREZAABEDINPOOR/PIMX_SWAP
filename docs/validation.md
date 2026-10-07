# Validation report — 2026-10-01

Validated on this Windows x64 workspace. The release artifacts are in `release/`; the executable and NSIS installer use the PIMXSWAP name, icon and 1.0.0 version metadata. SHA256 checksums accompany the binaries.

## Completed checks

* Strict TypeScript compilation and Vite production build passed.
* `cargo fmt --check` and Clippy `--all-targets -- -D warnings` passed.
* 14 ordinary Rust unit tests and 5 real Windows layout integration tests passed.
* 8 explicitly invoked native tests passed: Unicode clipboard round trip, selected text/word/unfinished-line SendInput replacement, bitmap and registered custom-format preservation, refusal to overwrite newer clipboard data, real global hotkey registration/callback/collision/rollback, and two conversions through the running application's real global shortcut. Tests requiring a running app initially failed when the installer check had stopped it; both passed after restarting it.
* Earlier build: real Chrome textarea/contenteditable tests and real Office COM text checks passed. Word and PowerPoint each passed selected Persian (`اثممخ` → `hello`), selected partial sentence and no-selection caret partial sentence (`sghl lk o,` → `سلام من خو`). Later repeat tests did not all pass; the most recent JSON results record foreground interruptions and are not a clean release pass. A clipboard-read publication wait was added, and all final native tests passed. Final Office/Chrome verification still requires a stable foreground session. Test fixtures use temporary unsaved Office documents and close them afterward.
* Installer install/reinstall/uninstall checks preserved the existing user's registration, shortcuts and settings. The packaged executable is byte-identical to the tested portable executable except for Tauri's expected three-byte `UNK` → `NSS` bundle marker. Bundled privacy, quick-start and dependency notices are checked against their source files.
* Real OS mappings passed English ↔ Persian, Arabic, Russian and German sentences, punctuation, numbers, Shift, AltGr, dead-key round trips, Unicode preservation and mixed text. Both Direct and Auto Fix produced `sghl → سلام`; correct `hello world` stayed unchanged.
* Browser UI checks passed localization-key parity, input undo/redo/counts, English/Persian switching, RTL at 600×500, theme switching and absence of JavaScript runtime errors. Screenshots: `test-results/`.
* Windows UI Automation verified the compiled desktop UI and real Tauri commands: three-step onboarding, Auto Fix, Direct convert, Swap, unchanged-input protection, ambiguous candidate selection, close-to-tray and reopening the existing instance with a second launch. No mocked native conversion was used.
* NSIS silent installation to an isolated workspace directory succeeded; the installed executable passed native UI checks. Start Menu shortcut and uninstall registration were verified. Silent uninstall succeeded and removed the installed executable.
* The release process was started normally in background mode; it did not create a WebView. See `benchmark-background.json` for the final 10-second sample, including native CPU, private bytes and working set of the full process tree. An earlier release sample measured 0% native CPU and 16.01 MiB working set with one process. These are measurements on this machine, not universal guarantees.

## Distribution limits

The binaries are unsigned; no current-user code-signing certificate was found. Publisher signing and broad compatibility testing remain release operations for public distribution. Windows 11 x64 was exercised here; Windows 10 and ARM64 were not separately tested. Word, PowerPoint and Chrome were tested in the cases described above. Windows Start Search could not be activated through the available automation and remains unverified. Messaging apps, secure/password fields, IMEs and elevated targets require application-specific checks. Universal application compatibility is not claimed.

Scores are conservative local heuristics built from small language profiles. Unknown vocabulary and ambiguous inputs require manual review. Some composed/custom dead-key sequences cannot be uniquely recovered from text. Clipboard restoration has a bounded 350 ms paste-read grace period; applications that defer reads longer may require manual paste.

The installer may download WebView2 if that system prerequisite is missing. Conversion, scoring, shortcuts and the installed UI do not depend on Internet access.

## Implementation references

* [Microsoft ToUnicodeEx documentation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-tounicodeex): modifier state, UTF-16 and non-mutating flag 4.
* [Microsoft clipboard sequence documentation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getclipboardsequencenumber).
* [Tauri Windows installer documentation](https://v2.tauri.app/distribute/windows-installer/): NSIS packaging and WebView2 deployment.
