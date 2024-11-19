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
    let excludes: Vec<&str> = args
        .get(3)
        .unwrap_or(&"test,wpp,fitnesse".to_string())
        .split(',')
        .collect();

    // Prepare output CSV
    let mut csv_output = Vec::new();
    let mut total_lines = 0;

    // Recursively search for .java files
    if let Err(err) = visit_dirs(Path::new(folder), &|file_path| {
        if file_path.extension().unwrap_or_default() == "java" {
            if let Ok(file) = fs::File::open(file_path) {
                let reader = io::BufReader::new(file);
                for (line_num, line) in reader.lines().enumerate() {
                    if let Ok(content) = line {
                        if content.contains(search)
                            && !excludes.iter().any(|&ex| content.contains(ex))
                        {
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

fn visit_dirs(dir: &Path, cb: &dyn Fn(&Path)) -> io::Result<()> {
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
