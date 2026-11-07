//! Ventana principal del IDE: menús, barra de herramientas, pestañas
//! acoplables (editores, Salida, Terminal), diálogos y barra de estado.

use crate::config::{self, IdeTheme, Settings};
use crate::dosbox::{self, CancelHandle, DosBox};
use crate::editor::{self, Document, EditOp, EditorStyle, FindQuery};
use crate::highlight::Palette;
use crate::i18n::I18n;
use crate::theme;
use crate::toolchain::{self, Event, LogKind, Step};
use eframe::egui::{
    self, Align, Color32, Key, KeyboardShortcut, Layout, Modifiers, RichText, TextStyle, Vec2, ViewportCommand,
};
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer, tab_viewer::OnCloseResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};
use std::time::Instant;

pub const APP_NAME: &str = "Rust Turbo Assembler";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const ORIGINAL_REPO: &str = "https://github.com/ljnath/GUI-Turbo-Assembler";

const EXAMPLES: &[(&str, &str)] = &[
    ("HOLA.ASM", include_str!("../examples/HOLA.ASM")),
    ("SUMA.ASM", include_str!("../examples/SUMA.ASM")),
    ("CICLO.ASM", include_str!("../examples/CICLO.ASM")),
    ("INVERSA.ASM", include_str!("../examples/INVERSA.ASM")),
    ("NUMERO.ASM", include_str!("../examples/NUMERO.ASM")),
    ("MACROS.ASM", include_str!("../examples/MACROS.ASM")),
    ("COLORES.ASM", include_str!("../examples/COLORES.ASM")),
    ("GRAFICO.ASM", include_str!("../examples/GRAFICO.ASM")),
];

const NEW_FILE_TEMPLATE: &str = ".MODEL SMALL\n.STACK 100h\n\n.DATA\n\n.CODE\ninicio:\n\tmov ax, @data\n\tmov ds, ax\n\n\t\n\n\tmov ax, 4C00h\n\tint 21h\nEND inicio\n";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Editor(u64),
    Output,
    Terminal,
}

/// Pestaña tal como se guarda en disco (los editores por índice de sesión).
#[derive(Serialize, Deserialize, Clone, Copy)]
enum SavedTab {
    Editor(usize),
    Output,
    Terminal,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Assemble,
    Build,
    AssembleBuild,
    Run,
    AssembleBuildRun,
    Debug,
}

/// Lo que se hace al responder un diálogo de confirmación.
#[derive(Clone, Debug)]
enum Flow {
    None,
    /// Continuar con la acción (re-evalúa las comprobaciones).
    Proceed(Action, u64),
    /// Ejecutar los pasos indicados sin más preguntas.
    Steps(Vec<Step>, u64),
    SaveThen(Box<Flow>, u64),
    Log(LogKind, String),
    CloseDoc(u64),
    SaveAndCloseDoc(u64),
    Quit,
    SaveAllAndQuit,
    RemoveRecent(PathBuf),
    ClearRecent,
}

struct Confirm {
    title: String,
    message: String,
    yes: Flow,
    no: Flow,
    cancel: bool,
}

struct LogLine {
    time: String,
    kind: LogKind,
    text: String,
    jump: Option<(PathBuf, usize)>,
}

#[derive(Default)]
struct FindState {
    open: bool,
    replace: bool,
    needle: String,
    replacement: String,
    match_case: bool,
    whole_word: bool,
    status: String,
    focus: bool,
}

struct TaskState {
    rx: Receiver<Event>,
    cancel: CancelHandle,
    label: String,
    started: Instant,
    terminal: bool,
}

struct TerminalState {
    lines: Vec<(LogKind, String)>,
    input: String,
    history_pos: Option<usize>,
    focus: bool,
}

pub struct State {
    pub settings: Settings,
    i18n: Arc<I18n>,
    docs: HashMap<u64, Document>,
    next_id: u64,
    untitled_counter: usize,
    active_doc: Option<u64>,
    logs: Vec<LogLine>,
    log_scroll: bool,
    terminal: TerminalState,
    task: Option<TaskState>,
    dosbox: Option<DosBox>,
    bin_dir: Option<PathBuf>,
    find: FindState,
    confirm: Option<Confirm>,
    message: Option<(String, String)>,
    prefs: Option<Settings>,
    prefs_tab: usize,
    about: bool,
    help: bool,
    goto: Option<String>,
    properties: Option<u64>,
    status: String,
    pending: Vec<Flow>,
    quit_confirmed: bool,
    open_tabs: Vec<Tab>,
    close_tabs: Vec<Tab>,
    focus_tab: Option<Tab>,
    theme_applied: Option<IdeTheme>,
}

pub struct App {
    dock: DockState<Tab>,
    s: State,
    devshot: crate::devshot::DevShot,
}

fn now_hms() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Hora local aproximada usando el desfase de la zona (si se puede leer).
    let off = local_offset_secs();
    let t = (secs as i64 + off).rem_euclid(86_400);
    format!("{:02}:{:02}:{:02}", t / 3600, (t / 60) % 60, t % 60)
}

fn local_offset_secs() -> i64 {
    static OFF: std::sync::OnceLock<i64> = std::sync::OnceLock::new();
    *OFF.get_or_init(|| {
        #[cfg(unix)]
        {
            if let Ok(o) = std::process::Command::new("date").arg("+%z").output() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if s.len() == 5 {
                    let sign = if s.starts_with('-') { -1 } else { 1 };
                    let h: i64 = s[1..3].parse().unwrap_or(0);
                    let m: i64 = s[3..5].parse().unwrap_or(0);
                    return sign * (h * 3600 + m * 60);
                }
            }
        }
        0
    })
}

fn fmt_time(t: std::io::Result<std::time::SystemTime>) -> String {
    let Ok(t) = t else { return "-".into() };
    let secs = t.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0) + local_offset_secs();
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    // Conversión de días a fecha civil (algoritmo de Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{d:02}/{m:02}/{y} {:02}:{:02}:{:02}", rem / 3600, (rem / 60) % 60, rem % 60)
}

/// Reglas de nombre de Turbo Assembler/Linker: máximo 8 caracteres, sin espacios.
fn validate_filename(p: &Path, t: &dyn Fn(&str) -> String) -> Result<(), String> {
    let stem = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let ext = p.extension().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default();
    if ext != "asm" && ext != "inc" {
        return Err(t("IDE.InvalidFileExtension"));
    }
    const BAD: &str = " \"*+,/:;<=>?[\\]|.";
    if stem.is_empty() || stem.chars().any(|c| BAD.contains(c) || !c.is_ascii() || c.is_control()) {
        return Err(format!("{} {}", t("IDE.InvalidFilenameCharacter"), "\" * + , / : ; < = > ? [ \\ ] | ."));
    }
    if stem.chars().count() > 8 {
        return Err(t("IDE.InvalidFilenameLength"));
    }
    Ok(())
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings = Settings::load();
        let locales_dir = config::find_locales_dir();
        let i18n = Arc::new(I18n::load(locales_dir.as_deref(), &settings.language));
        cc.egui_ctx.options_mut(|o| o.zoom_with_keyboard = false);
        theme::apply(&cc.egui_ctx, settings.ide_theme);

        let mut s = State {
            dosbox: dosbox::detect(&settings.dosbox_path),
            bin_dir: config::find_bin_dir(&settings.bin_dir),
            settings,
            i18n,
            docs: HashMap::new(),
            next_id: 1,
            untitled_counter: 0,
            active_doc: None,
            logs: Vec::new(),
            log_scroll: true,
            terminal: TerminalState { lines: Vec::new(), input: String::new(), history_pos: None, focus: false },
            task: None,
            find: FindState::default(),
            confirm: None,
            message: None,
            prefs: None,
            prefs_tab: 0,
            about: false,
            help: false,
            goto: None,
            properties: None,
            status: String::new(),
            pending: Vec::new(),
            quit_confirmed: false,
            open_tabs: Vec::new(),
            close_tabs: Vec::new(),
            focus_tab: None,
            theme_applied: None,
        };
        s.status = s.t("Port.Ready");
        let welcome = s.t("IDE.TerminalWelcomeMsg");
        for l in welcome.lines() {
            s.terminal.lines.push((LogKind::Info, l.to_string()));
        }
        s.log(LogKind::Info, format!("{} {VERSION}", APP_NAME));
        match &s.dosbox {
            Some(d) => {
                let msg = format!("{}: {} ({})", s.t("Port.DosboxFound"), d.flavor.label(), d.exe.display());
                s.log(LogKind::Info, msg);
            }
            None => {
                let msg = s.t("Port.DosboxMissing");
                s.log(LogKind::Warning, msg);
            }
        }
        match &s.bin_dir {
            Some(b) => {
                let msg = format!("{}: {}", s.t("Port.BinFound"), b.display());
                s.log(LogKind::Info, msg);
            }
            None => {
                let msg = s.t("Port.BinMissing");
                s.log(LogKind::Warning, msg);
            }
        }

        // Restaurar sesión.
        let mut restored_ids: Vec<Option<u64>> = Vec::new();
        if s.settings.restore_session {
            let files = s.settings.session_files.clone();
            for p in &files {
                restored_ids.push(s.open_path(p, false));
            }
            if files.iter().any(|p| p.exists()) {
                let msg = s.t("IDE.SuccessullyRestoredLastSession");
                s.log(LogKind::Success, msg);
            }
        }
        // CLI: archivos pasados como argumentos.
        let cli: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();
        let mut cli_ids = Vec::new();
        for p in cli {
            if let Some(id) = s.open_path(&p, true) {
                cli_ids.push(id);
            }
        }

        let dock = Self::restore_layout(&s, &restored_ids).unwrap_or_else(|| {
            let mut editors: Vec<Tab> = restored_ids.iter().flatten().map(|id| Tab::Editor(*id)).collect();
            editors.extend(cli_ids.iter().map(|id| Tab::Editor(*id)));
            let mut d = DockState::new(editors);
            let mut bottom = Vec::new();
            if s.settings.show_output {
                bottom.push(Tab::Output);
            }
            if s.settings.show_terminal {
                bottom.push(Tab::Terminal);
            }
            if !bottom.is_empty() {
                d.main_surface_mut().split_below(NodeIndex::root(), 0.72, bottom);
            }
            d
        });
        let mut app = Self { dock, s, devshot: crate::devshot::DevShot::from_env() };
        if app.devshot.active() {
            match std::env::var("RTASM_SHOT_UI").unwrap_or_default().as_str() {
                "prefs0" => { app.s.prefs = Some(app.s.settings.clone()); app.s.prefs_tab = 0; }
                "prefs1" => { app.s.prefs = Some(app.s.settings.clone()); app.s.prefs_tab = 1; }
                "prefs2" => { app.s.prefs = Some(app.s.settings.clone()); app.s.prefs_tab = 2; }
                "help" => app.s.help = true,
                "about" => app.s.about = true,
                "find" => { app.s.find.open = true; app.s.find.replace = true; app.s.find.needle = "mov".into(); }
                "nodosbox" => app.s.message = Some((app.s.t("Port.DosboxMissingTitle"), app.s.t("Port.DosboxMissingHelp"))),
                _ => {}
            }
        }
        // Editores abiertos por CLI que no estén en el diseño restaurado.
        for id in cli_ids {
            app.s.open_tabs.push(Tab::Editor(id));
        }
        if let Some(active) = app.s.settings.session_active.clone() {
            if let Some((id, _)) = app.s.docs.iter().find(|(_, d)| d.path.as_ref() == Some(&active)) {
                app.s.focus_tab = Some(Tab::Editor(*id));
            }
        }
        if app.s.docs.is_empty() {
            app.s.new_doc(String::new());
        }
        app
    }

    fn layout_file() -> PathBuf {
        config::config_dir().join("layout.json")
    }

    fn restore_layout(s: &State, ids: &[Option<u64>]) -> Option<DockState<Tab>> {
        if !s.settings.restore_session {
            return None;
        }
        let text = std::fs::read_to_string(Self::layout_file()).ok()?;
        let saved: DockState<SavedTab> = serde_json::from_str(&text).ok()?;
        let d = saved.filter_map_tabs(|t| match *t {
            SavedTab::Editor(i) => ids.get(i).copied().flatten().map(Tab::Editor),
            SavedTab::Output => Some(Tab::Output),
            SavedTab::Terminal => Some(Tab::Terminal),
        });
        if d.iter_all_tabs().next().is_none() {
            return None;
        }
        Some(d)
    }

    fn save_state(&mut self) {
        // Orden de las pestañas de editor para la sesión.
        let mut files: Vec<PathBuf> = Vec::new();
        for (_, tab) in self.dock.iter_all_tabs() {
            if let Tab::Editor(id) = tab {
                if let Some(p) = self.s.docs.get(id).and_then(|d| d.path.clone()) {
                    if !files.contains(&p) {
                        files.push(p);
                    }
                }
            }
        }
        let docs = &self.s.docs;
        let saved = self.dock.filter_map_tabs(|t| match *t {
            Tab::Editor(id) => {
                let p = docs.get(&id)?.path.clone()?;
                files.iter().position(|f| *f == p).map(SavedTab::Editor)
            }
            Tab::Output => Some(SavedTab::Output),
            Tab::Terminal => Some(SavedTab::Terminal),
        });
        if let Ok(j) = serde_json::to_string(&saved) {
            let _ = std::fs::create_dir_all(config::config_dir());
            let _ = std::fs::write(Self::layout_file(), j);
        }
        self.s.settings.session_active = self.s.active_doc.and_then(|id| self.s.docs.get(&id)).and_then(|d| d.path.clone());
        self.s.settings.session_files = files;
        self.s.settings.save();
    }

    fn apply_tab_requests(&mut self) {
        for tab in std::mem::take(&mut self.s.close_tabs) {
            if let Some(path) = self.dock.find_tab(&tab) {
                self.dock.remove_tab(path);
            }
            if let Tab::Editor(id) = tab {
                self.s.docs.remove(&id);
                if self.s.active_doc == Some(id) {
                    self.s.active_doc = None;
                }
            }
        }
        for tab in std::mem::take(&mut self.s.open_tabs) {
            if self.dock.find_tab(&tab).is_some() {
                self.s.focus_tab = Some(tab);
                continue;
            }
            match tab {
                Tab::Editor(_) => {
                    // Añadir a la zona de editores; si no existe, crearla arriba.
                    // Zona de editores existente o, si quedó vacía, esa misma hoja.
                    let leaf = self
                        .dock
                        .iter_leaves()
                        .find(|(_, l)| l.tabs().iter().any(|t| matches!(t, Tab::Editor(_))))
                        .or_else(|| self.dock.iter_leaves().find(|(_, l)| l.is_empty()))
                        .map(|(p, _)| p);
                    match leaf {
                        Some(p) => {
                            if let Ok(l) = self.dock.leaf_mut(p) {
                                l.append_tab(tab);
                            }
                        }
                        None => {
                            let empty = self.dock.iter_all_tabs().next().is_none();
                            if empty {
                                self.dock = DockState::new(vec![tab]);
                            } else {
                                self.dock.main_surface_mut().split_above(NodeIndex::root(), 0.72, vec![tab]);
                            }
                        }
                    }
                }
                Tab::Output | Tab::Terminal => {
                    let other = if tab == Tab::Output { Tab::Terminal } else { Tab::Output };
                    if let Some(p) = self.dock.find_tab(&other) {
                        if let Ok(l) = self.dock.leaf_mut(p.node_path()) {
                            l.append_tab(tab);
                        }
                    } else if self.dock.iter_all_tabs().next().is_none() {
                        self.dock = DockState::new(vec![tab]);
                    } else {
                        self.dock.main_surface_mut().split_below(NodeIndex::root(), 0.72, vec![tab]);
                    }
                }
            }
            self.s.focus_tab = Some(tab);
        }
        if let Some(tab) = self.s.focus_tab.take() {
            if let Some(p) = self.dock.find_tab(&tab) {
                let _ = self.dock.set_active_tab(p);
                self.dock.set_focused_node_and_surface(p.node_path());
                if let Tab::Editor(id) = tab {
                    self.s.active_doc = Some(id);
                    if let Some(d) = self.s.docs.get_mut(&id) {
                        d.focus = true;
                    }
                }
            }
        }
        // Editor activo = el de la hoja enfocada (o el primero visible).
        if let Some((_, Tab::Editor(id))) = self.dock.find_active_focused() {
            self.s.active_doc = Some(*id);
        }
        if self.s.active_doc.map(|id| !self.s.docs.contains_key(&id)).unwrap_or(true) {
            self.s.active_doc = self.dock.iter_all_tabs().find_map(|(_, t)| if let Tab::Editor(id) = t { Some(*id) } else { None });
        }
        self.s.settings.show_output = self.dock.find_tab(&Tab::Output).is_some();
        self.s.settings.show_terminal = self.dock.find_tab(&Tab::Terminal).is_some();
    }
}

