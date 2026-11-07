; Instalador de Rust Turbo Assembler para Windows (Inno Setup 6).
; Lo compila empaquetar-windows.ps1 (o GitHub Actions); espera los archivos en dist\rust-tasm.
; No pide permisos de administrador: se instala en la carpeta del usuario.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#define AppName "Rust Turbo Assembler"
#define Src "..\dist\rust-tasm"

[Setup]
AppId={{6E0C4A2B-7F3D-4B8E-9C51-2A7D3F1E8B40}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher=Fulish-code
AppPublisherURL=https://github.com/Fulish-code/rust-turbo-assembler
AppSupportURL=https://github.com/Fulish-code/rust-turbo-assembler/issues
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
LicenseFile={#Src}\LICENSE
SetupIconFile=..\assets\rust-tasm.ico
UninstallDisplayIcon={app}\rust-tasm.exe
OutputDir=..\dist
OutputBaseFilename=rust-tasm-windows-setup
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "en"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "{#Src}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\rust-tasm.exe"; WorkingDir: "{app}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\rust-tasm.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\rust-tasm.exe"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent
