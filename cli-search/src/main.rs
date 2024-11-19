use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

fn main() {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: <program> <folder> <search> [exclude]");
        return;
    }

    let folder = &args[1];
    let search = &args[2];
    let default_excludes = "test,wpp,fitnesse".to_string();
    let exclude_str = args.get(3).unwrap_or(&default_excludes);
    let excludes: Vec<&str> = exclude_str.split(',').collect();

    let mut csv_output = Vec::new(); // Mutable outside the closure
    let mut total_lines = 0;         // Mutable outside the closure

    // Recursively search for .java files
    if let Err(err) = visit_dirs(Path::new(folder), &mut |file_path| {
        let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();

        // Exclude files with patterns in their names
        if excludes.iter().any(|&ex| file_name.contains(ex)) {
            return; // Skip this file
        }

        if file_path.extension().unwrap_or_default() == "java" {
            if let Ok(file) = fs::File::open(file_path) {
                let reader = io::BufReader::new(file);
                for (line_num, line) in reader.lines().enumerate() {
                    if let Ok(content) = line {
                        if content.contains(search) {
                            total_lines += 1;
                            csv_output.push(format!(
                                "{},{},{}",
                                file_path.display(),
                                line_num + 1,
                                content
                            ));
                        }
                    }
                }
            }
        }
    }) {
        eprintln!("Error while searching: {}", err);
    }

    // Print the total number of matching lines
    println!("Total matching lines: {}", total_lines);

    // Print CSV output
    println!("file,line,content");
    for entry in csv_output {
        println!("{}", entry);
    }
}

fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&Path)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&path);
            }
        }
    }
    Ok(())
}