impl State {
    fn t(&self, key: &str) -> String {
        self.i18n.t(key)
    }

    fn log(&mut self, kind: LogKind, text: String) {
        let jump = if kind == LogKind::Tool {
            toolchain::parse_tasm(&text).into_iter().find(|d| d.line > 0).and_then(|d| {
                self.task_source_dir().map(|dir| (dir.join(&d.file), d.line))
            })
        } else {
            None
        };
        self.logs.push(LogLine { time: now_hms(), kind, text, jump });
        if self.logs.len() > 5000 {
            self.logs.drain(..1000);
        }
        self.log_scroll = true;
    }

    fn task_source_dir(&self) -> Option<PathBuf> {
        self.active_doc.and_then(|id| self.docs.get(&id)).and_then(|d| d.path.as_ref()).and_then(|p| p.parent().map(Path::to_path_buf))
    }

    fn untitled(&self) -> String {
        self.t("Port.Untitled")
    }

    fn new_doc(&mut self, text: String) -> u64 {
        self.untitled_counter += 1;
        let id = self.next_id;
        self.next_id += 1;
        let mut d = Document::new(id, self.untitled_counter, None, text.clone());
        if !text.is_empty() {
            d.saved_hash = editor::hash_text("");
        }
        self.docs.insert(id, d);
        self.open_tabs.push(Tab::Editor(id));
        id
    }

