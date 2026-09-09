use tauri::{Emitter, Manager};

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Dwm::{
    DwmSetWindowAttribute, DWMWINDOWATTRIBUTE,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongW, SetWindowLongW, SetWindowPos, GWL_EXSTYLE,
    HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
    WS_EX_LAYERED, WS_EX_TOOLWINDOW,
};

pub fn apply_spotlight_styling(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let window = app.get_webview_window("main")
        .ok_or("Window 'main' not found")?;

    #[cfg(target_os = "windows")]
    {
        apply_acrylic(&window)?;
        apply_rounded_corners(&window)?;
        apply_window_flags(&window)?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn apply_acrylic(window: &tauri::WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    window_vibrancy::apply_acrylic(window, Some((18, 18, 18, 125)))?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn apply_rounded_corners(window: &tauri::WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    use std::ffi::c_void;
    let hwnd = window.hwnd()?;
    let hwnd_raw = hwnd.0 as *mut c_void;
    unsafe {
        let preference: u32 = 3;
        DwmSetWindowAttribute(
            HWND(hwnd_raw as *mut _),
            DWMWINDOWATTRIBUTE(33),
            &preference as *const _ as *const c_void,
            std::mem::size_of_val(&preference) as u32,
        )?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn apply_window_flags(window: &tauri::WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    use std::ffi::c_void;
    let hwnd = window.hwnd()?;
    let hwnd_raw = hwnd.0 as *mut c_void;
    unsafe {
        let ex_style = GetWindowLongW(HWND(hwnd_raw as *mut _), GWL_EXSTYLE);
        SetWindowLongW(
            HWND(hwnd_raw as *mut _),
            GWL_EXSTYLE,
            ex_style | WS_EX_LAYERED.0 as i32 | WS_EX_TOOLWINDOW.0 as i32,
        );
        SetWindowPos(
            HWND(hwnd_raw as *mut _),
            Some(HWND_TOPMOST),
            0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
        )?;
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
