# Excel Processor

A macOS desktop app for processing Excel log files. Built with Tauri v2, React, and Rust.

## What it does

1. Opens a source `.xlsx` file and reads a named sheet
2. Finds all columns whose header starts with **"Log Work"**
3. For each cell in those columns: if the cell contains the search string, extracts the integer after the last `;`
4. Saves the result to a new `.xlsx` file

Example — a cell value of `foo Jan/25; bar; 8` with search string `Jan/25` becomes `8`.

## Requirements

- [Rust](https://rustup.rs) (stable)
- [Bun](https://bun.sh)
- macOS (the bundle targets `.app` / `.dmg`)

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

Produces a signed `.app` and `.dmg` in:

```
src-tauri/target/release/bundle/macos/excel_processor_tauri.app
src-tauri/target/release/bundle/dmg/excel_processor_tauri_0.1.0_aarch64.dmg
```

## Project structure

```
src/
  App.tsx          # UI — form + Tauri invoke
  App.css          # Dark theme styles
src-tauri/
  src/
    lib.rs         # Rust backend — process_excel command
  capabilities/
    default.json   # Tauri permission grants
  tauri.conf.json  # App config (window, bundle, identifier)
  Cargo.toml       # Rust dependencies
```

## Key dependencies

| Dependency | Version | Role |
|------------|---------|------|
| `tauri` | 2 | App framework |
| `calamine` | 0.26 | Read `.xlsx` files |
| `rust_xlsxwriter` | 0.81 | Write `.xlsx` files |
| `tauri-plugin-dialog` | 2 | Native file open/save dialogs |
| `react` | 19 | UI |