    fn open_path(&mut self, p: &Path, validate: bool) -> Option<u64> {
        let p = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
        if let Some((id, _)) = self.docs.iter().find(|(_, d)| d.path.as_ref() == Some(&p)) {
            let id = *id;
            self.focus_tab = Some(Tab::Editor(id));
            if validate {
                self.status = self.t("IDE.FileAlreadyOpenedMsg");
            }
            return Some(id);
        }
        if validate {
            let i18n = self.i18n.clone();
            if let Err(e) = validate_filename(&p, &|k| i18n.t(k)) {
                self.message = Some((self.t("IDE.InvalidFileTitle"), format!("{e}\n\n{}", p.display())));
                return None;
            }
        }
        let bytes = match std::fs::read(&p) {
            Ok(b) => b,
            Err(e) => {
                if validate {
                    self.message = Some((self.t("IDE.OpenError"), format!("{}\n{e}", p.display())));
                }
                return None;
            }
        };
        // Los archivos de DOS suelen venir en CP437/ASCII; si no es UTF-8, se decodifica.
        let text = match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => dosbox::decode_cp437(e.as_bytes()),
        };
        let id = self.next_id;
        self.next_id += 1;
        self.docs.insert(id, Document::new(id, 0, Some(p.clone()), text));
        self.open_tabs.push(Tab::Editor(id));
        self.settings.push_recent(&p);
        let msg = format!("{} {}", self.t("IDE.OpeningFile"), p.display());
        self.status = msg.clone();
        self.log(LogKind::Info, msg);
        Some(id)
    }

    fn open_dialog(&mut self) {
        let mut dlg = rfd::FileDialog::new().add_filter("Assembly (*.asm, *.inc)", &["asm", "ASM", "inc", "INC"]);
        if let Some(dir) = self.task_source_dir() {
            dlg = dlg.set_directory(dir);
        }
        if let Some(files) = dlg.pick_files() {
            for f in files {
                self.open_path(&f, true);
            }
        }
    }

    /// Guarda el documento; `as_new` fuerza el diálogo "Guardar como".
    fn save_doc(&mut self, id: u64, as_new: bool) -> bool {
        let Some(doc) = self.docs.get(&id) else { return false };
        let target = if as_new || doc.path.is_none() {
            let mut dlg = rfd::FileDialog::new().add_filter("Assembly (*.asm)", &["asm", "ASM", "inc", "INC"]);
            let default_name = doc
                .path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().into_owned())
                .or_else(|| EXAMPLE_NAMES.with(|m| m.borrow().get(&id).cloned()))
                .unwrap_or_else(|| "MAIN.ASM".into());
            dlg = dlg.set_file_name(default_name);
            if let Some(dir) = doc.path.as_ref().and_then(|p| p.parent()).map(Path::to_path_buf).or_else(|| self.task_source_dir()) {
                dlg = dlg.set_directory(dir);
            }
            let Some(mut p) = dlg.save_file() else { return false };
            if p.extension().is_none() {
                p.set_extension("ASM");
            }
            let i18n = self.i18n.clone();
            if let Err(e) = validate_filename(&p, &|k| i18n.t(k)) {
                self.message = Some((self.t("IDE.InvalidFileTitle"), e));
                return false;
            }
            p
        } else {
            doc.path.clone().unwrap()
        };
        let text = self.docs[&id].text.clone();
        // DOS necesita CRLF.
        let data = text.replace('\n', "\r\n");
        match std::fs::write(&target, data) {
            Ok(()) => {
                let target = target.canonicalize().unwrap_or(target);
                let msg_key = if as_new { "IDE.SavingFileAs" } else { "IDE.SavingFile" };
                let msg = format!("{} {}", self.t(msg_key), target.display());
                let d = self.docs.get_mut(&id).unwrap();
                d.saved_hash = editor::hash_text(&d.text);
                d.path = Some(target.clone());
                self.settings.push_recent(&target);
                self.status = msg.clone();
                self.log(LogKind::Info, msg);
                true
            }
            Err(e) => {
                self.message = Some((self.t("IDE.MenuSave"), format!("{}\n{e}", target.display())));
                false
            }
        }
    }

    fn request_close(&mut self, ids: Vec<u64>) {
        let dirty: Vec<u64> = ids.iter().copied().filter(|id| self.docs.get(id).map(|d| d.is_dirty()).unwrap_or(false)).collect();
        for id in ids.iter().filter(|id| !dirty.contains(id)) {
            self.close_tabs.push(Tab::Editor(*id));
        }
        for id in dirty {
            let title = self.docs[&id].title(&self.untitled());
            self.pending.push(Flow::None);
            self.confirm_push(Confirm {
                title: format!("{} — {title}", self.t("IDE.UnsavedTabTitle")),
                message: self.t("IDE.SaveTabPromptMsg"),
                yes: Flow::SaveAndCloseDoc(id),
                no: Flow::CloseDoc(id),
                cancel: true,
            });
        }
    }

    fn confirm_push(&mut self, c: Confirm) {
        // Un diálogo a la vez; los demás esperan en cola.
        if self.confirm.is_none() {
            self.confirm = Some(c);
        } else {
            CONFIRM_QUEUE.with(|q| q.borrow_mut().push(c));
        }
    }

    fn file_newer(a: &Path, b: &Path) -> bool {
        match (std::fs::metadata(a).and_then(|m| m.modified()), std::fs::metadata(b).and_then(|m| m.modified())) {
            (Ok(x), Ok(y)) => x >= y,
            _ => false,
        }
    }

    fn doc_assembled(&self, d: &Document) -> bool {
        if d.is_assembled() {
            return true;
        }
        let Some(p) = &d.path else { return false };
        let dir = p.parent().unwrap_or(Path::new("."));
        !d.is_dirty()
            && d.assembled_hash.is_none()
            && toolchain::find_ci(dir, &format!("{}.OBJ", toolchain::dos_stem(p))).map(|o| Self::file_newer(&o, p)).unwrap_or(false)
    }

    fn doc_built(&self, d: &Document) -> bool {
        if d.is_built() {
            return true;
        }
        let Some(p) = &d.path else { return false };
        let dir = p.parent().unwrap_or(Path::new("."));
        !d.is_dirty()
            && d.built_hash.is_none()
            && d.assembled_hash.is_none()
            && toolchain::find_ci(dir, &format!("{}.EXE", toolchain::dos_stem(p))).map(|o| Self::file_newer(&o, p)).unwrap_or(false)
    }

    /// Punto de entrada de Ensamblar/Construir/Ejecutar/Depurar con las mismas
    /// preguntas que el IDE original.
    fn action(&mut self, action: Action, id: Option<u64>) {
        let Some(id) = id.or(self.active_doc) else { return };
        if self.task.is_some() {
            self.status = self.t("Port.Busy");
            return;
        }
        let Some(doc) = self.docs.get(&id) else { return };
        let skipped_key = match action {
            Action::Assemble => "IDE.AssembleSkipped",
            Action::Build | Action::AssembleBuild => "IDE.BuildSkipped",
            Action::Run | Action::AssembleBuildRun => "IDE.RunSkipped",
            Action::Debug => "IDE.DebugSkipped",
        };
        if doc.path.is_none() || doc.is_dirty() {
            self.confirm_push(Confirm {
                title: self.t("IDE.UnsavedFileTitle"),
                message: self.t("IDE.SaveFileAndProceedPromptMsg"),
                yes: Flow::SaveThen(Box::new(Flow::Proceed(action, id)), id),
                no: Flow::Log(LogKind::Warning, self.t(skipped_key)),
                cancel: false,
            });
            return;
        }
        let assembled = self.doc_assembled(doc);
        let built = self.doc_built(doc);
        let steps: Vec<Step> = match action {
            Action::Assemble => vec![Step::Assemble],
            Action::AssembleBuild => vec![Step::Assemble, Step::Build],
            Action::AssembleBuildRun => vec![Step::Assemble, Step::Build, Step::Run],
            Action::Build => {
                if !assembled {
                    self.confirm_push(Confirm {
                        title: self.t("IDE.UnAssembledCode"),
                        message: self.t("IDE.AssembleFileAndProceedPromptMsg"),
                        yes: Flow::Steps(vec![Step::Assemble, Step::Build], id),
                        no: Flow::Log(LogKind::Warning, self.t(skipped_key)),
                        cancel: false,
                    });
                    return;
                }
                vec![Step::Build]
            }
            Action::Run | Action::Debug => {
                let last = if action == Action::Run { Step::Run } else { Step::Debug };
                if !built {
                    let mut steps = vec![];
                    if !assembled {
                        steps.push(Step::Assemble);
                    }
                    steps.push(Step::Build);
                    steps.push(last);
                    self.confirm_push(Confirm {
                        title: self.t("IDE.UnBuiltCode"),
                        message: self.t("IDE.BuildFileAndProceedPromptMsg"),
                        yes: Flow::Steps(steps, id),
                        no: Flow::Log(LogKind::Warning, self.t(skipped_key)),
                        cancel: false,
                    });
                    return;
                }
                vec![last]
            }
        };
        self.run_steps(steps, id);
    }

    fn check_backend(&mut self) -> Option<(DosBox, PathBuf)> {
        if self.dosbox.is_none() {
            self.dosbox = dosbox::detect(&self.settings.dosbox_path);
        }
        if self.bin_dir.is_none() {
            self.bin_dir = config::find_bin_dir(&self.settings.bin_dir);
        }
        match (&self.dosbox, &self.bin_dir) {
            (Some(d), Some(b)) => Some((d.clone(), b.clone())),
            (None, _) => {
                self.message = Some((self.t("Port.DosboxMissingTitle"), self.t("Port.DosboxMissingHelp")));
                None
            }
            (_, None) => {
                self.message = Some((self.t("Others.MissingFileTitle"), self.t("Port.BinMissing")));
                None
            }
        }
    }

    fn run_steps(&mut self, steps: Vec<Step>, id: u64) {
        let Some((dosbox, bin_dir)) = self.check_backend() else { return };
        let Some(doc) = self.docs.get_mut(&id) else { return };
        let Some(source) = doc.path.clone() else { return };
        if steps.contains(&Step::Assemble) {
            doc.diagnostics.clear();
        }
        let req = toolchain::Request {
            steps: steps.clone(),
            source,
            text_hash: editor::hash_text(&doc.text),
            tasm_args: self.settings.tasm_args.clone(),
            tlink_args: self.settings.tlink_args.clone(),
            use_launcher: self.settings.use_launcher,
            silent: self.settings.silent_tools,
            bin_dir,
            dosbox,
            launcher_msg: self.t("Port.LauncherMsg"),
        };
        let label = match steps.last() {
            Some(Step::Assemble) => self.t("IDE.MenuAssemble"),
            Some(Step::Build) => self.t("IDE.MenuBuild"),
            Some(Step::Run) => self.t("IDE.MenuRun"),
            Some(Step::Debug) => self.t("IDE.MenuDebug"),
            None => return,
        };
        let (tx, rx) = channel();
        let cancel = CancelHandle::default();
        let c2 = cancel.clone();
        let i18n = self.i18n.clone();
        std::thread::spawn(move || toolchain::execute(req, tx, c2, move |k| i18n.t(k)));
        self.status = format!("{label}…");
        self.task = Some(TaskState { rx, cancel, label, started: Instant::now(), terminal: false });
    }

    fn run_terminal(&mut self, command: String) {
        let cmd = command.trim().to_string();
        if cmd.is_empty() {
            return;
        }
        self.settings.terminal_history.retain(|c| *c != cmd);
        self.settings.terminal_history.push(cmd.clone());
        if self.settings.terminal_history.len() > 100 {
            self.settings.terminal_history.remove(0);
        }
        self.terminal.history_pos = None;
        let work_dir = self.task_source_dir().or_else(|| self.bin_dir.clone()).unwrap_or_else(|| ".".into());
        let prompt = format!("{}> {cmd}", work_dir.file_name().map(|s| s.to_string_lossy().to_uppercase()).unwrap_or_default());
        self.terminal.lines.push((LogKind::Command, prompt));
        match cmd.to_uppercase().as_str() {
            "CLS" | "CLEAR" => {
                self.terminal.lines.clear();
                return;
            }
            "HELP" | "AYUDA" => {
                let h = self.t("Port.TerminalHelp");
                for l in h.lines() {
                    self.terminal.lines.push((LogKind::Info, l.to_string()));
                }
                return;
            }
            _ => {}
        }
        if self.task.is_some() {
            let b = self.t("Port.Busy");
            self.terminal.lines.push((LogKind::Warning, b));
            return;
        }
        let Some((dosbox, bin_dir)) = self.check_backend() else { return };
        let req = toolchain::TerminalRequest {
            command: cmd.clone(),
            work_dir,
            bin_dir,
            dosbox,
            silent: self.settings.silent_tools,
            launcher_msg: self.t("Port.LauncherMsg"),
        };
        let (tx, rx) = channel();
        let cancel = CancelHandle::default();
        let c2 = cancel.clone();
        std::thread::spawn(move || toolchain::execute_terminal(req, tx, c2));
        self.status = format!("{} {cmd}", self.t("IDE.ExcutingCmd"));
        self.task = Some(TaskState { rx, cancel, label: cmd, started: Instant::now(), terminal: true });
    }

    fn poll_task(&mut self, ctx: &egui::Context) {
        let Some(task) = &self.task else { return };
        let mut events = Vec::new();
        while let Ok(e) = task.rx.try_recv() {
            events.push(e);
        }
        let is_terminal = task.terminal;
        for e in events {
            match e {
                Event::Log(k, s) => self.log(k, s),
                Event::TerminalOutput(s) => {
                    for l in s.lines() {
                        self.terminal.lines.push((LogKind::Tool, l.to_string()));
                    }
                }
                Event::Diagnostics { source, diags } => {
                    if let Some(d) = self.docs.values_mut().find(|d| d.path.as_ref() == Some(&source)) {
                        d.diagnostics = diags;
                        if let Some(first) = d.diagnostics.iter().find(|x| x.line > 0 && x.severity == toolchain::Severity::Error) {
                            d.goto_line = Some(first.line);
                        }
                    }
                }
                Event::StepDone { step, outcome, source, text_hash } => {
                    let ok = matches!(outcome, toolchain::Outcome::Ok | toolchain::Outcome::OkWithWarnings);
                    if let Some(d) = self.docs.values_mut().find(|d| d.path.as_ref() == Some(&source)) {
                        match step {
                            Step::Assemble => {
                                d.assembled_hash = ok.then_some(text_hash);
                                if !ok {
                                    d.built_hash = None;
                                }
                            }
                            Step::Build => d.built_hash = ok.then_some(text_hash),
                            _ => {}
                        }
                    }
                    if !ok && step != Step::Run && step != Step::Debug {
                        self.open_tabs.push(Tab::Output);
                    }
                }
                Event::SilentUnsupported => {
                    if self.settings.silent_tools {
                        self.settings.silent_tools = false;
                        let m = self.t("Port.SilentDisabled");
                        self.log(LogKind::Warning, m);
                    }
                }
                Event::Finished => {
                    let elapsed = self.task.as_ref().map(|t| t.started.elapsed().as_secs_f32()).unwrap_or(0.0);
                    self.task = None;
                    self.status = format!("{} ({elapsed:.1} s)", self.t("Port.Ready"));
                    if is_terminal {
                        self.terminal.focus = true;
                    }
                    break;
                }
            }
        }
        if self.task.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }

    fn run_flow(&mut self, flow: Flow) {
        match flow {
            Flow::None => {}
            Flow::Proceed(a, id) => self.action(a, Some(id)),
            Flow::Steps(s, id) => self.run_steps(s, id),
            Flow::SaveThen(next, id) => {
                if self.save_doc(id, false) {
                    self.run_flow(*next);
                }
            }
            Flow::Log(k, s) => {
                self.status = s.clone();
                self.log(k, s);
            }
            Flow::CloseDoc(id) => self.close_tabs.push(Tab::Editor(id)),
            Flow::SaveAndCloseDoc(id) => {
                if self.save_doc(id, false) {
                    self.close_tabs.push(Tab::Editor(id));
                }
            }
            Flow::Quit => {
                self.quit_confirmed = true;
            }
            Flow::SaveAllAndQuit => {
                let ids: Vec<u64> = self.docs.iter().filter(|(_, d)| d.is_dirty()).map(|(id, _)| *id).collect();
                if ids.into_iter().all(|id| self.save_doc(id, false)) {
                    self.quit_confirmed = true;
                }
            }
            Flow::RemoveRecent(p) => self.settings.recent_files.retain(|x| *x != p),
            Flow::ClearRecent => self.settings.recent_files.clear(),
        }
    }

    fn print_doc(&mut self, id: u64) {
        let Some(d) = self.docs.get(&id) else { return };
        let title = d.title(&self.untitled());
        let mut html = format!(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>{title}</title><style>body{{font:12px 'DejaVu Sans Mono',monospace;margin:24px}}h1{{font:600 15px sans-serif}}td.n{{color:#999;text-align:right;padding-right:12px;user-select:none;vertical-align:top}}pre{{margin:0;white-space:pre-wrap}}</style></head><body onload=\"window.print()\"><h1>{title}</h1><table>"
        );
        for (i, l) in d.text.lines().enumerate() {
            let esc = l.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('\t', "        ");
            html.push_str(&format!("<tr><td class=n>{}</td><td><pre>{esc}</pre></td></tr>", i + 1));
        }
        html.push_str("</table></body></html>");
        let p = std::env::temp_dir().join(format!("rust-tasm-print-{id}.html"));
        if std::fs::write(&p, html).is_ok() {
            let _ = open::that_detached(&p);
            let m = self.t("Port.PrintNote");
            self.status = m;
        }
    }
}

thread_local! {
    static CONFIRM_QUEUE: std::cell::RefCell<Vec<Confirm>> = const { std::cell::RefCell::new(Vec::new()) };
}

// ---------------------------------------------------------------------------
// Pestañas
// ---------------------------------------------------------------------------

struct Viewer<'a> {
    s: &'a mut State,
}

