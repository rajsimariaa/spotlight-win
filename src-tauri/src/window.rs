use tauri::{Emitter, Manager};

pub fn apply_spotlight_styling(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let window = app.get_webview_window("main")
        .ok_or("Window 'main' not found")?;

    // PRD: rgba(24, 24, 27, 0.75) with 25px blur
    window_vibrancy::apply_acrylic(&window, Some((24, 24, 27, 190)))?;

    #[cfg(target_os = "windows")]
    {
        use std::ffi::c_void;
        use windows::Win32::Foundation::HWND;
        use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};

        let hwnd = window.hwnd()?;
        let h = HWND(hwnd.0 as *mut c_void);

        unsafe {
            // DWMWA_WINDOW_CORNER_PREFERENCE (33) = DWMWCP_ROUND (2) for 12px
            let corner: u32 = 2;
            let _ = DwmSetWindowAttribute(h, DWMWINDOWATTRIBUTE(33),
                &corner as *const _ as *const c_void, std::mem::size_of::<u32>() as u32);

            // DWMWA_BORDER_COLOR (34) = CLR_NONE
            let border_color: u32 = 0x00FFFFFF;
            let _ = DwmSetWindowAttribute(h, DWMWINDOWATTRIBUTE(34),
                &border_color as *const _ as *const c_void, std::mem::size_of::<u32>() as u32);

            // DWMWA_USE_IMMERSIVE_DARK_MODE (20)
            let dark: u32 = 2;
            let _ = DwmSetWindowAttribute(h, DWMWINDOWATTRIBUTE(20),
                &dark as *const _ as *const c_void, std::mem::size_of::<u32>() as u32);
        }

        // Position window near top-center of screen (~20% from top, like macOS Spotlight)
        position_window_top_center(&window)?;
    }

    // PRD: WM_KILLFOCUS auto-hide via window focus event
    let app_clone = app.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(focused) = event {
            if !focused {
                let _ = hide_window(&app_clone);
            }
        }
    });

    Ok(())
}

fn position_window_top_center(window: &tauri::WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::c_void;
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN
        };

        let hwnd = HWND(window.hwnd()?.0 as *mut c_void);

        unsafe {
            let screen_w = GetSystemMetrics(SM_CXSCREEN) as f64;
            let screen_h = GetSystemMetrics(SM_CYSCREEN) as f64;

            // 20% from top for macOS-like positioning
            let y_offset = (screen_h * 0.20) as i32;
            let window_w = 750.0;
            let x = ((screen_w - window_w) / 2.0) as i32;

            use windows::Win32::UI::WindowsAndMessaging::SetWindowPos;
            use windows::Win32::Foundation::RECT;

            let _ = SetWindowPos(
                hwnd,
                None,
                x,
                y_offset,
                0, 0,
                windows::Win32::UI::WindowsAndMessaging::SWP_NOSIZE
                    | windows::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                    | windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
            );
        }
    }
    Ok(())
}

pub fn show_window(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("main") {
        // Reposition to top-center every time it shows
        position_window_top_center(&window)?;
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
