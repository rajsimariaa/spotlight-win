use tauri::{Emitter, Manager};

pub fn apply_spotlight_styling(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let window = app.get_webview_window("main")
        .ok_or("Window 'main' not found")?;

    // Dark acrylic with blur
    window_vibrancy::apply_acrylic(&window, Some((18, 18, 20, 140)))?;

    #[cfg(target_os = "windows")]
    {
        use std::ffi::c_void;
        use windows::Win32::Foundation::HWND;
        use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};

        let hwnd = window.hwnd()?;
        let h = HWND(hwnd.0 as *mut c_void);

        unsafe {
            // DWMWA_WINDOW_CORNER_PREFERENCE (33) = DWMWCP_DONOTROUND (1)
            let corner: u32 = 1;
            let _ = DwmSetWindowAttribute(h, DWMWINDOWATTRIBUTE(33),
                &corner as *const _ as *const c_void, std::mem::size_of::<u32>() as u32);

            // DWMWA_BORDER_COLOR (34) = DWMWA_CB_NONE — transparent border
            let border_color: u32 = 0x00FFFFFF; // CLR_NONE
            let _ = DwmSetWindowAttribute(h, DWMWINDOWATTRIBUTE(34),
                &border_color as *const _ as *const c_void, std::mem::size_of::<u32>() as u32);

            // DWMWA_CAPTION_COLOR (35) = dark to match bg
            let caption: u32 = 0x00121214;
            let _ = DwmSetWindowAttribute(h, DWMWINDOWATTRIBUTE(35),
                &caption as *const _ as *const c_void, std::mem::size_of::<u32>() as u32);

            // DWMWA_TEXT_COLOR (36) = white
            let text_color: u32 = 0x00FFFFFF;
            let _ = DwmSetWindowAttribute(h, DWMWINDOWATTRIBUTE(36),
                &text_color as *const _ as *const c_void, std::mem::size_of::<u32>() as u32);

            // DWMWA_USE_IMMERSIVE_DARK_MODE (20) = enabled
            let dark: u32 = 2;
            let _ = DwmSetWindowAttribute(h, DWMWINDOWATTRIBUTE(20),
                &dark as *const _ as *const c_void, std::mem::size_of::<u32>() as u32);
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