impl TabViewer for Viewer<'_> {
    type Tab = Tab;

    fn id(&mut self, tab: &mut Tab) -> egui::Id {
        egui::Id::new(("tab", format!("{tab:?}")))
    }

    fn title(&mut self, tab: &mut Tab) -> egui::WidgetText {
        match tab {
            Tab::Editor(id) => match self.s.docs.get(id) {
                Some(d) => {
                    let mut t = d.title(&self.s.untitled());
                    if d.is_dirty() {
                        t.push_str(" ●");
                    }
                    let errors = d.diagnostics.iter().filter(|x| x.severity == toolchain::Severity::Error).count();
                    if errors > 0 {
                        t = format!("{t}  ⚠{errors}");
                    }
                    t.into()
                }
                None => "?".into(),
            },
            Tab::Output => format!("📄 {}", self.s.t("IDE.OutputWindowTitle")).into(),
            Tab::Terminal => format!("⌨ {}", self.s.t("IDE.TerminalWindowTitle")).into(),
        }
    }

    fn on_tab_button(&mut self, tab: &mut Tab, response: &egui::Response) {
        if let Tab::Editor(id) = tab {
            if let Some(p) = self.s.docs.get(id).and_then(|d| d.path.as_ref()) {
                response.clone().on_hover_text(p.display().to_string());
            }
            if response.clicked() {
                self.s.active_doc = Some(*id);
            }
        }
    }

    fn context_menu(&mut self, ui: &mut egui::Ui, tab: &mut Tab, _path: egui_dock::NodePath) {
        if let Tab::Editor(id) = *tab {
            let s = &mut *self.s;
            if ui.button(s.t("IDE.MenuSave")).clicked() {
                s.save_doc(id, false);
                ui.close();
            }
            if ui.button(s.t("IDE.MenuCloseOthers")).clicked() {
                let others: Vec<u64> = s.docs.keys().copied().filter(|x| *x != id).collect();
                s.request_close(others);
                ui.close();
            }
            if ui.button(s.t("IDE.MenuOpenContainingFolder")).clicked() {
                if let Some(dir) = s.docs.get(&id).and_then(|d| d.path.as_ref()).and_then(|p| p.parent()) {
                    let _ = open::that_detached(dir);
                }
                ui.close();
            }
        }
    }

    fn on_close(&mut self, tab: &mut Tab) -> OnCloseResponse {
        match *tab {
            Tab::Editor(id) => {
                if self.s.docs.get(&id).map(|d| d.is_dirty()).unwrap_or(false) {
                    self.s.request_close(vec![id]);
                    OnCloseResponse::Ignore
                } else {
                    self.s.docs.remove(&id);
                    OnCloseResponse::Close
                }
            }
            _ => OnCloseResponse::Close,
        }
    }

    fn scroll_bars(&self, _tab: &Tab) -> [bool; 2] {
        [false, false]
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {
        match *tab {
            Tab::Editor(id) => self.editor_tab(ui, id),
            Tab::Output => self.output_tab(ui),
            Tab::Terminal => self.terminal_tab(ui),
        }
    }
}

impl Viewer<'_> {
    fn editor_tab(&mut self, ui: &mut egui::Ui, id: u64) {
        let s = &mut *self.s;
        let pal = if s.settings.editor_dark { Palette::dark() } else { Palette::light() };
        let is_active = s.active_doc == Some(id);
        if is_active && s.find.open {
            find_bar(ui, s, id);
        }
        let style = EditorStyle {
            palette: pal,
            font_size: s.settings.font_size,
            word_wrap: s.settings.word_wrap,
            auto_indent: s.settings.auto_indent,
        };
        let needle = s.find.needle.clone();
        let fq = (is_active && s.find.open && !needle.is_empty()).then(|| FindQuery {
            needle: &needle,
            match_case: s.find.match_case,
            whole_word: s.find.whole_word,
        });
        let Some(doc) = s.docs.get_mut(&id) else { return };
        egui::Frame::new().fill(pal.background).show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            ui.visuals_mut().selection.bg_fill = pal.selection;
            ui.visuals_mut().extreme_bg_color = pal.background;
            ui.visuals_mut().override_text_color = Some(pal.text);
            let resp_changed = editor::show(ui, doc, &style, fq);
            if resp_changed && !doc.diagnostics.is_empty() {
                // Las marcas viejas siguen visibles hasta el siguiente ensamblado,
                // igual que en el original.
            }
        });
        if ui.ui_contains_pointer() && ui.input(|i| i.pointer.any_pressed()) {
            s.active_doc = Some(id);
        }
        // Zoom con Ctrl + rueda.
        if ui.ui_contains_pointer() {
            // Con Ctrl, egui convierte la rueda en "zoom".
            let zoom = ui.input(|i| i.zoom_delta());
            if zoom != 1.0 {
                s.settings.font_size = (s.settings.font_size * zoom).round().clamp(8.0, 40.0);
            }
        }
    }

    fn output_tab(&mut self, ui: &mut egui::Ui) {
        let s = &mut *self.s;
        let c = theme::colors(s.settings.ide_theme);
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{}:", s.t("IDE.OutputFrom"))).color(c.dim));
            ui.label(RichText::new("TASM · TLINK · TD").strong());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.small_button(format!("🗑 {}", s.t("IDE.MenuClearLogConsole"))).clicked() {
                    s.logs.clear();
                }
                if ui.small_button("📋").on_hover_text(s.t("IDE.MenuBarCopyText")).clicked() {
                    let all: Vec<String> = s.logs.iter().map(|l| format!("[{}] {}", l.time, l.text)).collect();
                    ui.ctx().copy_text(all.join("\n"));
                }
                let tip = s.t("Preference.EnableAutoScrollInLogConsole");
                ui.toggle_value(&mut s.settings.auto_scroll, "⬇ Auto").on_hover_text(tip);
            });
        });
        ui.separator();
        let mut jump: Option<(PathBuf, usize)> = None;
        egui::Frame::new().fill(c.log_bg).inner_margin(6.0).show(ui, |ui| {
            let scroll_bottom = s.settings.auto_scroll && s.log_scroll;
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .stick_to_bottom(s.settings.auto_scroll)
                .show(ui, |ui| {
                    ui.style_mut().override_text_style = Some(TextStyle::Monospace);
                    for l in &s.logs {
                        let color = match l.kind {
                            LogKind::Info => c.log_fg,
                            LogKind::Success => c.success,
                            LogKind::Warning => c.warning,
                            LogKind::Error => c.error,
                            LogKind::Tool => c.tool,
                            LogKind::Command => c.accent,
                        };
                        let icon = match l.kind {
                            LogKind::Success => "✔ ",
                            LogKind::Warning => "⚠ ",
                            LogKind::Error => "✖ ",
                            LogKind::Tool => "  ",
                            _ => "• ",
                        };
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&l.time).color(c.dim));
                            let text = RichText::new(format!("{icon}{}", l.text)).color(color);
                            if let Some(j) = &l.jump {
                                let r = ui.add(egui::Label::new(text.underline()).sense(egui::Sense::click()));
                                if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                    jump = Some(j.clone());
                                }
                            } else {
                                ui.add(egui::Label::new(text).selectable(true));
                            }
                        });
                    }
                    if scroll_bottom {
                        ui.scroll_to_cursor(Some(Align::BOTTOM));
                    }
                });
        });
        s.log_scroll = false;
        if let Some((p, line)) = jump {
            let pl = p.to_string_lossy().to_lowercase();
            let found = s
                .docs
                .iter()
                .find(|(_, d)| d.path.as_ref().map(|x| x.to_string_lossy().to_lowercase() == pl).unwrap_or(false))
                .map(|(id, _)| *id)
                .or_else(|| {
                    let dir = p.parent()?;
                    let name = p.file_name()?.to_string_lossy().into_owned();
                    let real = toolchain::find_ci(dir, &name)?;
                    s.open_path(&real, false)
                });
            if let Some(id) = found {
                if let Some(d) = s.docs.get_mut(&id) {
                    d.goto_line = Some(line);
                }
                s.focus_tab = Some(Tab::Editor(id));
            }
        }
    }

    fn terminal_tab(&mut self, ui: &mut egui::Ui) {
        let s = &mut *self.s;
        let c = theme::colors(s.settings.ide_theme);
        let input_h = 30.0;
        egui::Frame::new().fill(c.log_bg).inner_margin(6.0).show(ui, |ui| {
            let h = (ui.available_height() - input_h).max(40.0);
            egui::ScrollArea::both()
                .id_salt("term-scroll")
                .auto_shrink([false, false])
                .max_height(h)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.style_mut().override_text_style = Some(TextStyle::Monospace);
                    ui.spacing_mut().item_spacing.y = 1.0;
                    for (k, l) in &s.terminal.lines {
                        let color = match k {
                            LogKind::Command => c.accent,
                            LogKind::Warning => c.warning,
                            LogKind::Error => c.error,
                            LogKind::Info => c.dim,
                            _ => c.log_fg,
                        };
                        ui.add(egui::Label::new(RichText::new(l).color(color)).selectable(true).extend());
                    }
                });
            ui.separator();
            ui.horizontal(|ui| {
                let dir = s.task_source_dir().or_else(|| s.bin_dir.clone());
                let prompt = dir
                    .as_ref()
                    .and_then(|d| d.file_name())
                    .map(|n| format!("{}>", n.to_string_lossy().to_uppercase()))
                    .unwrap_or_else(|| "C:\\>".into());
                let r = ui
                    .label(RichText::new(prompt).monospace().color(c.accent).strong())
                    .on_hover_text(dir.map(|d| d.display().to_string()).unwrap_or_default());
                let _ = r;
                let id = egui::Id::new("terminal-input");
                let edit = egui::TextEdit::singleline(&mut s.terminal.input)
                    .id(id)
                    .font(TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .hint_text("TASM /?  ·  TLINK  ·  TD MAIN.EXE  ·  DIR  ·  HELP");
                let resp = ui.add(edit);
                if s.terminal.focus {
                    resp.request_focus();
                    s.terminal.focus = false;
                }
                if resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                    let cmd = std::mem::take(&mut s.terminal.input);
                    s.run_terminal(cmd);
                    s.terminal.focus = true;
                }
                if resp.has_focus() {
                    let hist = &s.settings.terminal_history;
                    if !hist.is_empty() {
                        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp)) {
                            let p = s.terminal.history_pos.map(|p| p.saturating_sub(1)).unwrap_or(hist.len() - 1);
                            s.terminal.history_pos = Some(p);
                            s.terminal.input = hist[p].clone();
                        }
                        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown)) {
                            match s.terminal.history_pos {
                                Some(p) if p + 1 < hist.len() => {
                                    s.terminal.history_pos = Some(p + 1);
                                    s.terminal.input = hist[p + 1].clone();
                                }
                                _ => {
                                    s.terminal.history_pos = None;
                                    s.terminal.input.clear();
                                }
                            }
                        }
                    }
                    if ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::L)) {
                        s.terminal.lines.clear();
                    }
                }
            });
        });
    }
}

fn find_bar(ui: &mut egui::Ui, s: &mut State, id: u64) {
    let mut find_next = false;
    let mut find_prev = false;
    let mut replace_one = false;
    let mut replace_all = false;
    egui::Frame::new().inner_margin(egui::Margin::symmetric(8, 5)).fill(ui.visuals().faint_bg_color).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("🔍");
            let hint = s.t("IDE.Find");
            let r = ui.add(egui::TextEdit::singleline(&mut s.find.needle).desired_width(220.0).hint_text(hint));
            if s.find.focus {
                r.request_focus();
                s.find.focus = false;
            }
            if r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                if ui.input(|i| i.modifiers.shift) {
                    find_prev = true;
                } else {
                    find_next = true;
                }
                r.request_focus();
            }
            if ui.button("⏶").on_hover_text("Shift+Enter").clicked() {
                find_prev = true;
            }
            if ui.button("⏷").on_hover_text(s.t("IDE.FindNext")).clicked() {
                find_next = true;
            }
            ui.toggle_value(&mut s.find.match_case, "Aa").on_hover_text(s.t("IDE.MatchCase"));
            ui.toggle_value(&mut s.find.whole_word, "\"ab\"").on_hover_text(s.t("IDE.MatchWholeWord"));
            ui.toggle_value(&mut s.find.replace, "🔁").on_hover_text(s.t("IDE.Replace"));
            ui.label(RichText::new(&s.find.status).weak());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("✖").clicked() {
                    s.find.open = false;
                    if let Some(d) = s.docs.get_mut(&id) {
                        d.focus = true;
                    }
                }
            });
        });
        if s.find.replace {
            ui.horizontal(|ui| {
                ui.label("🔁");
                let hint = s.t("IDE.Replace");
                ui.add(egui::TextEdit::singleline(&mut s.find.replacement).desired_width(220.0).hint_text(hint));
                if ui.button(s.t("IDE.Replace")).clicked() {
                    replace_one = true;
                }
                if ui.button(s.t("IDE.ReplaceAll")).clicked() {
                    replace_all = true;
                }
            });
        }
    });
    if ui.input(|i| i.key_pressed(Key::Escape)) {
        s.find.open = false;
        if let Some(d) = s.docs.get_mut(&id) {
            d.focus = true;
        }
    }
    let Some(doc) = s.docs.get_mut(&id) else { return };
    let ctx = ui.ctx().clone();
    let matches = crate::highlight::find_matches(&doc.text, &s.find.needle, s.find.match_case, s.find.whole_word);
    let to_chars = |b: usize| doc.text[..b].chars().count();
    let sel = egui::text_edit::TextEditState::load(&ctx, doc.text_id())
        .and_then(|st| st.cursor.char_range())
        .map(|r| {
            let [a, b] = r.sorted_cursors();
            (a.index.0, b.index.0)
        })
        .unwrap_or((0, 0));
    if replace_all && !matches.is_empty() {
        let n = matches.len();
        let mut out = String::with_capacity(doc.text.len());
        let mut last = 0;
        for (a, b) in &matches {
            out.push_str(&doc.text[last..*a]);
            out.push_str(&s.find.replacement);
            last = *b;
        }
        out.push_str(&doc.text[last..]);
        doc.select_range = Some((0, 0));
        doc.text = out;
        s.find.status = format!("{n} {}", s.t("IDE.OccuranceReplaced"));
        return;
    }
    if replace_one {
        // Si la selección es una coincidencia, se reemplaza y se pasa a la siguiente.
        if let Some(&(a, b)) = matches.iter().find(|(a, b)| to_chars(*a) == sel.0 && to_chars(*b) == sel.1) {
            let _ = (a, b);
            doc.replace_text = Some(s.find.replacement.clone());
            s.find.status = format!("1 {}", s.t("IDE.OccuranceReplaced"));
            return;
        }
        find_next = true;
    }
    if find_next || find_prev {
        if matches.is_empty() {
            s.find.status = s.t("IDE.NotFound");
            return;
        }
        let pos: Vec<(usize, usize)> = matches.iter().map(|(a, b)| (to_chars(*a), to_chars(*b))).collect();
        let idx = if find_next {
            pos.iter().position(|(a, _)| *a >= sel.1).unwrap_or(0)
        } else {
            pos.iter().rposition(|(_, b)| *b <= sel.0).unwrap_or(pos.len() - 1)
        };
        doc.select_range = Some((pos[idx].0, pos[idx].1));
        s.find.status = format!("{}/{}", idx + 1, pos.len());
        s.find.focus = true;
    } else if !s.find.needle.is_empty() {
        if matches.is_empty() {
            s.find.status = s.t("IDE.NotFound");
        } else if !s.find.status.contains('/') {
            s.find.status = format!("{} {}", matches.len(), s.t("Port.Matches"));
        }
    } else {
        s.find.status.clear();
    }
}

