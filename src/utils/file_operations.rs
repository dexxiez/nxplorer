use std::fs;
use std::path::Path;

pub fn find_files(dir: &Path, target_files: &[&str]) -> Vec<String> {
    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            // Check if it's a file and matches any of our target filenames
            if path.is_file() {
                if let Some(filename) = path.file_name() {
                    if let Some(filename_str) = filename.to_str() {
                        if target_files.contains(&filename_str) {
                            if let Some(path_str) = path.to_str() {
                                results.push(path_str.to_string());
                            }
                        }
                    }
                }
            }
            // Recursively search directories
            else if path.is_dir() {
                if path.file_name() == Some("node_modules".as_ref()) {
                    continue;
                }
                results.extend(find_files(&path, target_files));
            }
        }
    }

    results
}

pub fn path_exists(file: &Path) -> bool {
    fs::metadata(&file).is_ok()
}
