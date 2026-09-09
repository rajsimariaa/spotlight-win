use tauri::{AppHandle, Emitter};

use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetMessageW, TranslateMessage, DispatchMessageW, MSG,
};
use windows::Win32::Foundation::HWND;

const HOTKEY_ID: i32 = 1;
const VK_SPACE: u32 = 0x20;

pub fn register_hotkey(app: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    std::thread::spawn(move || unsafe {
        eprintln!("[Spotlight] Hotkey thread started");

        let hwnd_none: Option<HWND> = None;
        let mut registered = false;

        // Try 1: Ctrl+Space (least conflict)
        {
            use windows::Win32::UI::Input::KeyboardAndMouse::MOD_CONTROL;
            match RegisterHotKey(hwnd_none, HOTKEY_ID, MOD_CONTROL, VK_SPACE) {
                Ok(()) => {
                    eprintln!("[Spotlight] Ctrl+Space registered");
                    registered = true;
                }
                Err(e) => eprintln!("[Spotlight] Ctrl+Space failed: {}", e),
            }
        }

        // Try 2: Alt+Space
        if !registered {
            use windows::Win32::UI::Input::KeyboardAndMouse::MOD_ALT;
            match RegisterHotKey(hwnd_none, HOTKEY_ID, MOD_ALT, VK_SPACE) {
                Ok(()) => {
                    eprintln!("[Spotlight] Alt+Space registered");
                    registered = true;
                }
                Err(e) => eprintln!("[Spotlight] Alt+Space failed: {}", e),
            }
        }

        // Try 3: Ctrl+Alt+Space
        if !registered {
            use windows::Win32::UI::Input::KeyboardAndMouse::{MOD_CONTROL, MOD_ALT};
            let combo = HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_ALT.0);
            match RegisterHotKey(hwnd_none, HOTKEY_ID, combo, VK_SPACE) {
                Ok(()) => {
                    eprintln!("[Spotlight] Ctrl+Alt+Space registered");
                    registered = true;
                }
                Err(e) => eprintln!("[Spotlight] Ctrl+Alt+Space failed: {}", e),
            }
        }

        if !registered {
            eprintln!("[Spotlight] ERROR: No hotkey could be registered!");
            return;
        }

        // Win32 message loop
        let mut msg = MSG::default();
        loop {
            let ret = GetMessageW(&mut msg, None, 0, 0);
            if !ret.as_bool() {
                break;
            }

            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);

            // WM_HOTKEY = 0x0312
            if msg.message == 0x0312 && msg.wParam.0 == HOTKEY_ID as usize {
                eprintln!("[Spotlight] Hotkey detected");
                let _ = app.emit("hotkey-toggle", ());
            }
        }

        let _ = UnregisterHotKey(hwnd_none, HOTKEY_ID);
    });

    Ok(())
}
