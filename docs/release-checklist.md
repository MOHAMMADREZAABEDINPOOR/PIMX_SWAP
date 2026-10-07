# Release candidate 1.0.0 — 2026-10-01

## Completed on the owner's Windows x64 machine

- Packaged native executable and current-user NSIS installer, source archive and SHA-256 checksums.
- Earlier build: real Chrome textarea/contenteditable cases passed (4 cases). The final repeat was interrupted and did not pass all cases; `test-results/chrome-release.json` records the latest results, not the earlier successful run. The address-bar test was also corrected to retain the native window handle when its title changes.
- Earlier build: Word and PowerPoint each passed selected Persian, selected unfinished sentence and no-selection caret cases (3 per app). Later repeats exposed an intermittent clipboard-read failure and foreground changes. The copy path now waits for delayed text publication within its existing deadline; final native tests passed, but final Office verification remains pending a stable foreground session. `test-results/office-release.json` and `office-focus-interruption.json` record the interruptions.
- Isolated installer, same-version reinstall/update and uninstall passed; existing installation registration, shortcuts and settings were restored. Evidence: `test-results/installer-release.json`.
- UI screenshots and native clipboard/layout tests are documented in `validation.md`.
- Final build: 14 unit tests, 5 Windows-layout integration tests and 8 explicitly invoked native clipboard/hotkey tests passed. The native UI smoke test passed before the clipboard-only change. Test window activation now dispatches and settles activation messages before starting clipboard operations.
- npm production dependency audit and OSV scan of 262 locked Windows Rust dependencies reported no known advisories. This does not establish absence of vulnerabilities.
- Privacy information, quick start and locked-dependency license inventory prepared. Full available third-party license texts are provided alongside the binaries.

## Remaining release gates

- Repeat final Word/PowerPoint/Chrome tests without other apps taking foreground focus. The owner was asked to temporarily close other PIMXSWAP development instances and avoid mouse/keyboard input for approximately one minute. Current failures must not be represented as a clean final pass.
- Windows Start Search could not be activated by the available automation. Its shortcut behavior is unverified.
- No trusted code-signing certificate exists in the current-user certificate store. Obtain an appropriate publisher certificate, sign both executable and installer, then regenerate checksums. `scripts/sign-release.ps1` signs with an explicitly supplied certificate.
- Choose the application/source distribution license as the copyright owner. Dependency inventory generation does not choose or legally approve that license. All 266 listed packages have available license texts, including supplemental texts fetched from their recorded source commits or the license publisher.
- Test a clean Windows installation without an existing WebView2 runtime, plus other Windows versions/architectures you intend to support. Current-machine tests do not substitute for that coverage.
- Select a public download destination and publisher identity. No files have been uploaded or publicly published.

This is a tested release candidate, not a claim that the program has zero bugs or works in every application. Keep this report with the exact artifacts tested; repeat affected checks after changing them.
