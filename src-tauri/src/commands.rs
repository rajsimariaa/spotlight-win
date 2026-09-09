use tauri::Manager;
use crate::search;

#[tauri::command]
pub fn search_query(query: String) -> search::SearchResponse {
    search::search(&query)
}

#[tauri::command]
pub fn execute_action(action_id: String) -> Result<String, String> {
    if let Some(path) = action_id.strip_prefix("open:") {
        open::that(path).map_err(|e| e.to_string())?;
        return Ok(format!("Opened: {}", path));
    }
    if let Some(query) = action_id.strip_prefix("web:") {
        let url = format!("https://www.google.com/search?q={}", urlencoding::encode(query));
        open::that(&url).map_err(|e| e.to_string())?;
        return Ok(format!("Searching: {}", query));
    }
    crate::search::quick_actions::execute_action(&action_id)
}

#[tauri::command]
pub fn toggle_window(app: tauri::AppHandle) -> Result<(), String> {
    crate::window::toggle_window_visibility(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resize_window(app: tauri::AppHandle, height: f64) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let h = height.clamp(54.0, 440.0);
        window.set_size(tauri::LogicalSize::new(750.0, h)).map_err(|e| e.to_string())?;
    }
    Ok(())
}
