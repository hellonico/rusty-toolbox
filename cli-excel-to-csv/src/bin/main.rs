use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use indicatif::ProgressBar;
use calamine::{open_workbook, DataType, Reader, Xlsx};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse the Excel file path from arguments
    let excel_path = std::env::args().nth(1).expect("Please provide an Excel file path.");
    let excel_path = Path::new(&excel_path);

    // Get the file name without extension and create a folder with that name
    let folder_name = excel_path.file_stem().unwrap().to_string_lossy().into_owned();
    fs::create_dir_all(&folder_name)?;

    // Open the Excel workbook
    let mut workbook: Xlsx<_> = open_workbook(excel_path)?;

    // Get the total sheet count for progress bar
    let total_sheets = workbook.sheet_names().len();
    let progress_bar = ProgressBar::new(total_sheets as u64);

    // Iterate over each sheet
    for sheet_name in workbook.sheet_names().to_owned() {
        if let Some(Ok(range)) = workbook.worksheet_range(&sheet_name) {
            // Create CSV file path in the folder
            let csv_file_path = Path::new(&folder_name).join(format!("{}.csv", sheet_name));
            let mut csv_file = File::create(csv_file_path)?;

            // Write worksheet data as CSV
            for row in range.rows() {
                let row_values: Vec<String> = row.iter()
                    .map(|cell| match cell {
                        DataType::String(s) => s.lines().collect::<Vec<&str>>().join("."), // Handle multi-line cell
                        DataType::Float(f) => f.to_string(),
                        DataType::Int(i) => i.to_string(),
                        DataType::Bool(b) => b.to_string(),
                        _ => String::new(),
                    })
                    .collect();
                writeln!(csv_file, "{}", row_values.join(","))?;
            }

            // Update progress bar
            progress_bar.inc(1);
        }
    }

    // Finish the progress bar
    progress_bar.finish_with_message("Export completed.");

    Ok(())
}
