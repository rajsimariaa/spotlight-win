use super::{SearchResult, SearchResultCategory};

#[derive(Debug, Clone)]
pub struct QuickAction {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
}

pub fn get_all_actions() -> Vec<QuickAction> {
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
    ]
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
        _ => Ok("Unknown action".into()),
    }
}
