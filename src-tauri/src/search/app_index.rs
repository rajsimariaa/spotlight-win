use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::{SearchResult, SearchResultCategory};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub name: String,
    pub path: String,
    pub app_type: AppType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppType {
    StartMenu,
    Drive,
}

pub fn build_app_cache() -> Vec<AppEntry> {
    let mut apps = Vec::new();
    let mut seen = HashMap::new();

    // Scan Start Menu
    let start_menu_dirs = vec![
        dirs::data_dir().map(|p| p.join(r"Microsoft\Windows\Start Menu\Programs")),
        Some(std::path::PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs")),
    ];
    for dir in start_menu_dirs.into_iter().flatten() {
        if dir.exists() {
            scan_shortcuts(&dir, &mut apps, &mut seen, 0);
        }
    }

    // Scan ALL drives A-Z with deep recursion
    for letter in b'A'..=b'Z' {
        let drive = format!("{}:\\", letter as char);
        let drive_path = std::path::Path::new(&drive);
        if drive_path.exists() {
            scan_recursive(drive_path, &mut apps, &mut seen, 0, 10);
        }
    }

    apps
}

fn scan_recursive(
    dir: &std::path::Path,
    apps: &mut Vec<AppEntry>,
    seen: &mut HashMap<String, bool>,
    depth: u32,
    max_depth: u32,
) {
    if depth > max_depth { return; }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name_lower = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_lowercase(),
            None => continue,
        };
        if name_lower.is_empty() || seen.contains_key(&name_lower) { continue; }

        if path.is_dir() {
            // Only skip truly irrelevant dirs
            let skip = ["$recycle.bin", "system volume information", "recovery", "perflogs"];
            if skip.contains(&name_lower.as_str()) { continue; }
            scan_recursive(&path, apps, seen, depth + 1, max_depth);
        } else if path.extension().map_or(false, |e| e == "exe") {
            seen.insert(name_lower, true);
            apps.push(AppEntry {
                name: path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string(),
                path: path.to_string_lossy().to_string(),
                app_type: AppType::Drive,
            });
        }
    }
}

fn scan_shortcuts(dir: &std::path::Path, apps: &mut Vec<AppEntry>, seen: &mut HashMap<String, bool>, depth: u32) {
    if depth > 5 { return; }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_shortcuts(&path, apps, seen, depth + 1);
        } else if path.extension().map_or(false, |e| e == "lnk") {
            let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
            let key = name.to_lowercase();
            if !name.is_empty() && !seen.contains_key(&key) {
                seen.insert(key, true);
                apps.push(AppEntry {
                    name,
                    path: path.to_string_lossy().to_string(),
                    app_type: AppType::StartMenu,
                });
            }
        }
    }
}

pub fn search_applications(query: &str, apps: &[AppEntry]) -> Vec<SearchResult> {
    let query_lower = query.to_lowercase();
    apps.iter().filter_map(|app| {
        let name_lower = app.name.to_lowercase();
        let score = super::fuzzy::calculate_fuzzy_score(&query_lower, &name_lower);
        if score > 0.0 {
            Some(SearchResult {
                id: app.path.clone(),
                name: app.name.clone(),
                path: app.path.clone(),
                category: SearchResultCategory::Application,
                icon: None,
                score,
                metadata: None,
            })
        } else {
            None
        }
    }).collect()
}
