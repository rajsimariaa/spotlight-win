use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};

use super::{SearchResult, SearchResultCategory};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub name: String,
    pub path: String,
    pub icon_path: Option<String>,
    pub app_type: AppType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppType {
    StartMenuShortcut,
    UwpApp,
    SystemBinary,
}

static APP_CACHE: OnceLock<Vec<AppEntry>> = OnceLock::new();

pub fn get_all_applications() -> Vec<AppEntry> {
    if let Some(cache) = APP_CACHE.get() {
        return cache.clone();
    }

    let mut apps = Vec::new();

    // Scan Start Menu shortcuts
    apps.extend(scan_start_menu_shortcuts());

    // Scan UWP apps
    apps.extend(scan_uwp_apps());

    // Scan PATH binaries
    apps.extend(scan_path_binaries());

    let _ = APP_CACHE.set(apps.clone());
    apps
}

fn scan_start_menu_shortcuts() -> Vec<AppEntry> {
    let mut apps = Vec::new();

    let start_menu_paths = vec![
        dirs::data_dir()
            .map(|p| p.join("Microsoft\\Windows\\Start Menu\\Programs")),
        Some(std::path::PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs")),
    ];

    for base_path in start_menu_paths.into_iter().flatten() {
        if base_path.exists() {
            scan_directory_for_shortcuts(&base_path, &mut apps);
        }
    }

    apps
}

fn scan_directory_for_shortcuts(dir: &PathBuf, apps: &mut Vec<AppEntry>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_directory_for_shortcuts(&path, apps);
            } else if path.extension().map_or(false, |e| e == "lnk") {
                let name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let target = resolve_lnk_target(&path).unwrap_or_else(|| path.to_string_lossy().to_string());

                apps.push(AppEntry {
                    name,
                    path: target,
                    icon_path: None,
                    app_type: AppType::StartMenuShortcut,
                });
            }
        }
    }
}

fn resolve_lnk_target(lnk_path: &PathBuf) -> Option<String> {
    // Return the .lnk path itself - Tauri shell open handles shortcuts natively
    Some(lnk_path.to_string_lossy().to_string())
}

fn scan_uwp_apps() -> Vec<AppEntry> {
    let mut apps = Vec::new();

    #[cfg(target_os = "windows")]
    {
        // Use PowerShell to enumerate UWP apps from AppsFolder
        if let Ok(output) = std::process::Command::new("powershell")
            .args([
                "-NoProfile", "-Command",
                "Get-StartApps | Select-Object Name, AppID | ConvertTo-Json"
            ])
            .output()
        {
            if let Ok(json_str) = String::from_utf8(output.stdout) {
                if let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(&json_str) {
                    for item in items {
                        let name = item["Name"].as_str().unwrap_or("").to_string();
                        let app_id = item["AppID"].as_str().unwrap_or("").to_string();
                        if !name.is_empty() && !app_id.is_empty() {
                            apps.push(AppEntry {
                                name,
                                path: app_id,
                                icon_path: None,
                                app_type: AppType::UwpApp,
                            });
                        }
                    }
                }
            }
        }
    }

    apps
}

fn scan_path_binaries() -> Vec<AppEntry> {
    let mut apps = Vec::new();
    let mut seen = HashMap::new();

    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(';') {
            let path = PathBuf::from(dir);
            if path.exists() {
                if let Ok(entries) = std::fs::read_dir(&path) {
                    for entry in entries.flatten() {
                        let entry_path = entry.path();
                        if entry_path.extension().map_or(false, |e| e == "exe") {
                            let name = entry_path.file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("Unknown")
                                .to_string();

                            if !seen.contains_key(&name) {
                                seen.insert(name.clone(), true);
                                apps.push(AppEntry {
                                    name,
                                    path: entry_path.to_string_lossy().to_string(),
                                    icon_path: None,
                                    app_type: AppType::SystemBinary,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    apps
}

pub fn search_applications(query: &str, apps: &[AppEntry]) -> Vec<SearchResult> {
    let query_lower = query.to_lowercase();

    apps.iter()
        .filter_map(|app| {
            let name_lower = app.name.to_lowercase();
            let score = super::fuzzy::calculate_fuzzy_score(&query_lower, &name_lower);

            if score > 0.0 {
                Some(SearchResult {
                    id: app.path.clone(),
                    name: app.name.clone(),
                    path: app.path.clone(),
                    category: SearchResultCategory::Application,
                    icon: app.icon_path.clone(),
                    score,
                    metadata: Some(format!("{:?}", app.app_type)),
                })
            } else {
                None
            }
        })
        .collect()
}