// ---------------------------------------------------------------------------
// Ventana principal
// ---------------------------------------------------------------------------

impl App {
    fn active_doc_id(&self) -> Option<u64> {
        self.s.active_doc
    }

    fn push_op(&mut self, op: EditOp) {
        if let Some(d) = self.s.active_doc.and_then(|id| self.s.docs.get_mut(&id)) {
            d.ops.push(op);
        }
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        let cmd = Modifiers::COMMAND;
        let sc = |m: Modifiers, k: Key| KeyboardShortcut::new(m, k);
        let hit = |s: KeyboardShortcut| ctx.input_mut(|i| i.consume_shortcut(&s));
        if hit(sc(cmd, Key::N)) || hit(sc(cmd, Key::T)) {
            self.s.new_doc(String::new());
        }
        if hit(sc(cmd, Key::O)) {
            self.s.open_dialog();
        }
        if hit(sc(cmd | Modifiers::SHIFT, Key::S)) {
            if let Some(id) = self.active_doc_id() {
                self.s.save_doc(id, true);
            }
        }
        if hit(sc(cmd, Key::S)) {
            if let Some(id) = self.active_doc_id() {
                self.s.save_doc(id, false);
            }
        }
        if hit(sc(cmd, Key::W)) {
            if let Some(id) = self.active_doc_id() {
                self.s.request_close(vec![id]);
            }
        }
        if hit(sc(cmd, Key::F)) {
            self.s.find.open = true;
            self.s.find.replace = false;
            self.s.find.focus = true;
            if let Some(d) = self.s.active_doc.and_then(|id| self.s.docs.get(&id)) {
                let sel = d.selected_text(ctx);
                if !sel.is_empty() && !sel.contains('\n') {
                    self.s.find.needle = sel;
                }
            }
        }
        if hit(sc(cmd, Key::H)) {
            self.s.find.open = true;
            self.s.find.replace = true;
            self.s.find.focus = true;
        }
        if hit(sc(cmd, Key::G)) {
            self.s.goto = Some(String::new());
        }
        if hit(sc(Modifiers::NONE, Key::F3)) {
            self.s.find.open = true;
        }
        if hit(sc(cmd, Key::Equals)) || hit(sc(cmd, Key::Plus)) {
            self.s.settings.font_size = (self.s.settings.font_size + 1.0).min(40.0);
        }
        if hit(sc(cmd, Key::Minus)) {
            self.s.settings.font_size = (self.s.settings.font_size - 1.0).max(8.0);
        }
        if hit(sc(cmd, Key::Num0)) {
            self.s.settings.font_size = Settings::default().font_size;
        }
        if hit(sc(Modifiers::ALT, Key::F9)) {
            self.s.action(Action::Assemble, None);
        }
        if hit(sc(Modifiers::CTRL, Key::F9)) {
            self.s.action(Action::AssembleBuildRun, None);
        }
        if hit(sc(Modifiers::SHIFT, Key::F9)) {
            self.s.action(Action::Run, None);
        }
        if hit(sc(Modifiers::NONE, Key::F9)) {
            self.s.action(Action::AssembleBuild, None);
        }
        if hit(sc(Modifiers::NONE, Key::F6)) {
            self.s.action(Action::Build, None);
        }
        if hit(sc(Modifiers::NONE, Key::F8)) {
            self.s.action(Action::Debug, None);
        }
        if hit(sc(Modifiers::NONE, Key::F1)) {
            self.s.help = true;
        }
        if hit(sc(cmd, Key::Comma)) {
            self.s.prefs = Some(self.s.settings.clone());
        }
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        let t = |k: &str| self.s.t(k);
        let (m_file, m_edit, m_view, m_build, m_debug, m_tools, m_win, m_help) = (
            t("IDE.MenuFile"),
            t("IDE.MenuEdit"),
            t("IDE.MenuView"),
            t("IDE.MenuBuild"),
            t("IDE.MenuDebug"),
            t("IDE.MenuTools"),
            t("IDE.MenuWindows"),
            t("IDE.MenuHelp"),
        );
        let btn = |label: String, sc: &str| egui::Button::new(label).shortcut_text(sc.to_string());
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button(m_file, |ui| {
                if ui.add(btn(self.s.t("IDE.MenuNewTab"), "Ctrl+N")).clicked() {
                    self.s.new_doc(String::new());
                    ui.close();
                }
                if ui.button(format!("{} (plantilla)", self.s.t("IDE.MenuNew"))).clicked() {
                    self.s.new_doc(NEW_FILE_TEMPLATE.to_string());
                    ui.close();
                }
                if ui.add(btn(self.s.t("IDE.MenuOpen"), "Ctrl+O")).clicked() {
                    ui.close();
                    self.s.open_dialog();
                }
                ui.menu_button(self.s.t("IDE.MenuRecentFiles"), |ui| {
                    let recent = self.s.settings.recent_files.clone();
                    if recent.is_empty() {
                        ui.label(RichText::new(self.s.t("Others.None")).weak());
                    }
                    for (i, p) in recent.iter().enumerate() {
                        if ui.button(format!("{}  {}", i + 1, p.display())).clicked() {
                            ui.close();
                            if p.exists() {
                                self.s.open_path(p, true);
                            } else {
                                let msg = format!(
                                    "{}\n{}\n\n{}",
                                    self.s.t("RecentFile.MissingFileMsg"),
                                    p.display(),
                                    self.s.t("RecentFile.RemoveMissingFileConfirmation")
                                );
                                self.s.confirm_push(Confirm {
                                    title: self.s.t("Others.MissingFileTitle"),
                                    message: msg,
                                    yes: Flow::RemoveRecent(p.clone()),
                                    no: Flow::None,
                                    cancel: false,
                                });
                            }
                        }
                    }
                    ui.separator();
                    if ui.button(self.s.t("IDE.MenuClearAll")).clicked() {
                        ui.close();
                        self.s.confirm_push(Confirm {
                            title: self.s.t("RecentFile.DeleteConfirmation"),
                            message: self.s.t("RecentFile.RemoveRecentFileListConfirmation"),
                            yes: Flow::ClearRecent,
                            no: Flow::None,
                            cancel: false,
                        });
                    }
                });
                ui.menu_button(self.s.t("IDE.MenuExamples"), |ui| {
                    for (name, text) in EXAMPLES {
                        if ui.button(*name).clicked() {
                            let id = self.s.new_doc(text.replace("\r\n", "\n"));
                            if let Some(d) = self.s.docs.get_mut(&id) {
                                d.untitled_n = 0;
                                d.path = None;
                            }
                            EXAMPLE_NAMES.with(|m| m.borrow_mut().insert(id, name.to_string()));
                            ui.close();
                        }
                    }
                });
                ui.separator();
                let active = self.active_doc_id();
                if ui.add_enabled(active.is_some(), btn(self.s.t("IDE.MenuSave"), "Ctrl+S")).clicked() {
                    ui.close();
                    self.s.save_doc(active.unwrap(), false);
                }
                if ui.add_enabled(active.is_some(), btn(self.s.t("IDE.MenuSaveAs"), "Ctrl+Shift+S")).clicked() {
                    ui.close();
                    self.s.save_doc(active.unwrap(), true);
                }
                ui.separator();
                if ui.add_enabled(active.is_some(), egui::Button::new(self.s.t("IDE.MenuOpenContainingFolder"))).clicked() {
                    ui.close();
                    match active.and_then(|id| self.s.docs.get(&id)).and_then(|d| d.path.clone()) {
                        Some(p) => {
                            let _ = open::that_detached(p.parent().unwrap_or(Path::new(".")));
                        }
                        None => self.s.message = Some((self.s.t("IDE.OpenFolder"), self.s.t("IDE.OpenFolderError"))),
                    }
                }
                if ui.add_enabled(active.is_some(), egui::Button::new(self.s.t("IDE.MenuViewFileProperties"))).clicked() {
                    ui.close();
                    let id = active.unwrap();
                    if self.s.docs.get(&id).and_then(|d| d.path.as_ref()).is_some() {
                        self.s.properties = Some(id);
                    } else {
                        self.s.message = Some((self.s.t("IDE.FileProperties"), self.s.t("IDE.FilePropertiesError")));
                    }
                }
                if ui.add_enabled(active.is_some(), egui::Button::new(self.s.t("IDE.MenuPrint"))).clicked() {
                    ui.close();
                    self.s.print_doc(active.unwrap());
                }
                ui.separator();
                if ui.add(btn(self.s.t("IDE.MenuCloseActive"), "Ctrl+W")).clicked() {
                    ui.close();
                    if let Some(id) = active {
                        self.s.request_close(vec![id]);
                    }
                }
                if ui.button(self.s.t("IDE.MenuCloseAll")).clicked() {
                    ui.close();
                    let all: Vec<u64> = self.s.docs.keys().copied().collect();
                    self.s.request_close(all);
                }
                ui.separator();
                if ui.add(btn(self.s.t("Port.Quit"), "Ctrl+Q")).clicked() {
                    ui.close();
                    ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                }
            });

            ui.menu_button(m_edit, |ui| {
                let items: [(&str, &str, EditOp); 7] = [
                    ("IDE.MenuBarUndoText", "Ctrl+Z", EditOp::Undo),
                    ("IDE.MenuBarRedoText", "Ctrl+Y", EditOp::Redo),
                    ("IDE.MenuBarCutText", "Ctrl+X", EditOp::Cut),
                    ("IDE.MenuBarCopyText", "Ctrl+C", EditOp::Copy),
                    ("IDE.MenuBarPasteText", "Ctrl+V", EditOp::Paste),
                    ("IDE.MenuBarDeleteText", "Supr", EditOp::Delete),
                    ("IDE.MenuSelectAll", "Ctrl+A", EditOp::SelectAll),
                ];
                for (i, (k, sc, op)) in items.iter().enumerate() {
                    if i == 2 || i == 6 {
                        ui.separator();
                    }
                    if ui.add(btn(self.s.t(k), sc)).clicked() {
                        self.push_op(*op);
                        ui.close();
                    }
                }
                ui.separator();
                if ui.add(btn(self.s.t("IDE.Find"), "Ctrl+F")).clicked() {
                    self.s.find.open = true;
                    self.s.find.replace = false;
                    self.s.find.focus = true;
                    ui.close();
                }
                if ui.add(btn(self.s.t("IDE.FindAndReplace"), "Ctrl+H")).clicked() {
                    self.s.find.open = true;
                    self.s.find.replace = true;
                    self.s.find.focus = true;
                    ui.close();
                }
                if ui.add(btn(self.s.t("IDE.GoToLine"), "Ctrl+G")).clicked() {
                    self.s.goto = Some(String::new());
                    ui.close();
                }
                ui.separator();
                if ui.add(btn(self.s.t("IDE.MenuBarCommentCodeText"), "Ctrl+/")).clicked() {
                    self.push_op(EditOp::ToggleComment);
                    ui.close();
                }
                if ui.add(btn(self.s.t("IDE.MenuBarCodeUpText"), "Alt+↑")).clicked() {
                    self.push_op(EditOp::MoveUp);
                    ui.close();
                }
                if ui.add(btn(self.s.t("IDE.MenuBarCodeDownText"), "Alt+↓")).clicked() {
                    self.push_op(EditOp::MoveDown);
                    ui.close();
                }
                ui.menu_button(self.s.t("IDE.MenuChangeCaseTo"), |ui| {
                    if ui.add(btn(self.s.t("IDE.MenuChangeCaseToUppercase"), "Ctrl+Shift+U")).clicked() {
                        self.push_op(EditOp::Upper);
                        ui.close();
                    }
                    if ui.add(btn(self.s.t("IDE.MenuChangeCaseToLowercase"), "Ctrl+U")).clicked() {
                        self.push_op(EditOp::Lower);
                        ui.close();
                    }
                });
            });

            ui.menu_button(m_view, |ui| {
                let mut out = self.s.settings.show_output;
                if ui.checkbox(&mut out, self.s.t("IDE.MenuViewOuptutWindow")).clicked() {
                    if out { self.s.open_tabs.push(Tab::Output) } else { self.s.close_tabs.push(Tab::Output) }
                }
                let mut term = self.s.settings.show_terminal;
                if ui.checkbox(&mut term, self.s.t("IDE.MenuViewTerminalWindow")).clicked() {
                    if term { self.s.open_tabs.push(Tab::Terminal) } else { self.s.close_tabs.push(Tab::Terminal) }
                }
                let label = self.s.t("IDE.MenuViewToolBar");
                ui.checkbox(&mut self.s.settings.show_toolbar, label);
                let label = self.s.t("IDE.MenuViewStatusBar");
                ui.checkbox(&mut self.s.settings.show_statusbar, label);
                ui.separator();
                let label = self.s.t("IDE.WordWrap");
                ui.checkbox(&mut self.s.settings.word_wrap, label);
                ui.menu_button(self.s.t("IDE.MenuViewEditorTheme"), |ui| {
                    if ui.radio(!self.s.settings.editor_dark, self.s.t("IDE.MenuViewEditorThemeLight")).clicked() {
                        self.s.settings.editor_dark = false;
                    }
                    if ui.radio(self.s.settings.editor_dark, self.s.t("IDE.MenuViewEditorThemeDark")).clicked() {
                        self.s.settings.editor_dark = true;
                    }
                });
                ui.menu_button(self.s.t("Preference.SelectIdeTheme"), |ui| {
                    for (th, name) in [(IdeTheme::Blue, "Blue"), (IdeTheme::Dark, "Dark"), (IdeTheme::Light, "Light")] {
                        if ui.radio(self.s.settings.ide_theme == th, self.s.t(&format!("Port.Theme{name}"))).clicked() {
                            self.s.settings.ide_theme = th;
                        }
                    }
                });
                ui.separator();
                if ui.add(btn(self.s.t("IDE.MenuBarIncreaseZoomText"), "Ctrl++")).clicked() {
                    self.s.settings.font_size = (self.s.settings.font_size + 1.0).min(40.0);
                }
                if ui.add(btn(self.s.t("IDE.MenuBarDecreaseZoomText"), "Ctrl+-")).clicked() {
                    self.s.settings.font_size = (self.s.settings.font_size - 1.0).max(8.0);
                }
                if ui.add(btn(self.s.t("IDE.MenuBarResetZoomText"), "Ctrl+0")).clicked() {
                    self.s.settings.font_size = Settings::default().font_size;
                }
            });

            ui.menu_button(m_build, |ui| {
                let busy = self.s.task.is_some();
                let entries = [
                    ("IDE.MenuAssemble", "Alt+F9", Action::Assemble),
                    ("IDE.MenuBuild", "F6", Action::Build),
                    ("IDE.MenuAssembleAndBuild", "F9", Action::AssembleBuild),
                    ("IDE.MenuRun", "Shift+F9", Action::Run),
                    ("IDE.MenuAssembleBuildAndRun", "Ctrl+F9", Action::AssembleBuildRun),
                ];
                for (k, sc, a) in entries {
                    let label = self.s.t(k).replace("  ", " & ");
                    if ui.add_enabled(!busy, btn(label, sc)).clicked() {
                        self.s.action(a, None);
                        ui.close();
                    }
                }
            });

            ui.menu_button(m_debug, |ui| {
                let busy = self.s.task.is_some();
                if ui.add_enabled(!busy, btn(format!("🐞 {} (Turbo Debugger)", self.s.t("IDE.MenuBarDebugText")), "F8")).clicked() {
                    self.s.action(Action::Debug, None);
                    ui.close();
                }
            });

            ui.menu_button(m_tools, |ui| {
                if ui.add(btn(format!("⚙ {}", self.s.t("IDE.MenuBarPreferenceText")), "Ctrl+,")).clicked() {
                    self.s.prefs = Some(self.s.settings.clone());
                    ui.close();
                }
                ui.menu_button(self.s.t("IDE.MenuLogConsole"), |ui| {
                    let label = if self.s.settings.show_output { self.s.t("IDE.HideLogConsole") } else { self.s.t("IDE.ShowLogConsole") };
                    if ui.button(label).clicked() {
                        if self.s.settings.show_output { self.s.close_tabs.push(Tab::Output) } else { self.s.open_tabs.push(Tab::Output) }
                        ui.close();
                    }
                    if ui.button(self.s.t("IDE.MenuClearLogConsole")).clicked() {
                        self.s.logs.clear();
                        ui.close();
                    }
                });
                if ui.button(self.s.t("Port.ResetSettings")).clicked() {
                    let recent = std::mem::take(&mut self.s.settings.recent_files);
                    self.s.settings = Settings { recent_files: recent, ..Settings::default() };
                    let m = self.s.t("Port.SettingsRestored");
                    self.s.log(LogKind::Success, m);
                    ui.close();
                }
            });

            ui.menu_button(m_win, |ui| {
                let mut tabs: Vec<(u64, String)> = Vec::new();
                for (_, tab) in self.dock.iter_all_tabs() {
                    if let Tab::Editor(id) = tab {
                        if let Some(d) = self.s.docs.get(id) {
                            tabs.push((*id, d.title(&self.s.untitled())));
                        }
                    }
                }
                for (id, title) in tabs {
                    if ui.radio(self.s.active_doc == Some(id), title).clicked() {
                        self.s.focus_tab = Some(Tab::Editor(id));
                        ui.close();
                    }
                }
                ui.separator();
                if ui.button(self.s.t("IDE.MenuCloseActive")).clicked() {
                    if let Some(id) = self.s.active_doc {
                        self.s.request_close(vec![id]);
                    }
                    ui.close();
                }
                if ui.button(self.s.t("IDE.MenuCloseOthers")).clicked() {
                    let a = self.s.active_doc;
                    let others: Vec<u64> = self.s.docs.keys().copied().filter(|x| Some(*x) != a).collect();
                    self.s.request_close(others);
                    ui.close();
                }
                if ui.button(self.s.t("IDE.MenuCloseAll")).clicked() {
                    let all: Vec<u64> = self.s.docs.keys().copied().collect();
                    self.s.request_close(all);
                    ui.close();
                }
            });

            ui.menu_button(m_help, |ui| {
                if ui.add(btn(self.s.t("IDE.MenuBarHelpText"), "F1")).clicked() {
                    self.s.help = true;
                    ui.close();
                }
                ui.menu_button(format!("📖 {}", self.s.t("Port.Manual")), |ui| manual_menu(ui, &mut self.s));
                if ui.button(self.s.t("Port.ViewTasmArgs")).clicked() {
                    self.s.open_tabs.push(Tab::Terminal);
                    self.s.run_terminal("TASM".into());
                    ui.close();
                }
                if ui.button(self.s.t("Port.ViewTlinkArgs")).clicked() {
                    self.s.open_tabs.push(Tab::Terminal);
                    self.s.run_terminal("TLINK".into());
                    ui.close();
                }
                ui.separator();
                if ui.button(self.s.t("Others.VisitGithub")).clicked() {
                    let _ = open::that_detached(ORIGINAL_REPO);
                    ui.close();
                }
                if ui.button(format!("{} {APP_NAME}", self.s.t("Others.About"))).clicked() {
                    self.s.about = true;
                    ui.close();
                }
            });
        });
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        let busy = self.s.task.is_some();
        let active = self.active_doc_id();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            let tb = |ui: &mut egui::Ui, icon: &str, tip: String, enabled: bool| -> bool {
                ui.add_enabled(enabled, egui::Button::new(RichText::new(icon).size(16.0)).frame(false).min_size(Vec2::new(30.0, 26.0)))
                    .on_hover_text(tip)
                    .clicked()
            };
            if tb(ui, "🗋", self.s.t("IDE.MenuBarNewText"), true) {
                self.s.new_doc(String::new());
            }
            if tb(ui, "🗁", self.s.t("IDE.MenuBarOpenText"), true) {
                self.s.open_dialog();
            }
            if tb(ui, "💾", self.s.t("IDE.MenuBarSaveText"), active.is_some()) {
                self.s.save_doc(active.unwrap(), false);
            }
            if tb(ui, "🖶", self.s.t("IDE.MenuBarPrintText"), active.is_some()) {
                self.s.print_doc(active.unwrap());
            }
            ui.separator();
            for (icon, k, op) in [
                ("✂", "IDE.MenuBarCutText", EditOp::Cut),
                ("🗐", "IDE.MenuBarCopyText", EditOp::Copy),
                ("📥", "IDE.MenuBarPasteText", EditOp::Paste),
                ("⟲", "IDE.MenuBarUndoText", EditOp::Undo),
                ("⟳", "IDE.MenuBarRedoText", EditOp::Redo),
            ] {
                if tb(ui, icon, self.s.t(k), active.is_some()) {
                    self.push_op(op);
                }
            }
            ui.separator();
            if tb(ui, "🔍", self.s.t("IDE.Find"), active.is_some()) {
                self.s.find.open = !self.s.find.open;
                self.s.find.focus = true;
            }
            if tb(ui, ";", self.s.t("IDE.MenuBarCommentCodeText"), active.is_some()) {
                self.push_op(EditOp::ToggleComment);
            }
            if tb(ui, "⏶", self.s.t("IDE.MenuBarCodeUpText"), active.is_some()) {
                self.push_op(EditOp::MoveUp);
            }
            if tb(ui, "⏷", self.s.t("IDE.MenuBarCodeDownText"), active.is_some()) {
                self.push_op(EditOp::MoveDown);
            }
            ui.separator();
            let c = theme::colors(self.s.settings.ide_theme);
            let action_btn = |ui: &mut egui::Ui, icon: &str, text: String, tip: String, color: Color32| -> bool {
                ui.add_enabled(
                    !busy && active.is_some(),
                    egui::Button::new(RichText::new(format!("{icon} {text}")).color(color).strong()).min_size(Vec2::new(0.0, 26.0)),
                )
                .on_hover_text(tip)
                .clicked()
            };
            if action_btn(ui, "⚙", self.s.t("IDE.MenuAssemble"), format!("{} (Alt+F9)", self.s.t("IDE.MenuBarAssembleText")), c.log_fg) {
                self.s.action(Action::Assemble, None);
            }
            if action_btn(ui, "🔗", self.s.t("IDE.MenuBuild"), format!("{} (F6)", self.s.t("IDE.MenuBarLinkText")), c.log_fg) {
                self.s.action(Action::Build, None);
            }
            if action_btn(ui, "▶", self.s.t("IDE.MenuRun"), format!("{} (Ctrl+F9)", self.s.t("IDE.MenuBarAssembleBuildAndRunText")), c.success) {
                self.s.action(Action::AssembleBuildRun, None);
            }
            if action_btn(ui, "🐞", "TD".into(), format!("{} (F8)", self.s.t("IDE.MenuBarDebugText")), c.warning) {
                self.s.action(Action::Debug, None);
            }
            if busy {
                ui.separator();
                ui.add(egui::Spinner::new());
                if ui.button(format!("⏹ {}", self.s.t("Others.Cancel"))).clicked() {
                    if let Some(t) = &self.s.task {
                        t.cancel.cancel();
                    }
                }
            }
            ui.separator();
            let manual_label = RichText::new(format!("📖 {}", self.s.t("Port.Manual"))).strong();
            ui.menu_button(manual_label, |ui| manual_menu(ui, &mut self.s))
                .response
                .on_hover_text(self.s.t("Port.ManualTip"));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if tb(ui, "⚙", self.s.t("IDE.MenuBarPreferenceText"), true) {
                    self.s.prefs = Some(self.s.settings.clone());
                }
                ui.label(RichText::new("TASM 4.1 · 16-bit").color(c.dim).small());
            });
        });
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        let c = theme::colors(self.s.settings.ide_theme);
        let bg = if self.s.task.is_some() { c.status_busy } else { c.status_bg };
        egui::Frame::new().fill(bg).inner_margin(egui::Margin::symmetric(10, 3)).show(ui, |ui| {
            ui.horizontal(|ui| {
                let fg = c.status_fg;
                let txt = |s: String| RichText::new(s).color(fg).small();
                if let Some(t) = &self.s.task {
                    ui.label(txt(format!("⏳ {} · {:.1} s", t.label, t.started.elapsed().as_secs_f32())));
                } else {
                    ui.label(txt(self.s.status.clone()));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let db = match &self.s.dosbox {
                        Some(d) => d.flavor.label().to_string(),
                        None => format!("⚠ {}", self.s.t("Port.NoDosbox")),
                    };
                    ui.label(txt(db));
                    ui.label(txt("|".into()));
                    if let Some(d) = self.s.active_doc.and_then(|id| self.s.docs.get(&id)) {
                        ui.label(txt(format!("{}: {}", self.s.t("IDE.TotalCharacters"), d.text.chars().count())));
                        ui.label(txt(format!("{}: {}", self.s.t("IDE.TotalLines"), d.line_count())));
                        if d.selection_chars > 0 {
                            ui.label(txt(format!("Sel: {}", d.selection_chars)));
                        }
                        ui.label(txt(format!("{} {}, Col {}", self.s.t("IDE.Line"), d.cursor_line, d.cursor_col)));
                        let state = if d.is_built() {
                            "EXE ✔"
                        } else if d.is_assembled() {
                            "OBJ ✔"
                        } else {
                            ""
                        };
                        if !state.is_empty() {
                            ui.label(txt(state.into()));
                        }
                    }
                });
            });
        });
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        // --- Confirmación ---
        if let Some(c) = &self.s.confirm {
            let (title, message, cancel) = (c.title.clone(), c.message.clone(), c.cancel);
            let mut answer: Option<Option<bool>> = None;
            let resp = egui::Modal::new(egui::Id::new("confirm")).show(ctx, |ui| {
                ui.set_width(420.0);
                ui.heading(&title);
                ui.add_space(6.0);
                ui.label(&message);
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new(self.s.t("Others.Yes")).strong()).clicked() || ui.input(|i| i.key_pressed(Key::Enter)) {
                        answer = Some(Some(true));
                    }
                    if ui.button(self.s.t("Others.No")).clicked() {
                        answer = Some(Some(false));
                    }
                    if cancel && ui.button(self.s.t("Others.Cancel")).clicked() {
                        answer = Some(None);
                    }
                });
            });
            if resp.should_close() && answer.is_none() {
                answer = Some(if cancel { None } else { Some(false) });
            }
            if let Some(a) = answer {
                let c = self.s.confirm.take().unwrap();
                if let Some(next) = CONFIRM_QUEUE.with(|q| {
                    let mut q = q.borrow_mut();
                    if q.is_empty() { None } else { Some(q.remove(0)) }
                }) {
                    self.s.confirm = Some(next);
                }
                match a {
                    Some(true) => self.s.run_flow(c.yes),
                    Some(false) => self.s.run_flow(c.no),
                    None => {
                        // Cancelar también cancela una salida en curso.
                        CONFIRM_QUEUE.with(|q| q.borrow_mut().clear());
                        self.s.confirm = None;
                    }
                }
            }
        }

        // --- Mensaje simple ---
        if let Some((title, msg)) = self.s.message.clone() {
            let resp = egui::Modal::new(egui::Id::new("message")).show(ctx, |ui| {
                ui.set_width(440.0);
                ui.heading(&title);
                ui.add_space(6.0);
                ui.add(egui::Label::new(&msg).selectable(true));
                ui.add_space(10.0);
                ui.button(self.s.t("Others.Ok")).clicked() || ui.input(|i| i.key_pressed(Key::Enter))
            });
            if resp.inner || resp.should_close() {
                self.s.message = None;
            }
        }

        // --- Ir a línea ---
        if let Some(mut g) = self.s.goto.take() {
            let max = self.s.active_doc.and_then(|id| self.s.docs.get(&id)).map(|d| d.line_count()).unwrap_or(1);
            let mut keep = true;
            let resp = egui::Modal::new(egui::Id::new("goto")).show(ctx, |ui| {
                ui.heading(self.s.t("IDE.GoToLine"));
                ui.label(format!("{} (1 - {max})", self.s.t("IDE.LineNumber")));
                let r = ui.add(egui::TextEdit::singleline(&mut g).desired_width(200.0));
                r.request_focus();
                ui.horizontal(|ui| {
                    let ok = ui.button(self.s.t("Others.Ok")).clicked() || ui.input(|i| i.key_pressed(Key::Enter));
                    if ok {
                        if let Ok(n) = g.trim().parse::<usize>() {
                            let n = n.clamp(1, max);
                            if let Some(d) = self.s.active_doc.and_then(|id| self.s.docs.get_mut(&id)) {
                                d.goto_line = Some(n);
                            }
                        }
                        keep = false;
                    }
                    if ui.button(self.s.t("Others.Cancel")).clicked() {
                        keep = false;
                    }
                });
            });
            if keep && !resp.should_close() {
                self.s.goto = Some(g);
            }
        }

        // --- Propiedades ---
        if let Some(id) = self.s.properties {
            let mut open = true;
            let info = self.s.docs.get(&id).and_then(|d| d.path.clone()).map(|p| {
                let m = std::fs::metadata(&p);
                let size = m.as_ref().map(|m| m.len()).unwrap_or(0);
                (p.clone(), size, m.as_ref().ok().map(|m| (fmt_time(m.created()), fmt_time(m.modified()), fmt_time(m.accessed()))))
            });
            egui::Window::new(self.s.t("IDE.FileProperties"))
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    if let Some((p, size, times)) = info {
                        egui::Grid::new("props").num_columns(2).spacing([16.0, 6.0]).show(ui, |ui| {
                            ui.label(RichText::new(self.s.t("IDE.FileName")).strong());
                            ui.label(p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default());
                            ui.end_row();
                            ui.label(RichText::new(self.s.t("IDE.PropertiesFileLocation")).strong());
                            ui.add(egui::Label::new(p.parent().map(|d| d.display().to_string()).unwrap_or_default()).selectable(true));
                            ui.end_row();
                            ui.label(RichText::new(self.s.t("Port.Size")).strong());
                            ui.label(format!("{size} bytes"));
                            ui.end_row();
                            if let Some((c, m, a)) = times {
                                ui.label(RichText::new(self.s.t("IDE.PropertiesCreatedOn")).strong());
                                ui.label(c);
                                ui.end_row();
                                ui.label(RichText::new(self.s.t("IDE.PropertiesLastModifiedOn")).strong());
                                ui.label(m);
                                ui.end_row();
                                ui.label(RichText::new(self.s.t("IDE.PropertiesLastAccessedOn")).strong());
                                ui.label(a);
                                ui.end_row();
                            }
                        });
                    }
                });
            if !open {
                self.s.properties = None;
            }
        }

        // --- Acerca de ---
        if self.s.about {
            let mut open = true;
            egui::Window::new(self.s.t("Others.About"))
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(4.0);
                        ui.label(RichText::new("⚙").size(42.0));
                        ui.heading(RichText::new(APP_NAME).strong());
                        ui.label(format!("{} {VERSION}", self.s.t("ChangeHistory.Version")));
                        ui.add_space(6.0);
                        ui.label(self.s.t("Port.AboutText"));
                        ui.add_space(6.0);
                        ui.hyperlink_to("GUI Turbo Assembler — Lakhya Jyoti Nath (ljnath)", ORIGINAL_REPO);
                        ui.label(RichText::new("GPL-3.0 · Rust · egui · DOSBox").weak());
                        ui.label(RichText::new("Turbo Assembler, Turbo Linker, Turbo Debugger © Borland International").weak().small());
                        if let Some(d) = &self.s.dosbox {
                            ui.label(RichText::new(format!("{} — {}", d.flavor.label(), d.version)).weak().small());
                        }
                    });
                });
            self.s.about = open;
        }

        // --- Ayuda ---
        if self.s.help {
            let mut open = true;
            egui::Window::new(self.s.t("IDE.MenuHelp"))
                .open(&mut open)
                .default_width(560.0)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().max_height(480.0).show(ui, |ui| {
                        ui.label(self.s.t("Port.HelpIntro"));
                        ui.add_space(8.0);
                        egui::Grid::new("keys").striped(true).num_columns(2).spacing([20.0, 4.0]).show(ui, |ui| {
                            let rows = [
                                ("Alt+F9", "IDE.MenuBarAssembleText"),
                                ("F6", "IDE.MenuBarLinkText"),
                                ("F9", "IDE.MenuBarAssembleAndBuildText"),
                                ("Shift+F9", "IDE.MenuBarRunText"),
                                ("Ctrl+F9", "IDE.MenuBarAssembleBuildAndRunText"),
                                ("F8", "IDE.MenuBarDebugText"),
                                ("Ctrl+N / Ctrl+O / Ctrl+S", "Port.KeysFile"),
                                ("Ctrl+F / Ctrl+H / Ctrl+G", "Port.KeysFind"),
                                ("Ctrl+/", "IDE.MenuBarCommentCodeText"),
                                ("Alt+↑ / Alt+↓", "Port.KeysMove"),
                                ("Ctrl+Shift+U / Ctrl+U", "IDE.MenuChangeCaseTo"),
                                ("Tab / Shift+Tab", "Port.KeysIndent"),
                                ("Ctrl + / - / 0, Ctrl+rueda", "Port.KeysZoom"),
                            ];
                            for (k, d) in rows {
                                ui.label(RichText::new(k).monospace().strong());
                                ui.label(self.s.t(d));
                                ui.end_row();
                            }
                        });
                        ui.add_space(8.0);
                        ui.label(self.s.t("Port.HelpTd"));
                    });
                });
            self.s.help = open;
        }

        // --- Preferencias ---
        self.preferences(ctx);
    }

    fn preferences(&mut self, ctx: &egui::Context) {
        let Some(mut p) = self.s.prefs.take() else { return };
        let mut open = true;
        let mut apply = false;
        let mut close = false;
        let i18n = self.s.i18n.clone();
        let t = |k: &str| i18n.t(k);
        egui::Window::new(t("Preference.Title"))
            .open(&mut open)
            .collapsible(false)
            .default_width(600.0)
            .min_width(600.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for (i, k) in ["Preference.Basic", "Preference.Editor", "Preference.Advance"].iter().enumerate() {
                        ui.selectable_value(&mut self.s.prefs_tab, i, RichText::new(t(k)).strong());
                    }
                });
                ui.separator();
                match self.s.prefs_tab {
                    0 => {
                        egui::Grid::new("pbasic").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
                            ui.label(t("Preference.SelectIdeLanguage"));
                            egui::ComboBox::from_id_salt("lang")
                                .selected_text(self.s.i18n.locales.iter().find(|l| l.code.eq_ignore_ascii_case(&p.language)).map(|l| l.name.clone()).unwrap_or(p.language.clone()))
                                .show_ui(ui, |ui| {
                                    for l in &self.s.i18n.locales {
                                        ui.selectable_value(&mut p.language, l.code.clone(), format!("{} ({})", l.name, l.code));
                                    }
                                });
                            ui.end_row();
                            ui.label(t("Preference.SelectIdeTheme"));
                            ui.horizontal(|ui| {
                                for (th, name) in [(IdeTheme::Blue, "Blue"), (IdeTheme::Dark, "Dark"), (IdeTheme::Light, "Light")] {
                                    ui.radio_value(&mut p.ide_theme, th, t(&format!("Port.Theme{name}")));
                                }
                            });
                            ui.end_row();
                        });
                        ui.add_space(6.0);
                        ui.label(RichText::new(t("Preference.CustomizeIde")).strong());
                        ui.checkbox(&mut p.show_toolbar, t("Preference.DisplayStandardToolbar"));
                        ui.checkbox(&mut p.show_statusbar, t("Preference.DisplayStatusBar"));
                        ui.checkbox(&mut p.show_output, t("Preference.DisplayLogConsole"));
                        ui.checkbox(&mut p.auto_scroll, t("Preference.EnableAutoScrollInLogConsole"));
                        ui.checkbox(&mut p.restore_session, t("Preference.RestoreLastSesion"));
                        ui.checkbox(&mut p.restore_window, t("Preference.RestoreIdePosAndSize"));
                        ui.checkbox(&mut p.launch_maximized, t("Preference.LaunchInMaximixedMode"));
                    }
                    1 => {
                        ui.label(RichText::new(t("Preference.CustomizeEditor")).strong());
                        ui.checkbox(&mut p.editor_dark, t("Preference.EnableEditorDarkMode"));
                        ui.checkbox(&mut p.word_wrap, t("Preference.EnableTextWrappingInEditor"));
                        ui.checkbox(&mut p.auto_indent, t("Preference.EnableAutoLineIndentInEditor"));
                        ui.horizontal(|ui| {
                            ui.label(t("Preference.SelectEditorFontSize"));
                            ui.add(egui::Slider::new(&mut p.font_size, 8.0..=40.0).step_by(1.0));
                        });
                        ui.add_space(6.0);
                        let pal = if p.editor_dark { Palette::dark() } else { Palette::light() };
                        egui::Frame::new().fill(pal.background).inner_margin(8.0).corner_radius(4.0).show(ui, |ui| {
                            let job = crate::highlight::layout(
                                "inicio:  mov ax, @data     ; comentario\n         int 21h\nmsg      DB 'Hola$', 0Dh",
                                &pal,
                                egui::FontId::monospace(p.font_size),
                                f32::INFINITY,
                                &[],
                            );
                            ui.label(job);
                        });
                    }
                    _ => {
                        ui.label(RichText::new(t("Preference.SelectTurboArchitecture")).strong());
                        ui.label("Turbo 16-bit · TASM 4.1 · TLINK 7.1 · TD 5.0 (DOSBox)");
                        ui.add_space(6.0);
                        ui.label(RichText::new(t("Preference.AdvCustomizeArgs")).strong());
                        ui.label(RichText::new(t("Preference.CustomizeArgsNote")).small().weak());
                        egui::Grid::new("padv").num_columns(3).spacing([10.0, 8.0]).show(ui, |ui| {
                            ui.label(t("Others.TurboAssembler"));
                            ui.add_sized([240.0, 22.0], egui::TextEdit::singleline(&mut p.tasm_args).font(TextStyle::Monospace));
                            if ui.small_button(t("Others.Default")).clicked() {
                                p.tasm_args = config::DEFAULT_TASM_ARGS.into();
                            }
                            ui.end_row();
                            ui.label(t("Others.TurboLinker"));
                            ui.add_sized([240.0, 22.0], egui::TextEdit::singleline(&mut p.tlink_args).font(TextStyle::Monospace));
                            if ui.small_button(t("Others.Default")).clicked() {
                                p.tlink_args = config::DEFAULT_TLINK_ARGS.into();
                            }
                            ui.end_row();
                        });
                        ui.label(RichText::new(t("Preference.CustomizeArgsWaningMessage")).small().color(theme::colors(p.ide_theme).warning));
                        ui.add_space(6.0);
                        ui.label(RichText::new(t("Preference.AdvCurrentArgs")).strong());
                        ui.checkbox(&mut p.use_launcher, t("Preference.UseCustomLauncherToRunExe"));
                        ui.checkbox(&mut p.silent_tools, t("Port.SilentTools"));
                        egui::Grid::new("pdos").num_columns(3).spacing([10.0, 8.0]).show(ui, |ui| {
                            ui.label(t("Port.DosboxPath"));
                            ui.add_sized([300.0, 22.0], egui::TextEdit::singleline(&mut p.dosbox_path).hint_text(
                                self.s.dosbox.as_ref().map(|d| d.exe.display().to_string()).unwrap_or_else(|| "dosbox-staging".into()),
                            ));
                            if ui.small_button("…").clicked() {
                                if let Some(f) = rfd::FileDialog::new().pick_file() {
                                    p.dosbox_path = f.display().to_string();
                                }
                            }
                            ui.end_row();
                            ui.label(t("Port.BinDir"));
                            ui.add_sized([300.0, 22.0], egui::TextEdit::singleline(&mut p.bin_dir).hint_text(
                                self.s.bin_dir.as_ref().map(|d| d.display().to_string()).unwrap_or_default(),
                            ));
                            if ui.small_button("…").clicked() {
                                if let Some(f) = rfd::FileDialog::new().pick_folder() {
                                    p.bin_dir = f.display().to_string();
                                }
                            }
                            ui.end_row();
                        });
                    }
                }
                ui.add_space(8.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button(RichText::new(t("Others.Ok")).strong()).clicked() {
                        apply = true;
                        close = true;
                    }
                    if ui.button(t("Others.Cancel")).clicked() {
                        close = true;
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(t("Port.ResetSettings")).clicked() {
                            let keep = (p.recent_files.clone(), p.session_files.clone());
                            p = Settings { recent_files: keep.0, session_files: keep.1, ..Settings::default() };
                        }
                    });
                });
            });
        if apply {
            let lang_changed = !p.language.eq_ignore_ascii_case(&self.s.settings.language);
            let dos_changed = p.dosbox_path != self.s.settings.dosbox_path;
            let bin_changed = p.bin_dir != self.s.settings.bin_dir;
            if p.show_output != self.s.settings.show_output {
                if p.show_output { self.s.open_tabs.push(Tab::Output) } else { self.s.close_tabs.push(Tab::Output) }
            }
            // Conservar listas que pudieron cambiar mientras el diálogo estaba abierto.
            p.recent_files = self.s.settings.recent_files.clone();
            p.terminal_history = self.s.settings.terminal_history.clone();
            self.s.settings = p.clone();
            if lang_changed {
                let dir = config::find_locales_dir();
                self.s.i18n = Arc::new(I18n::load(dir.as_deref(), &p.language));
            }
            if dos_changed {
                self.s.dosbox = dosbox::detect(&p.dosbox_path);
            }
            if bin_changed {
                self.s.bin_dir = config::find_bin_dir(&p.bin_dir);
            }
            self.s.settings.save();
        }
        if open && !close {
            self.s.prefs = Some(p);
        }
    }
}

