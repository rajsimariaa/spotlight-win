use super::{SearchResult, SearchResultCategory};

#[derive(Debug, Clone)]
pub struct QuickAction {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub command: ActionCommand,
}

#[derive(Debug, Clone)]
pub enum ActionCommand {
    Sleep,
    Lock,
    Shutdown,
    Restart,
    EmptyRecycleBin,
    ToggleDarkMode,
    VolumeUp,
    VolumeDown,
    VolumeMute,
}

pub fn get_all_actions() -> Vec<QuickAction> {
    vec![
        QuickAction {
            id: "sleep".into(),
            name: "Sleep".into(),
            description: "Put computer to sleep".into(),
            icon: "moon".into(),
            command: ActionCommand::Sleep,
        },
        QuickAction {
            id: "lock".into(),
            name: "Lock".into(),
            description: "Lock your computer".into(),
            icon: "lock".into(),
            command: ActionCommand::Lock,
        },
        QuickAction {
            id: "shutdown".into(),
            name: "Shut Down".into(),
            description: "Shut down your computer".into(),
            icon: "power".into(),
            command: ActionCommand::Shutdown,
        },
        QuickAction {
            id: "restart".into(),
            name: "Restart".into(),
            description: "Restart your computer".into(),
            icon: "refresh-cw".into(),
            command: ActionCommand::Restart,
        },
        QuickAction {
            id: "empty_recycle".into(),
            name: "Empty Recycle Bin".into(),
            description: "Permanently delete all files in Recycle Bin".into(),
            icon: "trash-2".into(),
            command: ActionCommand::EmptyRecycleBin,
        },
        QuickAction {
            id: "dark_mode".into(),
            name: "Toggle Dark Mode".into(),
            description: "Switch between dark and light theme".into(),
            icon: "palette".into(),
            command: ActionCommand::ToggleDarkMode,
        },
        QuickAction {
            id: "volume_up".into(),
            name: "Volume Up".into(),
            description: "Increase system volume".into(),
            icon: "volume-2".into(),
            command: ActionCommand::VolumeUp,
        },
        QuickAction {
            id: "volume_down".into(),
            name: "Volume Down".into(),
            description: "Decrease system volume".into(),
            icon: "volume-1".into(),
            command: ActionCommand::VolumeDown,
        },
        QuickAction {
            id: "volume_mute".into(),
            name: "Mute".into(),
            description: "Toggle mute".into(),
            icon: "volume-x".into(),
            command: ActionCommand::VolumeMute,
        },
    ]
}

pub fn search_actions(query: &str) -> Vec<SearchResult> {
    let actions = get_all_actions();
    let query_lower = query.to_lowercase();

    actions.iter()
        .filter_map(|action| {
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
        })
        .collect()
}

pub fn execute_action(action_id: &str) -> Result<String, String> {
    let actions = get_all_actions();
    let action = actions.iter().find(|a| a.id == action_id)
        .ok_or_else(|| format!("Unknown action: {}", action_id))?;

    match &action.command {
        ActionCommand::Sleep => {
            #[cfg(target_os = "windows")]
            unsafe {
                use windows::Win32::System::Power::SetSuspendState;
                let _ = SetSuspendState(false, false, false);
            }
            Ok("Sleep initiated".into())
        }
        ActionCommand::Lock => {
            // Use rundll32 to lock workstation (works with all windows crate versions)
            let _ = std::process::Command::new("rundll32.exe")
                .args(["user32.dll,LockWorkStation"])
                .spawn();
            Ok("Workstation locked".into())
        }
        ActionCommand::Shutdown => {
            std::process::Command::new("shutdown")
                .args(["/s", "/t", "0"])
                .spawn()
                .map_err(|e| e.to_string())?;
            Ok("Shutdown initiated".into())
        }
        ActionCommand::Restart => {
            std::process::Command::new("shutdown")
                .args(["/r", "/t", "0"])
                .spawn()
                .map_err(|e| e.to_string())?;
            Ok("Restart initiated".into())
        }
        ActionCommand::EmptyRecycleBin => {
            // Use PowerShell to empty recycle bin
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", "Clear-RecycleBin -Force -ErrorAction SilentlyContinue"])
                .output();
            Ok("Recycle Bin emptied".into())
        }
        ActionCommand::ToggleDarkMode => {
            let _ = std::process::Command::new("reg")
                .args([
                    "add",
                    r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                    "/v",
                    "AppsUseLightTheme",
                    "/t",
                    "REG_DWORD",
                    "/d",
                    "0",
                    "/f",
                ])
                .output();
            Ok("Dark mode toggled".into())
        }
        ActionCommand::VolumeUp => {
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", "$obj = New-Object -ComObject WScript.Shell; for($i=0;$i -lt 5;$i++){$obj.SendKeys([char]175)}"])
                .output();
            Ok("Volume increased".into())
        }
        ActionCommand::VolumeDown => {
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", "$obj = New-Object -ComObject WScript.Shell; for($i=0;$i -lt 5;$i++){$obj.SendKeys([char]174)}"])
                .output();
            Ok("Volume decreased".into())
        }
        ActionCommand::VolumeMute => {
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", "$obj = New-Object -ComObject WScript.Shell; $obj.SendKeys([char]173)"])
                .output();
            Ok("Volume toggled".into())
        }
    }
}
