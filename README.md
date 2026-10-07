<div align="center">

<img src="assets/readme/hero.gif" width="1200" alt="PIMX SWAP — rotating 3D geometry" />

**[English](README.md) · [فارسی](README.fa.md)**

<img src="assets/readme/identity.svg" width="1200" alt="web / English and Persian documentation" />

</div>

# PIMX SWAP

A local Windows keyboard-layout correction utility built with Rust, Tauri 2, React and TypeScript. It reconstructs physical key input and maps it to another layout: `sghl` → `سلام`.

[GitHub](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP) · [PIMX / Profile](https://github.com/MOHAMMADREZAABEDINPOOR) · [Static artwork](assets/readme/hero.png)

## Features

- Auto Fix with local language evidence and a candidate picker
- Global shortcuts, native editor and tray lifecycle
- Clipboard transactions with restoration checks
- English/Persian UI, themes and local settings

## Stack

| Tool | Version / source |
|---|---|
| React | `^19.1.0` |
| Vite | `^7.1.0` |
| TypeScript | `^5.9.0` |
| Rust | `src-tauri/Cargo.toml` |
| Tauri | `2.x` |

## Getting started

Windows 10/11 x64, WebView2, Node.js 24, stable Rust (MSVC) and Visual Studio C++ Build Tools with Windows SDK.

```bash
git clone https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP.git
cd PIMX_SWAP

npm ci
npm run icons
npm run tauri dev
```

## Configuration

No standard environment template is defined. Standalone exercises need no external configuration; inspect any service constants or paths in the source before running.

## Usage

On Windows, use npm run tauri dev for native integration. Select text and press Ctrl+Shift+Space for Auto Fix; Ctrl+Alt+Shift+Space converts the configured pair and Ctrl+Alt+P opens the editor. Shortcuts are configurable.

## Project structure

| Path | Role |
|---|---|
| [`assets/`](assets/) | Brand/media/README assets |
| [`docs/`](docs/) | Supporting documentation |
| [`language_profiles/`](language_profiles/) | Local language evidence |
| [`public/`](public/) | Public web assets |
| [`scripts/`](scripts/) | Development and maintenance utilities |
| [`src/`](src/) | Application source |
| [`src-tauri/`](src-tauri/) | Rust native desktop core |
| [`index.html`](index.html) | Project entry/configuration file |
| [`package.json`](package.json) | Project entry/configuration file |
| [`tsconfig.json`](tsconfig.json) | Project entry/configuration file |

## Commands and checks

```bash
npm run dev
npm run build
npm run preview
npm run test:ui
npm run icons
npm run test:workspace
```

These commands are declared in package.json; the list is not a test execution report. Test commands may need a browser, service or prepared database.

## Deployment

scripts/release.ps1 checks/builds the frontend and native core and produces an NSIS installer. See docs/ for release details. Build/release outputs are excluded from the source repository.

## Detailed project guide

[Extended project guide](docs/PROJECT_GUIDE.md)

## Limitations

Windows 10/11, WebView2 and native build tools are required. IMEs and arbitrary custom dead-key sequences are not fully reconstructible. Protected/elevated apps may reject replacement; use the editor. Browser preview lacks native integration.

## Troubleshooting

- A reserved shortcut: choose another combination in Settings.
- Missing layout: add it in Windows Settings and refresh the catalog.
- Native features unavailable in a browser: run tauri dev or the compiled executable.

## Contributing

Create a focused branch, verify the affected behavior and explain the change clearly. Keep private data, build outputs and local databases out of commits.

Supporting guides:

- [docs/architecture.md](docs/architecture.md)
- [docs/development.md](docs/development.md)
- [docs/PRIVACY.md](docs/PRIVACY.md)
- [docs/validation.md](docs/validation.md)
- [docs/QUICKSTART.md](docs/QUICKSTART.md)
- [docs/release-checklist.md](docs/release-checklist.md)

## License

No repository-level license file is included in this snapshot. Public visibility alone does not grant reuse rights; contact the repository owner for terms.

---

Part of **PIMX** · Documentation in English and Persian.
