use tauri::{AppHandle, Emitter};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_NOREPEAT,
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
        let flags = HOT_KEY_MODIFIERS(MOD_NOREPEAT.0 | MOD_ALT.0);

        if RegisterHotKey(hwnd_none, HOTKEY_ID, flags, VK_SPACE).is_err() {
            return;
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
