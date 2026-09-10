pub mod app_index;
pub mod fuzzy;
pub mod evaluator;
pub mod quick_actions;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub name: String,
    pub path: String,
    pub category: SearchResultCategory,
    pub icon: Option<String>,
    pub score: f64,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SearchResultCategory {
    Application,
    File,
    Calculator,
    Action,
    Conversion,
    Timezone,
    WebSearch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub query_time_ms: u64,
    pub total_results: usize,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
}

static CACHED_APPS: OnceLock<Vec<app_index::AppEntry>> = OnceLock::new();
static CACHED_FILES: OnceLock<Vec<FileEntry>> = OnceLock::new();

const FILE_EXTS: &[&str] = &[
    // Documents
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "rtf", "odt", "csv", "md",
    "epub", "mobi", "wps", "dif", "ods", "odp", "pages", "numbers", "key",
    // Images
    "jpg", "jpeg", "png", "gif", "bmp", "svg", "webp", "ico", "tiff", "raw",
    "heic", "heif", "psd", "ai", "eps", "cr2", "nef", "arw", "dng",
    // Audio
    "mp3", "wav", "flac", "aac", "ogg", "wma", "m4a", "opus", "aiff", "mid", "midi",
    // Video
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "ts", "vob", "3gp",
    // Code
    "js", "ts", "tsx", "jsx", "py", "rs", "go", "java", "c", "cpp", "h", "cs",
    "html", "css", "json", "xml", "yaml", "yml", "toml", "rb", "php", "swift",
    "kt", "scala", "r", "lua", "pl", "sh", "bash", "ps1", "bat", "cmd", "vbs",
    "vue", "svelte", "dart", "zig", "nim", "ex", "exs", "erl", "hs", "ml",
    "sql", "graphql", "proto", "cmake", "makefile", "dockerfile",
    // Config / build
    "ini", "cfg", "conf", "env", "gitignore", "dockerignore", "editorconfig",
    "prettierrc", "eslintrc", "babelrc", "tsconfig",
    // Archives
    "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "zst", "lz4",
    // Other
    "iso", "img", "dll", "log", "backup", "bak", "old", "tmp",
    "pptx", "docm", "xlsm", "dotx",
];

pub fn init_cache() {
    let apps = app_index::build_app_cache();
    let _ = CACHED_APPS.set(apps);

    let files = build_file_cache();
    let _ = CACHED_FILES.set(files);
}

fn build_file_cache() -> Vec<FileEntry> {
    let mut files = Vec::new();
    let mut seen = HashMap::new();

    // 1. User profile directories — deeper scan (depth 7)
    if let Some(home) = dirs::home_dir() {
        let user_dirs = ["Documents", "Desktop", "Downloads", "Pictures", "Videos", "Music"];
        for d in user_dirs {
            let dir = home.join(d);
            if dir.exists() {
                scan_files(&dir, &mut files, &mut seen, 0, 7);
            }
        }
    }

    // 2. Quick access locations
    let quick_dirs = [
        dirs::desktop_dir(),
        dirs::document_dir(),
        dirs::download_dir(),
    ];
    for dir in quick_dirs.into_iter().flatten() {
        if dir.exists() {
            scan_files(&dir, &mut files, &mut seen, 0, 6);
        }
    }

    // 3. All drives — root + 3 levels
    for letter in b'A'..=b'Z' {
        let drive = format!("{}:\\", letter as char);
        let drive_path = std::path::Path::new(&drive);
        if drive_path.exists() {
            scan_files(drive_path, &mut files, &mut seen, 0, 3);
        }
    }

    files
}

fn is_file_ext(ext: &str) -> bool {
    FILE_EXTS.contains(&ext)
}

const FILE_SKIP_DIRS: &[&str] = &[
    "$recycle.bin", "system volume information", "recovery", "perflogs",
    "msocache", "appdata", ".git", "node_modules", "__pycache__",
    ".vscode", "target", "dist", "build", ".cache", ".npm",
    "windows", "program files", "program files (x86)", "programdata",
    ".local", ".config", ".ssh", ".vscode-insiders", "snap",
    "android", ".android", "intel", "nvidia", "amd",
];

fn scan_files(
    dir: &std::path::Path,
    files: &mut Vec<FileEntry>,
    seen: &mut HashMap<String, bool>,
    depth: u32,
    max_depth: u32,
) {
    if depth > max_depth { return; }

    // Speed limit: stop scanning large dirs
    if files.len() > 100_000 { return; }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name_lower = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            if !FILE_SKIP_DIRS.contains(&name_lower.as_str()) && !name_lower.starts_with('.') {
                scan_files(&path, files, seen, depth + 1, max_depth);
            }
        } else {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if is_file_ext(ext) {
                let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                let key = path.to_string_lossy().to_lowercase();
                if !name.is_empty() && !seen.contains_key(&key) {
                    seen.insert(key, true);
                    files.push(FileEntry {
                        name,
                        path: path.to_string_lossy().to_string(),
                    });
                }
            }
        }
    }
}

pub fn search_files(query: &str, files: &[FileEntry]) -> Vec<SearchResult> {
    let query_lower = query.to_lowercase();
    let mut scored: Vec<(f64, &FileEntry)> = files.iter()
        .filter_map(|f| {
            let name_lower = f.name.to_lowercase();
            // Score against filename
            let name_score = fuzzy::calculate_fuzzy_score(&query_lower, &name_lower);

            // Score against full path (lower priority)
            let path_lower = f.path.to_lowercase();
            let path_score = fuzzy::calculate_fuzzy_score(&query_lower, &path_lower) * 0.6;

            let score = name_score.max(path_score);
            if score > 0.0 { Some((score, f)) } else { None }
        })
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(15).map(|(score, f)| {
        let ext = std::path::Path::new(&f.path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        SearchResult {
            id: f.path.clone(),
            name: f.name.clone(),
            path: f.path.clone(),
            category: SearchResultCategory::File,
            icon: None,
            score,
            metadata: Some(format!("file:{}", ext)),
        }
    }).collect()
}

pub fn search(query: &str) -> SearchResponse {
    let start = std::time::Instant::now();
    let mut results: Vec<SearchResult> = Vec::new();

    if query.trim().is_empty() {
        return SearchResponse { results, query_time_ms: 0, total_results: 0 };
    }

    // Apps
    if let Some(apps) = CACHED_APPS.get() {
        results.extend(app_index::search_applications(query, apps));
    }

    // Files
    if let Some(files) = CACHED_FILES.get() {
        results.extend(search_files(query, files));
    }

    // Calculator
    if let Some(r) = evaluator::evaluate_expression(query) {
        results.push(r);
    }

    // Quick actions
    results.extend(quick_actions::search_actions(query));

    // Sort by score, but keep actions near top
    results.sort_by(|a, b| {
        if a.category == SearchResultCategory::Action && b.category != SearchResultCategory::Action {
            return std::cmp::Ordering::Less;
        }
        if b.category == SearchResultCategory::Action && a.category != SearchResultCategory::Action {
            return std::cmp::Ordering::Greater;
        }
        b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
    });

    // NO web search added to results — handled by frontend separately

    let ms = start.elapsed().as_micros() as u64;
    let total = results.len();
    SearchResponse { results, query_time_ms: ms, total_results: total }
}
