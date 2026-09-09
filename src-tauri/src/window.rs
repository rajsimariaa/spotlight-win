use tauri::{Emitter, Manager};

pub fn apply_spotlight_styling(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let window = app.get_webview_window("main")
        .ok_or("Window 'main' not found")?;

    // Apply native acrylic blur using the window-vibrancy crate
    window_vibrancy::apply_acrylic(&window, Some((18, 18, 18, 125)))?;

    // Set rounded corners
    #[cfg(target_os = "windows")]
    {
        use std::ffi::c_void;
        use windows::Win32::Foundation::HWND;
        use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};

        let hwnd = window.hwnd()?;
        let hwnd_raw = hwnd.0 as *mut c_void;
        unsafe {
            let preference: u32 = 3; // DWMWCP_ROUND
            let _ = DwmSetWindowAttribute(
                HWND(hwnd_raw as *mut _),
                DWMWINDOWATTRIBUTE(33),
                &preference as *const _ as *const c_void,
                std::mem::size_of_val(&preference) as u32,
            );
        }
    }

    Ok(())
}

pub fn show_window(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("main") {
        window.show()?;
        window.set_focus()?;
        app.emit("spotlight-show", ())?;
    }
    Ok(())
}

pub fn hide_window(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide()?;
        app.emit("spotlight-hide", ())?;
    }
    Ok(())
}

pub fn toggle_window_visibility(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible()? {
            hide_window(app)?;
        } else {
            show_window(app)?;
        }
    }
    Ok(())
}
