//! Botón «Actualizar»: busca la última versión publicada en GitHub y, si es
//! más nueva, la instala con el mismo instalador que se usó la primera vez.
//!
//! Se usa `curl` (incluido en Linux, macOS y Windows 10+) para no agregar
//! una biblioteca HTTP al programa.

use std::path::PathBuf;
use std::process::Command;

pub const REPO: &str = "Fulish-code/rust-turbo-assembler";
pub const PROFILE_URL: &str = "https://github.com/Fulish-code";

/// Última versión publicada.
#[derive(Clone, Debug)]
pub struct Release {
    pub tag: String,
    pub page: String,
}

/// Cómo se instaló el programa (y por lo tanto cómo actualizarlo).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallKind {
    /// `instalar.sh` en ~/.local/share/rust-tasm.
    LinuxScript,
    /// `instalar-macos.sh` en ~/Applications.
    MacScript,
    /// Instalador de Windows (Inno Setup).
    WindowsSetup,
    /// Compilado a mano u otra ubicación: sólo se abre la página.
    Manual,
}

pub enum Msg {
    Checked(Result<Option<Release>, String>),
    Installed(Result<(), String>),
}

fn curl() -> Command {
    let mut c = Command::new(if cfg!(windows) { "curl.exe" } else { "curl" });
    crate::dosbox::no_console(&mut c);
    c
}

/// `v1.10.2` → (1, 10, 2).
pub fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.trim().trim_start_matches(['v', 'V']);
    let mut it = s.split('.').map(|p| p.trim().parse::<u32>().ok());
    Some((it.next()??, it.next().flatten().unwrap_or(0), it.next().flatten().unwrap_or(0)))
}

pub fn is_newer(tag: &str, current: &str) -> bool {
    matches!((parse_version(tag), parse_version(current)), (Some(a), Some(b)) if a > b)
}

/// Consulta la última Release. `Ok(None)` = ya es la más reciente.
pub fn check(current: &str) -> Result<Option<Release>, String> {
    let out = curl()
        .args(["-fsSL", "--max-time", "20", "-H", "Accept: application/vnd.github+json", "-A", "rust-tasm-updater"])
        .arg(format!("https://api.github.com/repos/{REPO}/releases/latest"))
        .output()
        .map_err(|e| format!("curl: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(if err.is_empty() { format!("curl ({})", out.status) } else { err });
    }
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    let tag = v["tag_name"].as_str().ok_or("tag_name")?.to_string();
    let page = v["html_url"].as_str().map(String::from).unwrap_or_else(|| format!("https://github.com/{REPO}/releases/latest"));
    Ok(is_newer(&tag, current).then_some(Release { tag, page }))
}

fn current_exe() -> Option<PathBuf> {
    let p = std::env::current_exe().ok()?;
    Some(std::fs::canonicalize(&p).unwrap_or(p))
}

pub fn install_kind() -> InstallKind {
    let Some(exe) = current_exe() else { return InstallKind::Manual };
    if cfg!(target_os = "linux") {
        let expected = dirs::data_dir().map(|d| d.join("rust-tasm").join("rust-tasm"));
        let expected = expected.map(|p| std::fs::canonicalize(&p).unwrap_or(p));
        if expected.as_deref() == Some(exe.as_path()) {
            return InstallKind::LinuxScript;
        }
    } else if cfg!(target_os = "macos") {
        let apps = dirs::home_dir().map(|h| h.join("Applications"));
        if apps.is_some_and(|a| exe.starts_with(a)) && exe.to_string_lossy().contains(".app/Contents/MacOS/") {
            return InstallKind::MacScript;
        }
    } else if cfg!(windows) && exe.parent().is_some_and(|d| d.join("unins000.exe").is_file()) {
        return InstallKind::WindowsSetup;
    }
    InstallKind::Manual
}

/// ¿El usuario tiene el ícono del Escritorio? (para que el instalador no
/// vuelva a preguntar y respete lo que eligió la primera vez).
fn has_desktop_icon() -> bool {
    let Some(desk) = dirs::desktop_dir() else { return false };
    desk.join("rust-tasm.desktop").exists() || desk.join(crate::app::APP_NAME).exists()
}

fn run_script(script: &str) -> Result<(), String> {
    let url = format!("https://raw.githubusercontent.com/{REPO}/main/{script}");
    let out = Command::new("bash")
        .arg("-c")
        .arg(format!("set -o pipefail; curl -fsSL '{url}' | bash"))
        .env("RTASM_ESCRITORIO", if has_desktop_icon() { "s" } else { "n" })
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        // Quitar los colores de la terminal y quedarse con las últimas líneas.
        let clean: String = text.replace("\x1b[1;31m", "").replace("\x1b[0m", "").replace("\x1b[1m", "");
        let tail: Vec<&str> = clean.lines().filter(|l| !l.trim().is_empty()).collect();
        Err(tail[tail.len().saturating_sub(3)..].join("\n"))
    }
}

/// Descarga el instalador de Windows y lo abre (Inno Setup pide cerrar el
/// programa para reemplazarlo).
fn run_windows_setup(tag: &str) -> Result<(), String> {
    let url = format!("https://github.com/{REPO}/releases/download/{tag}/rust-tasm-windows-setup.exe");
    let dest: PathBuf = std::env::temp_dir().join(format!("rust-tasm-setup-{tag}.exe"));
    let ok = curl().args(["-fsSL", "--max-time", "600", "-o"]).arg(&dest).arg(&url).status().map_err(|e| e.to_string())?;
    if !ok.success() {
        return Err(format!("curl ({ok}): {url}"));
    }
    Command::new(&dest).spawn().map(|_| ()).map_err(|e| e.to_string())
}

pub fn install(kind: InstallKind, release: &Release) -> Result<(), String> {
    match kind {
        InstallKind::LinuxScript => run_script("instalar.sh"),
        InstallKind::MacScript => run_script("instalar-macos.sh"),
        InstallKind::WindowsSetup => run_windows_setup(&release.tag),
        InstallKind::Manual => open::that_detached(&release.page).map_err(|e| e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions() {
        assert_eq!(parse_version("v1.3.0"), Some((1, 3, 0)));
        assert_eq!(parse_version("1.10"), Some((1, 10, 0)));
        assert!(is_newer("v1.4.0", "1.3.0"));
        assert!(is_newer("v1.10.0", "1.9.9"));
        assert!(!is_newer("v1.3.0", "1.3.0"));
        assert!(!is_newer("v1.2.9", "1.3.0"));
        assert!(!is_newer("nightly", "1.3.0"));
    }

    /// Consulta real a GitHub: `cargo test -- --ignored checks_github`
    #[test]
    #[ignore]
    fn checks_github() {
        let rel = check("0.0.1").expect("sin conexión").expect("debería haber versión más nueva");
        println!("{rel:?}");
        assert!(rel.tag.starts_with('v') && rel.page.contains(REPO));
        assert!(check("999.0.0").unwrap().is_none());
        assert_eq!(install_kind(), InstallKind::Manual); // binario de cargo, no instalado
    }
}
