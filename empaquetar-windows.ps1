# Crea el instalador de Windows: dist\rust-tasm-windows-setup.exe
# Incluye el programa, TASM/TLINK/TD, los manuales, los ejemplos y DOSBox Staging.
# Requiere: Rust (MSVC) e Inno Setup 6 (ISCC.exe). Lo usa GitHub Actions.
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot

$DosboxVersion = 'v0.83.0'
$DosboxUrl = "https://github.com/dosbox-staging/dosbox-staging/releases/download/$DosboxVersion/dosbox-staging-windows-x64-$DosboxVersion.zip"
$Version = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value

cargo build --release
if ($LASTEXITCODE -ne 0) { throw 'La compilación falló' }
$Target = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { 'target' }

# Debe funcionar sin "Visual C++ Redistributable" (ver .cargo/config.toml).
$Bytes = [IO.File]::ReadAllBytes((Resolve-Path "$Target\release\rust-tasm.exe"))
if ([Text.Encoding]::ASCII.GetString($Bytes) -match '(?i)vcruntime140|msvcp140') {
    throw 'rust-tasm.exe depende de VCRUNTIME140.dll; falta el enlace estático (crt-static)'
}

if (Test-Path dist) { Remove-Item -Recurse -Force dist }
$App = 'dist\rust-tasm'
New-Item -ItemType Directory -Force $App | Out-Null
Copy-Item "$Target\release\rust-tasm.exe" $App
Copy-Item -Recurse BIN, docs, locales, examples $App
Copy-Item README.md, LICENSE $App

# DOSBox Staging (portable) en dist\rust-tasm\dosbox
$Tmp = Join-Path ([IO.Path]::GetTempPath()) ("rtasm-dosbox-" + [guid]::NewGuid())
New-Item -ItemType Directory -Force $Tmp | Out-Null
try {
    Invoke-WebRequest -Uri $DosboxUrl -OutFile "$Tmp\dosbox.zip" -UseBasicParsing
    Expand-Archive "$Tmp\dosbox.zip" "$Tmp\x"
    $Exe = Get-ChildItem "$Tmp\x" -Recurse -Filter dosbox.exe | Select-Object -First 1
    if (-not $Exe) { throw 'No se encontró dosbox.exe en el paquete de DOSBox Staging' }
    Copy-Item -Recurse $Exe.DirectoryName "$App\dosbox"
    Remove-Item -Force -ErrorAction SilentlyContinue "$App\dosbox\dosbox_with_debugger.exe"
} finally {
    Remove-Item -Recurse -Force $Tmp
}

$Iscc = (Get-Command iscc -ErrorAction SilentlyContinue).Source
if (-not $Iscc) {
    $Iscc = @("${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe", "$env:ProgramFiles\Inno Setup 6\ISCC.exe",
              "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe") | Where-Object { Test-Path $_ } | Select-Object -First 1
}
if (-not $Iscc) { throw 'No encontré Inno Setup 6 (ISCC.exe). Instálalo con: winget install JRSoftware.InnoSetup' }
& $Iscc "/DAppVersion=$Version" windows\rust-tasm.iss
if ($LASTEXITCODE -ne 0) { throw 'Inno Setup falló' }

$Setup = 'dist\rust-tasm-windows-setup.exe'
$Hash = (Get-FileHash $Setup -Algorithm SHA256).Hash.ToLower()
# Fin de línea LF para que "sha256sum -c" funcione también en Linux y macOS.
[IO.File]::WriteAllText((Join-Path $PWD "$Setup.sha256"), "$Hash  rust-tasm-windows-setup.exe`n")
Get-ChildItem dist