/// Temas del manual de TASM incluidos en `docs/Manual TASM`.
const MANUAL_TOPICS: &[(&str, &str)] = &[
    ("Tema01", "Computadora digital"),
    ("Tema02", "El microprocesador 8086"),
    ("Tema03", "Estructura de un programa en ensamblador"),
    ("Tema04", "Transferencia, aritmética y lógica"),
    ("Tema05", "Instrucciones de control de flujo"),
    ("Tema06", "Procedimientos"),
    ("Tema07", "Arreglos y cadenas"),
    ("Tema08", "Interrupciones"),
];

/// Contenido del menú "Manual TASM" (barra de herramientas y menú Ayuda).
fn manual_menu(ui: &mut egui::Ui, s: &mut State) {
    let Some(docs) = config::find_docs_dir() else {
        ui.label(RichText::new(s.t("Port.ManualMissing")).weak());
        return;
    };
    let open_pdf = |s: &mut State, p: PathBuf| match open::that_detached(&p) {
        Ok(()) => s.status = format!("📖 {}", p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()),
        Err(e) => s.message = Some((s.t("IDE.OpenError"), format!("{}\n{e}", p.display()))),
    };
    let full = docs.join("Manual-TASM(completo).pdf");
    if full.is_file() && ui.button(RichText::new(format!("📕 {}", s.t("Port.ManualFull"))).strong()).clicked() {
        open_pdf(s, full);
        ui.close();
    }
    ui.separator();
    let topics_dir = docs.join("Manual TASM");
    for (i, (stem, title)) in MANUAL_TOPICS.iter().enumerate() {
        // Las extensiones vienen mezcladas (.pdf / .PDF).
        let Some(p) = toolchain::find_ci(&topics_dir, &format!("{stem}.pdf")) else { continue };
        if ui.button(format!("{}. {title}", i + 1)).clicked() {
            open_pdf(s, p);
            ui.close();
        }
    }
    ui.separator();
    if ui.button(format!("🗁 {}", s.t("Port.ManualFolder"))).clicked() {
        let _ = open::that_detached(&docs);
        ui.close();
    }
}

