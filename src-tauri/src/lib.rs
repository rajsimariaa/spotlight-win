mod hotkey;
mod search;
mod window;
mod commands;

use tauri::Listener;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Build app cache in background before Tauri starts
    std::thread::spawn(|| {
        search::init_cache();
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let handle = app.handle().clone();
            window::apply_spotlight_styling(&handle)?;
            hotkey::register_hotkey(app.handle().clone())?;

            let app_handle = app.handle().clone();
            app.listen("hotkey-toggle", move |_| {
                let _ = window::toggle_window_visibility(&app_handle);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search_query,
            commands::execute_action,
            commands::toggle_window,
            commands::resize_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Spotlight Windows");
}
