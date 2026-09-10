use super::{SearchResult, SearchResultCategory};
use windows::core::PCSTR;

#[derive(Debug, Clone)]
pub struct QuickAction {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
}

pub fn get_all_actions() -> Vec<QuickAction> {
    let startup_enabled = is_startup_enabled();
    let startup_label = if startup_enabled { "Disable" } else { "Enable" };
    let startup_desc = if startup_enabled { "Remove from startup" } else { "Run on Windows startup" };

    vec![
        QuickAction { id: "sleep".into(), name: "Sleep".into(), description: "Put computer to sleep".into(), icon: "moon".into() },
        QuickAction { id: "lock".into(), name: "Lock".into(), description: "Lock your computer".into(), icon: "lock".into() },
        QuickAction { id: "shutdown".into(), name: "Shut Down".into(), description: "Shut down your computer".into(), icon: "power".into() },
        QuickAction { id: "restart".into(), name: "Restart".into(), description: "Restart your computer".into(), icon: "refresh-cw".into() },
        QuickAction { id: "empty_recycle".into(), name: "Empty Recycle Bin".into(), description: "Delete all files in Recycle Bin".into(), icon: "trash-2".into() },
        QuickAction { id: "dark_mode".into(), name: "Toggle Dark Mode".into(), description: "Switch dark/light theme".into(), icon: "palette".into() },
        QuickAction { id: "volume_up".into(), name: "Volume Up".into(), description: "Increase system volume".into(), icon: "volume-2".into() },
        QuickAction { id: "volume_down".into(), name: "Volume Down".into(), description: "Decrease system volume".into(), icon: "volume-1".into() },
        QuickAction { id: "volume_mute".into(), name: "Mute".into(), description: "Toggle mute".into(), icon: "volume-x".into() },
        QuickAction { id: "toggle_startup".into(), name: format!("{} Startup", startup_label), description: startup_desc.into(), icon: "rocket".into() },
    ]
}

fn pcstr(s: &str) -> PCSTR {
    PCSTR::from_raw(s.as_ptr() as *const u8)
}

pub fn is_startup_enabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Registry::*;

        unsafe {
            let mut hkey = HKEY::default();
            let result = RegOpenKeyExA(
                HKEY_CURRENT_USER,
                pcstr("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
                Some(0),
                KEY_READ,
                &mut hkey,
            );
            if result.is_ok() {
                let mut buf_len: u32 = 256;
                let mut buf = [0u8; 256];
                let mut reg_type = REG_SZ;
                let result = RegQueryValueExA(
                    hkey,
                    pcstr("SpotlightWindows"),
                    None,
                    Some(&mut reg_type),
                    Some(buf.as_mut_ptr()),
                    Some(&mut buf_len),
                );
                let _ = RegCloseKey(hkey);
                return result.is_ok() && buf_len > 0;
            }
        }
    }
    false
}

pub fn set_startup(enable: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Registry::*;

        let exe_path = std::env::current_exe()
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .to_string();

        unsafe {
            let mut hkey = HKEY::default();
            let result = RegOpenKeyExA(
                HKEY_CURRENT_USER,
                pcstr("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
                Some(0),
                KEY_SET_VALUE,
                &mut hkey,
            );
            if result.is_err() {
                return Err("Failed to open registry key".into());
            }

            if enable {
                let value = format!("\"{}\"", exe_path);
                let result = RegSetValueExA(
                    hkey,
                    pcstr("SpotlightWindows"),
                    Some(0),
                    REG_SZ,
                    Some(value.as_bytes()),
                );
                let _ = RegCloseKey(hkey);
                if result.is_ok() { Ok(()) } else { Err("Failed to set registry value".into()) }
            } else {
                let result = RegDeleteValueA(hkey, pcstr("SpotlightWindows"));
                let _ = RegCloseKey(hkey);
                if result.is_ok() { Ok(()) } else { Err("Failed to delete registry value".into()) }
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("Startup not supported on this OS".into())
    }
}

pub fn search_actions(query: &str) -> Vec<SearchResult> {
    let actions = get_all_actions();
    let query_lower = query.to_lowercase();
    actions.iter().filter_map(|action| {
        let score = crate::search::fuzzy::calculate_fuzzy_score(&query_lower, &action.name.to_lowercase());
        if score > 0.0 {
            Some(SearchResult {
                id: format!("action_{}", action.id),
                name: action.name.clone(),
                path: action.description.clone(),
                category: SearchResultCategory::Action,
                icon: Some(action.icon.clone()),
                score,
                metadata: Some(action.id.clone()),
            })
        } else {
            None
        }
    }).collect()
}

fn run_hidden(cmd: &str, args: &[&str]) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let _ = std::process::Command::new(cmd)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output();
}

pub fn execute_action(action_id: &str) -> Result<String, String> {
    if let Some(path) = action_id.strip_prefix("open:") {
        open::that(path).map_err(|e| e.to_string())?;
        return Ok(format!("Opened: {}", path));
    }

    let actions = get_all_actions();
    let action = actions.iter().find(|a| a.id == action_id)
        .ok_or_else(|| format!("Unknown action: {}", action_id))?;

    match action.id.as_str() {
        "sleep" => {
            #[cfg(target_os = "windows")]
            unsafe {
                windows::Win32::System::Power::SetSuspendState(false, false, false);
            }
            Ok("Sleep initiated".into())
        }
        "lock" => {
            run_hidden("rundll32.exe", &["user32.dll,LockWorkStation"]);
            Ok("Workstation locked".into())
        }
        "shutdown" => {
            run_hidden("shutdown", &["/s", "/t", "0"]);
            Ok("Shutdown initiated".into())
        }
        "restart" => {
            run_hidden("shutdown", &["/r", "/t", "0"]);
            Ok("Restart initiated".into())
        }
        "empty_recycle" => {
            run_hidden("powershell", &["-NoProfile", "-WindowStyle", "Hidden", "-Command", "Clear-RecycleBin -Force -ErrorAction SilentlyContinue"]);
            Ok("Recycle Bin emptied".into())
        }
        "dark_mode" => {
            run_hidden("reg", &["add", r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Themes\Personalize", "/v", "AppsUseLightTheme", "/t", "REG_DWORD", "/d", "0", "/f"]);
            Ok("Dark mode toggled".into())
        }
        "volume_up" => {
            run_hidden("powershell", &["-NoProfile", "-WindowStyle", "Hidden", "-Command", "$obj = New-Object -ComObject WScript.Shell; for($i=0;$i -lt 5;$i++){$obj.SendKeys([char]175)}"]);
            Ok("Volume increased".into())
        }
        "volume_down" => {
            run_hidden("powershell", &["-NoProfile", "-WindowStyle", "Hidden", "-Command", "$obj = New-Object -ComObject WScript.Shell; for($i=0;$i -lt 5;$i++){$obj.SendKeys([char]174)}"]);
            Ok("Volume decreased".into())
        }
        "volume_mute" => {
            run_hidden("powershell", &["-NoProfile", "-WindowStyle", "Hidden", "-Command", "$obj = New-Object -ComObject WScript.Shell; $obj.SendKeys([char]173)"]);
            Ok("Volume toggled".into())
        }
        "toggle_startup" => {
            let enabled = !is_startup_enabled();
            set_startup(enabled)?;
            if enabled { Ok("Added to startup".into()) } else { Ok("Removed from startup".into()) }
        }
        _ => Ok("Unknown action".into()),
    }
}