thread_local! {
    static EXAMPLE_NAMES:std::cell::RefCell<HashMap<u64, String>> = std::cell::RefCell::new(HashMap::new());
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.s.theme_applied != Some(self.s.settings.ide_theme) {
            theme::apply(&ctx, self.s.settings.ide_theme);
            self.s.theme_applied = Some(self.s.settings.ide_theme);
        }
        self.s.poll_task(&ctx);
        self.shortcuts(&ctx);
        if ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Q))) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }

        // Archivos arrastrados a la ventana.
        let dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).filter(|p| !p.as_os_str().is_empty()).collect());
        for p in dropped {
            if self.s.open_path(&p, true).is_none() && self.s.message.is_none() {
                self.s.message = Some((self.s.t("IDE.OpenError"), self.s.t("IDE.DroppedFileErrorMsg")));
            }
        }

        // Cerrar la ventana: preguntar por archivos sin guardar.
        if ctx.input(|i| i.viewport().close_requested()) && !self.s.quit_confirmed {
            let dirty: Vec<String> = self.s.docs.values().filter(|d| d.is_dirty() && !d.text.is_empty()).map(|d| d.title(&self.s.untitled())).collect();
            if !dirty.is_empty() {
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                if self.s.confirm.is_none() {
                    self.s.confirm = Some(Confirm {
                        title: self.s.t("IDE.UnsavedFileTitle"),
                        message: format!("{}\n\n• {}", self.s.t("IDE.SaveFilePromptMsg"), dirty.join("\n• ")),
                        yes: Flow::SaveAllAndQuit,
                        no: Flow::Quit,
                        cancel: true,
                    });
                }
            } else {
                self.s.quit_confirmed = true;
            }
        }
        if self.s.quit_confirmed {
            if let Some(t) = &self.s.task {
                t.cancel.cancel();
            }
            if let Some(r) = ctx.input(|i| i.viewport().outer_rect) {
                self.s.settings.window = Some([r.min.x, r.min.y, r.width(), r.height()]);
            }
            self.save_state();
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }

        egui::Panel::top("menubar").show(ui, |ui| self.menu_bar(ui));
        if self.s.settings.show_toolbar {
            let c = theme::colors(self.s.settings.ide_theme);
            egui::Panel::top("toolbar")
                .frame(egui::Frame::new().fill(c.toolbar).inner_margin(egui::Margin::symmetric(6, 3)))
                .show(ui, |ui| self.toolbar(ui));
        }
        if self.s.settings.show_statusbar {
            egui::Panel::bottom("statusbar").frame(egui::Frame::NONE).show(ui, |ui| self.status_bar(ui));
        }

        // Ejemplos: el nombre sugerido al guardar.
        EXAMPLE_NAMES.with(|m| {
            let mut m = m.borrow_mut();
            m.retain(|id, _| self.s.docs.get(id).map(|d| d.path.is_none()).unwrap_or(false));
        });

        self.apply_tab_requests();
        let mut style = egui_dock::Style::from_egui(&ctx.global_style());
        let c = theme::colors(self.s.settings.ide_theme);
        style.tab.active.text_color = ctx.global_style().visuals.strong_text_color();
        style.tab.focused.text_color = ctx.global_style().visuals.strong_text_color();
        style.tab_bar.hline_color = c.accent;
        egui::CentralPanel::default().frame(egui::Frame::new().fill(c.panel)).show(ui, |ui| {
            if self.dock.iter_all_tabs().next().is_none() {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new(format!("{}  ·  Ctrl+N  ·  Ctrl+O", APP_NAME)).weak().size(18.0));
                });
                return;
            }
            let mut viewer = Viewer { s: &mut self.s };
            DockArea::new(&mut self.dock)
                .style(style)
                .show_add_buttons(false)
                .show_leaf_collapse_buttons(false)
                .show_leaf_close_all_buttons(false)
                .show_inside(ui, &mut viewer);
        });
        self.apply_tab_requests();
        self.dialogs(&ctx);

        // Ventana: título con el archivo activo.
        let title = match self.s.active_doc.and_then(|id| self.s.docs.get(&id)) {
            Some(d) => format!(
                "{}{} — {APP_NAME}",
                d.path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| d.title(&self.s.untitled())),
                if d.is_dirty() { " ●" } else { "" }
            ),
            None => APP_NAME.to_string(),
        };
        ctx.send_viewport_cmd(ViewportCommand::Title(title));
        self.devshot.tick(&ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if !self.s.quit_confirmed {
            self.save_state();
        }
    }
}
