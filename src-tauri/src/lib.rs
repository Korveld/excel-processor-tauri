use umya_spreadsheet::reader::xlsx as xlsx_reader;
use umya_spreadsheet::writer::xlsx as xlsx_writer;

fn extract_last_number(value: &str, search_str: &str) -> Option<i64> {
    if value.contains(search_str) {
        let last_part = value.split(';').last()?.trim();
        last_part.parse::<i64>().ok()
    } else {
        None
    }
}

fn process_excel_sync(
    source_path: String,
    output_path: String,
    sheet_name: String,
    search_str: String,
) -> Result<String, String> {
    let book = xlsx_reader::read(std::path::Path::new(&source_path))
        .map_err(|e| format!("Failed to open file: {:?}", e))?;

    let sheet = book
        .sheet_by_name(&sheet_name)
        .map_err(|_| format!("Sheet '{}' not found", sheet_name))?;

    let max_row = sheet.highest_row();
    let max_col = sheet.highest_column();

    if max_row == 0 {
        return Err("Sheet is empty".to_string());
    }

    // Collect all cell values
    let mut data: Vec<Vec<String>> = (1..=max_row)
        .map(|row| {
            (1..=max_col)
                .map(|col| {
                    sheet
                        .cell((col, row))
                        .map(|c| c.value().to_string())
                        .unwrap_or_default()
                })
                .collect()
        })
        .collect();

    // Find "Log Work" column indices (0-based)
    let log_work_indices: Vec<usize> = data[0]
        .iter()
        .enumerate()
        .filter(|(_, h)| h.starts_with("Log Work"))
        .map(|(i, _)| i)
        .collect();

    // Transform Log Work cells in data rows
    for row in data.iter_mut().skip(1) {
        for &col_idx in &log_work_indices {
            if let Some(cell) = row.get_mut(col_idx) {
                *cell = extract_last_number(cell, &search_str)
                    .map(|n| n.to_string())
                    .unwrap_or_default();
            }
        }
    }

    // Write output
    let mut out_book = umya_spreadsheet::new_file();
    let out_sheet = out_book
        .sheet_mut(0_usize)
        .map_err(|e| format!("{:?}", e))?;

    for (row_idx, row) in data.iter().enumerate() {
        for (col_idx, value) in row.iter().enumerate() {
            if !value.is_empty() {
                let r = (row_idx + 1) as u32;
                let c = (col_idx + 1) as u32;
                out_sheet.cell_mut((c, r)).set_value(value.clone());
            }
        }
    }

    xlsx_writer::write(&out_book, std::path::Path::new(&output_path))
        .map_err(|e| format!("Failed to save: {:?}", e))?;

    Ok(format!("Processing complete! Saved to: {}", output_path))
}

#[tauri::command]
async fn process_excel(
    source_path: String,
    output_path: String,
    sheet_name: String,
    search_str: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            process_excel_sync(source_path, output_path, sheet_name, search_str)
        }))
        .unwrap_or_else(|e| {
            let msg = e
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "unexpected error while processing file".to_string());
            Err(msg)
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![process_excel])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}