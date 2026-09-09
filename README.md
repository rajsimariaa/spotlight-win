# Spotlight Windows

Pixel-perfect macOS Spotlight launcher for Windows 10/11. Built with **Tauri v2 + Rust + React 19**.

## Install

1. Download the [latest release](https://github.com/rajsimariaa/spotlight-win/releases/latest)
2. Run `Spotlight-Windows-v1.1.0-x64-setup.exe`
3. Press **Ctrl+Space** to open

## Usage

| Key | Action |
|-----|--------|
| `Ctrl+Space` | Toggle spotlight |
| `Arrow Up/Down` | Navigate results |
| `Enter` | Open selected item |
| `Escape` | Close spotlight |

## Features

- **Application search** — Start Menu shortcuts, UWP apps, PATH executables
- **File search** — Via Voidtools Everything SDK (all drives)
- **Calculator** — Type math expressions directly: `2+2`, `100*3.14`
- **Timezone** — `time in Tokyo`, `time in New York`
- **Unit conversion** — `100 km in miles`, `72F to C`
- **Quick actions** — Shutdown, Restart, Lock, Sleep, Volume, Dark Mode
- **Glass effect** — Native Windows DWM Acrylic blur
- **Dynamic resizing** — Window grows/shrinks with results

## Windows Defender False Positive

This is a known false positive with Tauri-based applications. The app uses legitimate Windows APIs (global hotkeys, window composition) that some heuristics flag incorrectly.

To exclude from scanning:

1. Open **Windows Security** > **Virus & threat protection**
2. Click **Manage settings** under Virus & threat protection settings
3. Scroll to **Exclusions** > **Add or remove exclusions**
4. Click **Add an exclusion** > **File** and select `spotlight-win.exe`

Or via PowerShell (run as Admin):
```powershell
Add-MpPreference -ExclusionPath "C:\Path\To\spotlight-win.exe"
```

The app is fully open-source — you can audit the code in `src-tauri/src/`.

## Tech Stack

- **Backend:** Rust 1.80+ with Tauri v2
- **Frontend:** React 19, TypeScript, Tailwind CSS, Framer Motion
- **Search:** Voidtools Everything SDK C-API, fuzzy scoring
- **Window:** DWM Acrylic blur via window-vibrancy crate

## Development

```bash
npm install
npm run tauri dev
```

## License

MIT
