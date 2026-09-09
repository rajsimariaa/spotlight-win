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
    let mut seen: HashMap<String, bool> = HashMap::new();

    // 1. Scan Start Menu
    let sm_dirs = vec![
        dirs::data_dir().map(|p| p.join(r"Microsoft\Windows\Start Menu\Programs")),
        Some(std::path::PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs")),
    ];
    for dir in sm_dirs.into_iter().flatten() {
        if dir.exists() {
            scan_shortcuts(&dir, &mut apps, &mut seen, 0);
        }
    }

    // 2. Scan all drives A-Z in parallel
    let mut handles = vec![];
    for letter in b'A'..=b'Z' {
        let drive = format!("{}:\\", letter as char);
        let drive_path = std::path::PathBuf::from(&drive);
        if !drive_path.exists() { continue; }
        let sm_count = apps.len();
        handles.push(std::thread::spawn(move || {
            let mut local_apps = Vec::new();
            let mut local_seen = HashMap::new();
            scan_dir(&drive_path, &mut local_apps, &mut local_seen, 0, 8);
            local_apps
        }));
    }

    for h in handles {
        if let Ok(local) = h.join() {
            for app in local {
                let key = app.name.to_lowercase();
                if !seen.contains_key(&key) {
                    seen.insert(key, true);
                    apps.push(app);
                }
            }
        }
    }

    // 3. Scan PATH env dirs
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(';') {
            let p = std::path::Path::new(dir.trim());
            if p.exists() && p.is_dir() {
                scan_dir(p, &mut apps, &mut seen, 0, 2);
            }
        }
    }

    apps
}

const SKIP_DIRS: &[&str] = &[
    "$recycle.bin", "system volume information", "recovery", "perflogs",
    "msocache", "onedrive", "windows.old",
];

fn scan_dir(
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
        let is_dir = path.is_dir();
        let name_lower = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        if name_lower.is_empty() || seen.contains_key(&name_lower) {
            continue;
        }

        if is_dir {
            if SKIP_DIRS.contains(&name_lower.as_str()) { continue; }
            scan_dir(&path, apps, seen, depth + 1, max_depth);
        } else if path.extension().map_or(false, |e| e == "exe" || e == "lnk") {
            seen.insert(name_lower, true);
            apps.push(AppEntry {
                name: path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string(),
                path: path.to_string_lossy().to_string(),
                app_type: AppType::Drive,
            });
        }
    }
}

fn scan_shortcuts(
    dir: &std::path::Path,
    apps: &mut Vec<AppEntry>,
    seen: &mut HashMap<String, bool>,
    depth: u32,
) {
    if depth > 6 { return; }
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
    let mut scored: Vec<(f64, &AppEntry)> = apps.iter()
        .filter_map(|app| {
            let name_lower = app.name.to_lowercase();
            let score = super::fuzzy::calculate_fuzzy_score(&query_lower, &name_lower);
            if score > 0.0 { Some((score, app)) } else { None }
        })
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(20).map(|(score, app)| {
        SearchResult {
            id: app.path.clone(),
            name: app.name.clone(),
            path: app.path.clone(),
            category: SearchResultCategory::Application,
            icon: None,
            score,
            metadata: None,
        }
    }).collect()
}
