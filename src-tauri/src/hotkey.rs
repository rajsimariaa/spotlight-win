use tauri::{AppHandle, Emitter};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_NOREPEAT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetMessageW, TranslateMessage, DispatchMessageW, MSG,
};
use windows::Win32::Foundation::HWND;

const HOTKEY_ID: i32 = 1;
const VK_SPACE: u32 = 0x20;

pub fn register_hotkey(app: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    std::thread::spawn(move || unsafe {
        let hwnd_none: Option<HWND> = None;
        let flags = HOT_KEY_MODIFIERS(MOD_NOREPEAT.0);

        // Try Ctrl+Space first
        use windows::Win32::UI::Input::KeyboardAndMouse::MOD_CONTROL;
        let combo = HOT_KEY_MODIFIERS(flags.0 | MOD_CONTROL.0);
        if RegisterHotKey(hwnd_none, HOTKEY_ID, combo, VK_SPACE).is_ok() {
            eprintln!("[Spotlight] Ctrl+Space registered");
        } else {
            // Fallback to Alt+Space
            use windows::Win32::UI::Input::KeyboardAndMouse::MOD_ALT;
            let combo = HOT_KEY_MODIFIERS(flags.0 | MOD_ALT.0);
            if RegisterHotKey(hwnd_none, HOTKEY_ID, combo, VK_SPACE).is_ok() {
                eprintln!("[Spotlight] Alt+Space registered");
            } else {
                eprintln!("[Spotlight] No hotkey registered");
                return;
            }
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
            if msg.message == 0x0312 && msg.wParam.0 == HOTKEY_ID as usize {
                let _ = app.emit("hotkey-toggle", ());
            }
        }

        let _ = UnregisterHotKey(hwnd_none, HOTKEY_ID);
    });

    Ok(())
}
