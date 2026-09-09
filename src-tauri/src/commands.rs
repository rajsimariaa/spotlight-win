use std::time::Instant;
use tauri::Manager;

use crate::search::{SearchResult, SearchResponse};
use crate::search::app_index;
use crate::search::everything;
use crate::search::evaluator;
use crate::search::quick_actions;

#[tauri::command]
pub fn search_query(query: String) -> SearchResponse {
    let start = Instant::now();
    let mut results: Vec<SearchResult> = Vec::new();

    if query.trim().is_empty() {
        return SearchResponse { results, query_time_ms: 0, total_results: 0 };
    }

    let apps = app_index::get_all_applications();
    let app_results = app_index::search_applications(&query, &apps);
    results.extend(app_results);

    if let Some(eval_result) = evaluator::evaluate_expression(&query) {
        results.push(eval_result);
    }

    if everything::is_everything_available() {
        let file_results = everything::query_files(&query, 50);
        results.extend(file_results);
    }

    let action_results = quick_actions::search_actions(&query);
    results.extend(action_results);

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    let query_time = start.elapsed().as_millis() as u64;
    let total = results.len();

    SearchResponse { results, query_time_ms: query_time, total_results: total }
}

#[tauri::command]
pub fn execute_action(action_id: String) -> Result<String, String> {
    if let Some(path) = action_id.strip_prefix("open:") {
        open::that(path).map_err(|e| e.to_string())?;
        return Ok(format!("Opened: {}", path));
    }
    quick_actions::execute_action(&action_id)
}

#[tauri::command]
pub fn get_applications() -> Vec<crate::search::app_index::AppEntry> {
    app_index::get_all_applications()
}

#[tauri::command]
pub fn evaluate_expression(expression: String) -> Option<SearchResult> {
    evaluator::evaluate_expression(&expression)
}

#[tauri::command]
pub fn check_everything_status() -> bool {
    everything::is_everything_available()
}

#[tauri::command]
pub fn toggle_window(app: tauri::AppHandle) -> Result<(), String> {
    crate::window::toggle_window_visibility(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resize_window(app: tauri::AppHandle, height: f64) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let new_height = height.clamp(60.0, 420.0);
        window.set_size(tauri::LogicalSize::new(750.0, new_height)).map_err(|e| e.to_string())?;
    }
    Ok(())
}
