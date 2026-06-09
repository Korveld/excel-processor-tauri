use umya_spreadsheet::reader::xlsx as xlsx_reader;
use umya_spreadsheet::writer::xlsx as xlsx_writer;

fn csv_to_workbook(source_path: &str) -> Result<umya_spreadsheet::Workbook, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(source_path)
        .map_err(|e| format!("Failed to read CSV: {}", e))?;

    let mut records: Vec<Vec<String>> = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| format!("CSV parse error: {}", e))?;
        records.push(record.iter().map(|f| f.to_string()).collect());
    }

    let mut book = umya_spreadsheet::new_file();
    let sheet = book
        .sheet_by_name_mut("Sheet1")
        .map_err(|_| "Failed to get default sheet".to_string())?;

    for (row_idx, record) in records.iter().enumerate() {
        for (col_idx, field) in record.iter().enumerate() {
            sheet
                .cell_mut(((col_idx + 1) as u32, (row_idx + 1) as u32))
                .set_value_string(field);
        }
    }

    Ok(book)
}

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
    let mut book = if source_path.to_lowercase().ends_with(".csv") {
        csv_to_workbook(&source_path)?
    } else {
        xlsx_reader::read(std::path::Path::new(&source_path))
            .map_err(|e| format!("Failed to open file: {:?}", e))?
    };

    // First pass (immutable): find Log Work column indices from the header row
    let log_work_cols: Vec<u32> = {
        let sheet = book
            .sheet_by_name(&sheet_name)
            .map_err(|_| format!("Sheet '{}' not found", sheet_name))?;

        if sheet.highest_row() == 0 {
            return Err("Sheet is empty".to_string());
        }

        let max_col = sheet.highest_column();
        (1..=max_col)
            .filter(|&col| {
                sheet
                    .cell((col, 1))
                    .map(|c| c.value().starts_with("Log Work"))
                    .unwrap_or(false)
            })
            .collect()
    };

    // Second pass (mutable): transform Log Work cells in-place, leaving all other
    // cells (including date-formatted ones) untouched so their styles are preserved
    let sheet = book
        .sheet_by_name_mut(&sheet_name)
        .map_err(|_| format!("Sheet '{}' not found", sheet_name))?;

    let max_row = sheet.highest_row();

    for row in 2..=max_row {
        for &col in &log_work_cols {
            let current = match sheet.cell((col, row)) {
                Some(c) => c.value().to_string(),
                None => continue,
            };
            if current.is_empty() {
                continue;
            }
            if let Some(n) = extract_last_number(&current, &search_str) {
                sheet.cell_mut((col, row)).set_value_number(n as f64);
            } else {
                sheet.cell_mut((col, row)).set_value_string("");
            }
        }
    }

    xlsx_writer::write(&book, std::path::Path::new(&output_path))
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

#[tauri::command]
async fn get_sheet_names(source_path: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if source_path.to_lowercase().ends_with(".csv") {
            return Ok(vec!["Sheet1".to_string()]);
        }
        let book = xlsx_reader::read(std::path::Path::new(&source_path))
            .map_err(|e| format!("Failed to open file: {:?}", e))?;
        let names: Vec<String> = book
            .sheet_collection()
            .iter()
            .map(|s| s.name().to_string())
            .collect();
        Ok(names)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![process_excel, get_sheet_names])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}