# Excel Processor — Claude Code Context

## Project overview

A Tauri v2 desktop app (React + TypeScript frontend, Rust backend) that processes Excel and CSV files. It reads a named sheet (or the single "Sheet1" for CSV input), finds all columns whose header starts with "Log Work", and for each cell in those columns extracts the integer after the last semicolon (if the cell string contains the user-supplied search string). The result is written to a new `.xlsx` file.

## Stack

- **Frontend**: React 19, TypeScript, Vite — lives in `src/`
- **Backend**: Rust (Tauri v2) — lives in `src-tauri/src/lib.rs`
- **Package manager**: bun
- **Excel reading + writing**: `umya-spreadsheet 3`
- **CSV parsing**: `csv 1`
- **File dialogs**: `tauri-plugin-dialog 2`

## Commands

```bash
bun run desktop      # dev mode (hot-reload frontend + Rust recompile on change)
bun tauri build      # release build → .app + .dmg in src-tauri/target/release/bundle/
bun tauri icon <file>                              # regenerate all icon sizes from a source image
cargo build --manifest-path src-tauri/Cargo.toml  # check Rust only
```

## Key files

| File | Purpose |
|------|---------|
| `src/App.tsx` | Entire UI — 4-field form + invoke call |
| `src/App.css` | Dark minimalistic theme |
| `src-tauri/src/lib.rs` | Rust: `process_excel`, `get_sheet_names` commands + Excel logic |
| `src-tauri/tauri.conf.json` | Window config, bundle identifier, product name, version |
| `src-tauri/capabilities/default.json` | Tauri permission grants (dialog open/save) |
| `src-tauri/icons/` | All icon sizes — generated via `bun tauri icon`, do not edit manually |
| `.github/workflows/build.yml` | CI: builds all platforms, publishes GitHub Release on tag push |
| `hints.md` | Local-only cheatsheet (gitignored) |

## CI / releases

- **Push to `main`** → builds all 4 platforms, no artifacts kept, no release created
- **Push a `v*` tag** → same build + creates a GitHub Release with all installers attached
- Platforms: macOS ARM, macOS Intel (cross-compiled from ARM runner), Windows, Linux
- To release: `git tag v1.x.x && git push --tags`

## Icons

Source image lives at `src/assets/app_icon.png`. To regenerate all icon formats after updating it:
```bash
bun tauri icon src/assets/app_icon.png
```
This overwrites everything in `src-tauri/icons/` including `.icns`, `.ico`, all PNG sizes, and Android/iOS variants.

## Architecture notes

- Both Tauri commands are `async` and use `tauri::async_runtime::spawn_blocking` to run blocking file I/O on a thread pool, keeping the macOS event loop free (avoids beachball).
- `flushSync` is used in the React handler before `invoke` to guarantee the spinner renders before the Rust work begins (React 18 batches state updates and would otherwise defer the repaint until after the async call).
- File dialogs are handled entirely on the JS side via `@tauri-apps/plugin-dialog` — no Rust command needed for open/save pickers.
- When a source file is picked, `browseSource` calls `get_sheet_names` to populate a `<select>` with the available sheets (auto-selects if only one), and auto-fills the output path as `<source>_output.xlsx`. This works uniformly for both `.xlsx`/`.xls` and `.csv` inputs — the output is always `.xlsx`.
- CSV input is supported: `get_sheet_names` returns `["Sheet1"]` for `.csv` files. `process_excel_sync` detects the `.csv` extension, calls `csv_to_workbook` to parse the file into an in-memory `Workbook` with a single "Sheet1", then proceeds with the same Log Work column logic as for native Excel files.
- `std::panic::catch_unwind` wraps the sync processing in `process_excel` so any unexpected library panics are caught and returned as user-visible error messages rather than silently crashing.
- Excel processing modifies the source workbook in-place (read → mutate Log Work columns → write to output path) rather than building a new workbook from scratch. This preserves all cell styles including date number formats on columns like Created/Updated/Last Viewed.
- Excel I/O uses `umya-spreadsheet` (replaces the original `calamine` + `rust_xlsxwriter` pair). `calamine` had a persistent panic on multi-byte Unicode characters (e.g. `→`) in cell values — `umya-spreadsheet` handles these correctly.
- DevTools can be re-enabled for debugging by adding `.setup(|app| { app.get_webview_window("main").unwrap().open_devtools(); Ok(()) })` to the Tauri builder in `lib.rs` (guarded with `#[cfg(debug_assertions)]`).