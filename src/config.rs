//! Preferencias y sesión, guardadas como JSON en la carpeta de configuración
//! del usuario (Linux: ~/.config/rust-tasm/config.json).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const MAX_RECENT: usize = 20;
pub const DEFAULT_TASM_ARGS: &str = "/zi";
pub const DEFAULT_TLINK_ARGS: &str = "/v";

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum IdeTheme {
    Blue,
    Dark,
    Light,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    pub language: String,
    pub ide_theme: IdeTheme,
    pub editor_dark: bool,
    pub font_size: f32,
    pub word_wrap: bool,
    pub auto_indent: bool,
    pub show_toolbar: bool,
    pub show_statusbar: bool,
    pub show_output: bool,
    pub show_terminal: bool,
    pub auto_scroll: bool,
    pub use_launcher: bool,
    pub restore_session: bool,
    pub launch_maximized: bool,
    pub restore_window: bool,
    pub tasm_args: String,
    pub tlink_args: String,
    pub dosbox_path: String,
    pub bin_dir: String,
    pub silent_tools: bool,
    pub recent_files: Vec<PathBuf>,
    pub session_files: Vec<PathBuf>,
    pub session_active: Option<PathBuf>,
    pub window: Option<[f32; 4]>,
    pub terminal_history: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: crate::i18n::system_language(),
            ide_theme: IdeTheme::Dark,
            editor_dark: true,
            font_size: 15.0,
            word_wrap: false,
            auto_indent: true,
            show_toolbar: true,
            show_statusbar: true,
            show_output: true,
            show_terminal: true,
            auto_scroll: true,
            use_launcher: true,
            restore_session: true,
            launch_maximized: false,
            restore_window: true,
            tasm_args: DEFAULT_TASM_ARGS.into(),
            tlink_args: DEFAULT_TLINK_ARGS.into(),
            dosbox_path: String::new(),
            bin_dir: String::new(),
            silent_tools: true,
            recent_files: Vec::new(),
            session_files: Vec::new(),
            session_active: None,
            window: None,
            terminal_history: Vec::new(),
        }
    }
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(std::env::temp_dir).join("rust-tasm")
}

fn config_file() -> PathBuf {
    config_dir().join("config.json")
}

impl Settings {
    pub fn load() -> Self {
        std::fs::read_to_string(config_file())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let _ = std::fs::create_dir_all(config_dir());
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let tmp = config_file().with_extension("json.tmp");
            if std::fs::write(&tmp, s).is_ok() {
                let _ = std::fs::rename(tmp, config_file());
            }
        }
    }

    pub fn push_recent(&mut self, p: &Path) {
        self.recent_files.retain(|x| x != p);
        self.recent_files.insert(0, p.to_path_buf());
        self.recent_files.truncate(MAX_RECENT);
    }
}

/// Carpeta con TASM.EXE/TLINK.EXE/TD.EXE.
pub fn find_bin_dir(custom: &str) -> Option<PathBuf> {
    let has_tasm = |d: &Path| d.join("TASM.EXE").is_file() || d.join("tasm.exe").is_file();
    if !custom.trim().is_empty() {
        let p = PathBuf::from(custom.trim());
        if has_tasm(&p) {
            return Some(p);
        }
    }
    let cands: Vec<PathBuf> = resource_roots().into_iter().flat_map(|r| [r.join("BIN"), r.join("bin")]).collect();
    cands.into_iter().find(|d| has_tasm(d)).map(|d| d.canonicalize().unwrap_or(d))
}

/// Carpeta `docs/` con los manuales de TASM (se busca igual que BIN).
pub fn find_docs_dir() -> Option<PathBuf> {
    let ok = |d: &Path| d.join("Manual TASM").is_dir() || d.join("Manual-TASM(completo).pdf").is_file();
    let cands: Vec<PathBuf> = resource_roots().into_iter().map(|r| r.join("docs")).collect();
    cands.into_iter().find(|d| ok(d)).map(|d| d.canonicalize().unwrap_or(d))
}

/// Carpetas candidatas para recursos (BIN, docs, locales): junto al ejecutable,
/// sus carpetas superiores y, en macOS, `Contents/Resources` del paquete .app.
pub fn resource_roots() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(me) = std::env::current_exe() {
        let me = me.canonicalize().unwrap_or(me);
        if let Some(exe_dir) = me.parent() {
            v.push(exe_dir.join("../Resources"));
            let mut dir = Some(exe_dir.to_path_buf());
            for _ in 0..5 {
                if let Some(d) = dir {
                    v.push(d.clone());
                    dir = d.parent().map(Path::to_path_buf);
                }
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        v.push(cwd);
    }
    if let Some(d) = dirs::data_dir() {
        v.push(d.join("rust-tasm"));
    }
    #[cfg(unix)]
    v.push("/usr/share/rust-tasm".into());
    v
}

/// Carpeta `locales/` en disco (opcional; si no existe se usan los idiomas incrustados).
pub fn find_locales_dir() -> Option<PathBuf> {
    resource_roots().into_iter().map(|r| r.join("locales")).find(|d| d.is_dir())
}
