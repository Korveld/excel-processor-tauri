use calamine::{Data, Reader, Xlsx};
use std::io::BufReader;
use rust_xlsxwriter::Workbook;

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
    let file = std::fs::File::open(&source_path).map_err(|e| e.to_string())?;
    let mut workbook = Xlsx::new(BufReader::new(file)).map_err(|e| e.to_string())?;

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| e.to_string())?;

    let rows: Vec<Vec<Data>> = range.rows().map(|r| r.to_vec()).collect();

    if rows.is_empty() {
        return Err("Sheet is empty".to_string());
    }

    let headers = &rows[0];
    let log_work_indices: Vec<usize> = headers
        .iter()
        .enumerate()
        .filter(|(_, cell)| matches!(cell, Data::String(s) if s.starts_with("Log Work")))
        .map(|(i, _)| i)
        .collect();

    let mut out_workbook = Workbook::new();
    let worksheet = out_workbook.add_worksheet();

    for (row_idx, row) in rows.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            let r = row_idx as u32;
            let c = col_idx as u16;

            if row_idx > 0 && log_work_indices.contains(&col_idx) {
                if let Data::String(s) = cell {
                    if let Some(num) = extract_last_number(&s, &search_str) {
                        worksheet.write(r, c, num as f64).map_err(|e| e.to_string())?;
                    }
                }
            } else {
                write_cell(worksheet, r, c, &cell).map_err(|e| e.to_string())?;
            }
        }
    }

    out_workbook
        .save(&output_path)
        .map_err(|e| e.to_string())?;

    Ok(format!("Processing complete! Saved to: {}", output_path))
}

fn write_cell(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    r: u32,
    c: u16,
    cell: &Data,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    match cell {
        Data::Int(n) => { worksheet.write(r, c, *n as f64)?; }
        Data::Float(f) => { worksheet.write(r, c, *f)?; }
        Data::String(s) => { worksheet.write(r, c, s.as_str())?; }
        Data::Bool(b) => { worksheet.write(r, c, *b)?; }
        Data::DateTimeIso(s) | Data::DurationIso(s) => { worksheet.write(r, c, s.as_str())?; }
        Data::DateTime(_) | Data::Error(_) | Data::Empty => {}
    }
    Ok(())
}

#[tauri::command]
async fn process_excel(
    source_path: String,
    output_path: String,
    sheet_name: String,
    search_str: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        process_excel_sync(source_path, output_path, sheet_name, search_str)
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