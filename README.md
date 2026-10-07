<div align="center">

<img src="assets/readme/hero.gif" width="1200" height="480" alt="PIMX SWAP — animated 3D keyboard with layout switching arrows" />

**[🌐 English](README.md) · [🇮🇷 فارسی](README.fa.md)**

[**⬇️ Download for Windows**](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases/latest/download/PIMXSWAP-Setup.exe) · [Releases](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases)

</div>

# ⌨️ PIMX SWAP — one shortcut, the right layout

Finished a sentence and noticed your keyboard was on the wrong language? Select the text and press **Ctrl + Shift + Space**. PIMX SWAP maps the physical keys to the intended keyboard layout, locally on Windows, with no account or cloud service.

**`sghl` → `سلام`** · **`اثممخ` → `hello`**

Download the installer and start using it. No developer tools are required.

| At a glance | Details |
|:---|:---|
| 🪟 Platform | Windows 10 / 11 · x64 |
| 📦 Installer | [PIMXSWAP-Setup.exe](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases/latest/download/PIMXSWAP-Setup.exe) · 1.41 MiB |
| 🪶 Background RAM | About **13 MiB** in a local idle-tray measurement (12.85 MiB) |
| 🔒 Text processing | Local conversion and language detection |
| 🌐 Interface | English + Persian · themes · compact editor |

[⬇️ Install](#install) · [⚡ Use it](#use) · [🪶 RAM usage](#resources) · [🛠️ Development](#development)

<a id="install"></a>

## ⬇️ Download. Install. Use.

1. **[Download PIMXSWAP-Setup.exe](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases/latest/download/PIMXSWAP-Setup.exe)**, or open the [latest release](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases/latest).
2. Run the installer. It installs for the current Windows user and creates application shortcuts.
3. Open PIMXSWAP and choose your interface language, source layout and target layout.
4. Select wrong-layout text and press **Ctrl + Shift + Space**.

The interface requires **Microsoft Edge WebView2 Runtime**. If it is missing, setup downloads it from Microsoft; text conversion works offline after installation. The current installer is unsigned, so Windows may display a publisher confirmation.

| Download | Choose it for |
|:---|:---|
| [PIMXSWAP-Setup.exe](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases/latest/download/PIMXSWAP-Setup.exe) | Recommended: installation, Windows shortcuts and removal through Settings |
| [PIMXSWAP-Windows-x64.zip](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases/latest/download/PIMXSWAP-Windows-x64.zip) | Portable use: extract the folder and run PIMXSWAP.exe; WebView2 is still required |
| [SHA256SUMS.txt](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases/latest/download/SHA256SUMS.txt) | Verify that downloaded files match the published checksums |

Install a newer release manually to update; automatic updates are not configured. Uninstall preserves your preferences.

<a id="use"></a>

## ⚡ From wrong-layout text to readable text

| Default shortcut | Action |
|:---|:---|
| **Ctrl + Shift + Space** | Auto Fix selected text using local language evidence |
| **Ctrl + Alt + Shift + Space** | Convert using the configured source/target layout pair |
| **Ctrl + Alt + P** | Open the editor |
| **Ctrl + Enter** | Convert text inside the editor |

All shortcuts can be changed in Settings. Without a selection, the app tries the current line before the caret, then the previous word; select text to convert several lines. Ambiguous results can open a candidate picker.

Use the compact editor for quick conversions. Closing the window normally keeps the app in the tray; choose **Exit** in the tray menu to stop it completely. Shortcut pause and launch-at-Windows-startup are available in Settings.

## ✨ Details that make it useful

| Capability | What it gives you |
|:---|:---|
| 🧠 Auto Fix | Local language evidence and a picker for uncertain results |
| ⌨️ Native layouts | Read the Windows keyboard-layout catalog; choose an explicit layout pair |
| 📋 Clipboard handling | Copies and replaces text when you invoke a shortcut; restores supported clipboard contents when safe |
| 🎨 Editor | Themes, English/Persian UI, compact mode, input undo/redo and reduced motion |
| 💾 Preferences | Local layout, shortcut, theme and startup preferences |

<a id="resources"></a>

## 🪶 A small background footprint

In a local **1.0.0** Windows x64 measurement, the complete process-tree working set was **12.85 MiB** while **idle in the tray with the editor closed**: 10 samples over 11.37 seconds. Native CPU usage was zero in this idle sample.

Closing the editor destroys its WebView while the Rust core and global shortcuts remain available. Opening the editor adds WebView2 processes and uses more memory; the approximately 13 MiB figure describes the measured background state, not a guaranteed maximum for every mode.

📊 [Measurement method and results](docs/RESOURCE_USAGE.md) · [Raw samples](docs/RESOURCE_USAGE.json)

## 🔒 Your text stays on your device

Conversion and language detection run locally. The application has no account, analytics or cloud translation service and does not continuously record typing. Input text and clipboard snapshots are held temporarily in memory. Preferences file:

```text
%APPDATA%\com.pimxswap.desktop\settings.json
```

[Privacy details](docs/PRIVACY.md) · [Quick start](docs/QUICKSTART.md)

## 🧩 Compatibility and practical notes

This corrects keyboard layouts; it does not translate. Password fields, higher-privilege applications and some clipboard handlers can reject automatic replacement; use the editor and manual paste in those cases. IME composition and some custom keyboards have reconstruction limits. If a shortcut does nothing, release modifier keys and check the target application’s focus. Local detection profiles cover English, Persian, Arabic, Russian, German, French and Spanish; Direct convert lets you choose an explicit layout pair.

<a id="development"></a>

<details>
<summary>🛠️ Developers: source, builds and packaging</summary>

Source development requires Node.js, Rust MSVC 1.88 or newer, Visual Studio C++ Build Tools, Windows SDK and WebView2. The user installation path is at the top of this page.

```bash
git clone https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP.git
cd PIMX_SWAP
npm ci
npm run tauri dev
# Build an NSIS installer:
npm run release
```

| Path | Role |
|:---|:---|
| [`src/`](src/) | React / TypeScript interface |
| [`src-tauri/`](src-tauri/) | Rust engine, Windows integration and Tauri packaging |
| [`language_profiles/`](language_profiles/) | Local language evidence |
| [`scripts/`](scripts/) | Build, native checks, packaging and benchmarks |
| [`docs/`](docs/) | Usage, privacy and third-party notices |

[Detailed source guide](docs/PROJECT_GUIDE.md) · [Tauri Windows packaging](https://v2.tauri.app/distribute/windows-installer/)

</details>

## 🤝 Feedback

When opening an issue, include the app version, Windows version, layouts and reproduction steps. Use sample text.

[🐛 Issues](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/issues) · [⬇️ Releases](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases) · [PIMX](https://github.com/MOHAMMADREZAABEDINPOOR)
