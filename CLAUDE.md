# Excel Processor — Claude Code Context

## Project overview

A Tauri v2 desktop app (React + TypeScript frontend, Rust backend) that processes Excel files. It reads a named sheet, finds all columns whose header starts with "Log Work", and for each cell in those columns extracts the integer after the last semicolon (if the cell string contains the user-supplied search string). The result is written to a new `.xlsx` file.

## Stack

- **Frontend**: React 19, TypeScript, Vite — lives in `src/`
- **Backend**: Rust (Tauri v2) — lives in `src-tauri/src/lib.rs`
- **Package manager**: bun
- **Excel reading**: `calamine 0.26`
- **Excel writing**: `rust_xlsxwriter 0.81`
- **File dialogs**: `tauri-plugin-dialog 2`

## Commands

```bash
bun run desktop      # dev mode (hot-reload frontend + Rust recompile on change)
bun tauri build      # release build → .app + .dmg in src-tauri/target/release/bundle/
cargo build --manifest-path src-tauri/Cargo.toml   # check Rust only
```

## Key files

| File | Purpose |
|------|---------|
| `src/App.tsx` | Entire UI — 4-field form + invoke call |
| `src/App.css` | Dark minimalistic theme |
| `src-tauri/src/lib.rs` | Rust: `process_excel` command + Excel logic |
| `src-tauri/tauri.conf.json` | Window config, bundle identifier, permissions |
| `src-tauri/capabilities/default.json` | Tauri permission grants (dialog open/save) |

## Architecture notes

- The `process_excel` Tauri command is `async` and uses `tauri::async_runtime::spawn_blocking` to run the blocking file I/O on a thread pool, keeping the macOS event loop free (avoids beachball).
- `flushSync` is used in the React handler before `invoke` to guarantee the spinner renders before the Rust work begins (React 18 batches state updates and would otherwise defer the repaint until after the async call).
- File dialogs are handled entirely on the JS side via `@tauri-apps/plugin-dialog` — no Rust command needed for open/save pickers.
- DevTools can be re-enabled for debugging by adding `.setup(|app| { app.get_webview_window("main").unwrap().open_devtools(); Ok(()) })` to the Tauri builder in `lib.rs` (guarded with `#[cfg(debug_assertions)]`).