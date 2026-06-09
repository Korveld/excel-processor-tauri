# Excel Processor

A cross-platform desktop app for processing Excel and CSV log files. Built with Tauri v2, React, and Rust. Runs on macOS, Windows, and Linux.

## What it does

1. Opens a source `.xlsx`, `.xls`, or `.csv` file and reads a named sheet
2. Finds all columns whose header starts with **"Log Work"**
3. For each cell in those columns: if the cell contains the search string, extracts the integer after the last `;`
4. Saves the result to a new `.xlsx` file

CSV files are converted to a single-sheet workbook automatically — no manual conversion needed.

Example — a cell value of `foo Jan/25; bar; 8` with search string `Jan/25` becomes `8`.

## Requirements

- [Rust](https://rustup.rs) (stable)
- [Bun](https://bun.sh)

## Setup

```bash
bun install
```

## Development

```bash
bun run desktop
```

Starts the Vite dev server and Tauri in dev mode. The frontend hot-reloads; Rust recompiles on file save.

## Build

```bash
bun tauri build
```

Produces platform-native bundles in `src-tauri/target/release/bundle/`:

| Platform | Output |
|----------|--------|
| macOS | `.app` + `.dmg` |
| Windows | `.exe` installer + `.msi` |
| Linux | `.AppImage` + `.deb` + `.rpm` |

## Releases

Pre-built binaries for all platforms are available on the [Releases](../../releases) page.

## Project structure

```
src/
  App.tsx               # UI — form + Tauri invoke
  App.css               # Dark theme styles
  assets/app_icon.png   # Source app icon (1024x1024)
src-tauri/
  src/
    lib.rs              # Rust backend — process_excel command
  icons/                # Generated icon files (all sizes/formats)
  capabilities/
    default.json        # Tauri permission grants
  tauri.conf.json       # App config (window, bundle, identifier)
  Cargo.toml            # Rust dependencies
.github/workflows/
  build.yml             # CI — builds all platforms, releases on tag push
```

## Key dependencies

| Dependency | Version | Role |
|------------|---------|------|
| `tauri` | 2 | App framework |
| `umya-spreadsheet` | 3 | Read and write `.xlsx` files |
| `csv` | 1 | Parse `.csv` input files |
| `tauri-plugin-dialog` | 2 | Native file open/save dialogs |
| `react` | 19 | UI |