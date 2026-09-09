[Setup]
AppId={{A8F3D129-9B21-4E82-B2A8-9842FDC1A910}
AppName=Spotlight Windows
AppVersion=1.1.0
DefaultDirName={autopf}\SpotlightWin
DefaultGroupName=Spotlight Windows
OutputDir=.\Output
OutputBaseFilename=SpotlightWin-Setup-v1.1.0
Compression=lzma2/ultra64
SolidCompression=yes
PrivilegesRequired=lowest
SetupIconFile=..\src-tauri\icons\icon.ico
UninstallDisplayIcon={app}\spotlight-win.exe
VersionInfoVersion=1.1.0.0
VersionInfoDescription=Spotlight Windows - macOS Spotlight for Windows
VersionInfoProductName=Spotlight Windows

[Tasks]
Name: "autostart"; Description: "Start Spotlight automatically when Windows logs in"; Flags: unchecked

[Files]
Source: "..\src-tauri\target\release\spotlight-win.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\src-tauri\target\release\Everything64.dll"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{autoprograms}\Spotlight Windows"; Filename: "{app}\spotlight-win.exe"
Name: "{userstartup}\Spotlight Windows"; Filename: "{app}\spotlight-win.exe"; Tasks: autostart

[Run]
Filename: "{app}\spotlight-win.exe"; Description: "Launch Spotlight Windows"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{app}"

[Code]
procedure CurStepChanged(CurStep: TSetupStep);
var
  ResultCode: Integer;
begin
  if CurStep = ssPostInstall then
  begin
    // Check if Everything is installed
    if not FileExists(ExpandConstant('{pf}\Voidtools\Everything\Everything64.dll')) then
    begin
      if MsgBox('Voidtools Everything is required for file search.'#13#10 +
                'Would you like to download it now?', mbInformation, MB_YESNO) = IDYES then
      begin
        ShellExec('open', 'https://www.voidtools.com/downloads/', '', '', SW_SHOW, ewNoWait, ResultCode);
      end;
    end;
  end;
end;
