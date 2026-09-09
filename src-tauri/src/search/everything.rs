use super::{SearchResult, SearchResultCategory};

pub fn is_everything_available() -> bool {
    // Check if Everything is running
    std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq Everything.exe", "/NH"])
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.contains("Everything.exe")
        })
        .unwrap_or(false)
}

pub fn query_files(search_term: &str, max_results: u32) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let query_lower = search_term.to_lowercase();
    let drives = get_all_drive_letters();
    let count_limit = max_results as usize;

    for drive in &drives {
        let base = format!("{}\\", drive);
        scan_dir(&base, &query_lower, &mut results, count_limit, 0);
        if results.len() >= count_limit {
            break;
        }
    }

    results
}

fn scan_dir(base: &str, query: &str, results: &mut Vec<SearchResult>, limit: usize, depth: u32) {
    if depth > 2 || results.len() >= limit {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            if results.len() >= limit {
                return;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let name_lower = name.to_lowercase();
            let path = entry.path().to_string_lossy().to_string();

            if name_lower.contains(query) {
                let ext = entry.path().extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();
                results.push(SearchResult {
                    id: path.clone(),
                    name,
                    path,
                    category: SearchResultCategory::File,
                    icon: Some(ext),
                    score: 0.5,
                    metadata: None,
                });
            }

            if entry.path().is_dir() {
                scan_dir(&entry.path().to_string_lossy(), query, results, limit, depth + 1);
            }
        }
    }
}

fn get_all_drive_letters() -> Vec<String> {
    let mut drives = Vec::new();
    for i in 0..26 {
        let letter = (b'A' + i as u8) as char;
        let path = format!("{}:\\", letter);
        if std::path::Path::new(&path).exists() {
            drives.push(path);
        }
    }
    drives
}
