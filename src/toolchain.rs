//! Orquesta TASM → TLINK → ejecución → TD, igual que el GUI Turbo Assembler
//! original, pero en un hilo de fondo que informa a la interfaz por un canal.

use crate::dosbox::{CancelHandle, DosBox, Job};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Assemble,
    Build,
    Run,
    Debug,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub file: String,
    pub line: usize,
    pub severity: Severity,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogKind {
    Info,
    Success,
    Warning,
    Error,
    Tool,
    Command,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ok,
    OkWithWarnings,
    Failed,
    Aborted,
    SubProcessFailed,
}

pub enum Event {
    Log(LogKind, String),
    /// Resultado de un paso concreto.
    StepDone { step: Step, outcome: Outcome, source: PathBuf, text_hash: u64 },
    Diagnostics { source: PathBuf, diags: Vec<Diagnostic> },
    TerminalOutput(String),
    /// La tarea completa terminó (libera la interfaz).
    Finished,
    /// El modo silencioso no funcionó con este DOSBox: se desactiva.
    SilentUnsupported,
}

#[derive(Clone)]
pub struct Request {
    pub steps: Vec<Step>,
    pub source: PathBuf,
    pub text_hash: u64,
    pub tasm_args: String,
    pub tlink_args: String,
    pub use_launcher: bool,
    pub silent: bool,
    pub bin_dir: PathBuf,
    pub dosbox: DosBox,
    pub launcher_msg: String,
}

/// Nombre DOS (sin extensión, en mayúsculas) del archivo.
pub fn dos_stem(p: &Path) -> String {
    p.file_stem().map(|s| s.to_string_lossy().to_uppercase()).unwrap_or_default()
}

/// Qué clase de archivo fuente es.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceKind {
    /// Programa con punto de entrada (`end inicio`): se ensambla, enlaza y depura.
    Program,
    /// Módulo secundario (`end` sin etiqueta): se enlaza junto con el principal.
    Module,
    /// Archivo de inclusión (`.INC`, macros): lo usa otro programa con `include`.
    Include,
}

/// Instrucciones del código (sin comentarios ni etiqueta inicial), en minúsculas
/// sólo la primera palabra: `(directiva, resto)`.
fn statements(text: &str) -> impl Iterator<Item = (String, String)> + '_ {
    text.lines().filter_map(|raw| {
        let mut code = String::new();
        let mut quote: Option<char> = None;
        for c in raw.chars() {
            match quote {
                Some(q) if c == q => quote = None,
                Some(_) => {}
                None if c == '\'' || c == '"' => quote = Some(c),
                None if c == ';' => break,
                None => {}
            }
            code.push(c);
        }
        let mut rest = code.trim();
        // Etiqueta al inicio: "inicio:  mov ax, 1"
        if let Some((first, after)) = rest.split_once(char::is_whitespace) {
            if first.ends_with(':') {
                rest = after.trim();
            }
        } else if rest.ends_with(':') {
            return None;
        }
        if rest.is_empty() {
            return None;
        }
        let (word, args) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
        Some((word.to_ascii_lowercase(), args.trim().to_string()))
    })
}

fn symbol_list(args: &str) -> Vec<String> {
    args.split(',')
        .filter_map(|s| s.split(':').next())
        .map(|s| s.trim().to_ascii_uppercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Símbolos que el archivo importa (`extrn x: byte`).
pub fn extrns(text: &str) -> Vec<String> {
    statements(text).filter(|(w, _)| w == "extrn" || w == "extern").flat_map(|(_, a)| symbol_list(&a)).collect()
}

/// Símbolos que el archivo exporta (`public x`).
pub fn publics(text: &str) -> Vec<String> {
    statements(text).filter(|(w, _)| w == "public").flat_map(|(_, a)| symbol_list(&a)).collect()
}

/// Archivos incluidos con `include "X.INC"`.
pub fn includes(text: &str) -> Vec<String> {
    statements(text)
        .filter(|(w, _)| w == "include")
        .map(|(_, a)| a.trim_matches(|c| c == '"' || c == '\'' || c == '<' || c == '>').trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn source_kind(path: &Path, text: &str) -> SourceKind {
    let ext = path.extension().map(|e| e.to_string_lossy().to_ascii_uppercase()).unwrap_or_default();
    if ext != "ASM" {
        return SourceKind::Include;
    }
    let ends: Vec<String> = statements(text).filter(|(w, _)| w == "end").map(|(_, a)| a).collect();
    if !ends.is_empty() && ends.iter().all(|a| a.is_empty()) {
        SourceKind::Module
    } else {
        SourceKind::Program
    }
}

fn read_lossy(p: &Path) -> String {
    std::fs::read(p).map(|b| String::from_utf8_lossy(&b).into_owned()).unwrap_or_default()
}

/// Otros .ASM de la misma carpeta (con nombre DOS válido) y su texto.
fn sibling_sources(source: &Path) -> Vec<(PathBuf, String)> {
    let dir = source.parent().unwrap_or(Path::new("."));
    let mut v: Vec<(PathBuf, String)> = std::fs::read_dir(dir)
        .map(|rd| rd.flatten().map(|e| e.path()).collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .filter(|p| p != source && p.is_file())
        .filter(|p| {
            let n = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            crate::stage::is_dos_name(&n) && n.to_ascii_uppercase().ends_with(".ASM")
        })
        .map(|p| {
            let t = read_lossy(&p);
            (p, t)
        })
        .collect();
    v.sort();
    v
}

fn file_name_of(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().to_uppercase()).unwrap_or_default()
}

/// Módulos secundarios de la carpeta que exportan (`public`) lo que el
/// programa importa (`extrn`), también de forma indirecta. Se ensamblan y
/// enlazan junto con él (como `TLINK PRINCIPA MODULO`).
pub fn linked_modules(source: &Path, text: &str) -> Vec<String> {
    let mut need: Vec<String> = extrns(text);
    if need.is_empty() {
        return Vec::new();
    }
    let candidates: Vec<(PathBuf, String)> =
        sibling_sources(source).into_iter().filter(|(p, t)| source_kind(p, t) == SourceKind::Module).collect();
    let mut have: Vec<String> = publics(text);
    let mut chosen: Vec<String> = Vec::new();
    loop {
        let missing: Vec<&String> = need.iter().filter(|s| !have.contains(s)).collect();
        let next = candidates.iter().find(|(p, t)| {
            !chosen.contains(&file_name_of(p)) && publics(t).iter().any(|s| missing.contains(&s))
        });
        let Some((p, t)) = next else { break };
        chosen.push(file_name_of(p));
        have.extend(publics(t));
        need.extend(extrns(t));
    }
    chosen
}

/// Programas de la carpeta que usan este módulo secundario.
pub fn programs_using_module(module: &Path) -> Vec<String> {
    sibling_sources(module)
        .into_iter()
        .filter(|(p, t)| source_kind(p, t) == SourceKind::Program && linked_modules(p, t).contains(&file_name_of(module)))
        .map(|(p, _)| file_name_of(&p))
        .collect()
}

/// Programas de la carpeta que incluyen este archivo (`include`).
pub fn programs_including(inc: &Path) -> Vec<String> {
    let name = file_name_of(inc);
    sibling_sources(inc)
        .into_iter()
        .filter(|(_, t)| includes(t).iter().any(|i| i.eq_ignore_ascii_case(&name)))
        .map(|(p, _)| file_name_of(&p))
        .collect()
}

/// Explicación en lenguaje claro de los errores más comunes de TASM/TLINK
/// (la salida de las herramientas está en inglés y es escueta).
fn explain(log: &str, t: &dyn Fn(&str) -> String) -> Vec<String> {
    let mut out = Vec::new();
    for l in log.lines() {
        let l = l.trim();
        let hint = if l.contains("No program entry point") {
            t("Port.HintNoEntry")
        } else if let Some(sym) = l.strip_prefix("Error: Undefined symbol ") {
            let sym = sym.split_whitespace().next().unwrap_or("");
            t("Port.HintUndefined").replace("{sym}", sym)
        } else if l.contains("Can't locate file") {
            let file = l.rsplit(':').next().unwrap_or("").trim();
            t("Port.HintNoFile").replace("{file}", file)
        } else {
            continue;
        };
        if !out.contains(&hint) {
            out.push(hint);
        }
    }
    out
}

/// Busca un archivo sin importar mayúsculas (DOS crea MAIN.EXE, el usuario main.asm).
pub fn find_ci(dir: &Path, name: &str) -> Option<PathBuf> {
    let rd = std::fs::read_dir(dir).ok()?;
    rd.flatten()
        .map(|e| e.path())
        .find(|p| p.file_name().map(|n| n.to_string_lossy().eq_ignore_ascii_case(name)).unwrap_or(false))
}

/// Interpreta la salida de TASM:
/// `**Error** main.asm(12) Undefined symbol: FOO`
/// `*Warning* main.asm(7) Argument needs type override`
pub fn parse_tasm(log: &str) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for raw in log.lines() {
        let l = raw.trim();
        let (sev, rest) = if let Some(r) = l.strip_prefix("**Error**") {
            (Severity::Error, r)
        } else if let Some(r) = l.strip_prefix("**Fatal**") {
            (Severity::Error, r)
        } else if let Some(r) = l.strip_prefix("*Warning*") {
            (Severity::Warning, r)
        } else {
            continue;
        };
        let rest = rest.trim();
        if let (Some(open), Some(close)) = (rest.find('('), rest.find(')')) {
            if open < close {
                if let Ok(line) = rest[open + 1..close].trim().parse::<usize>() {
                    let file = rest[..open].trim().to_string();
                    let mut message = rest[close + 1..].trim().to_string();
                    // Errores dentro de macros: "main.asm(12) MIMACRO(3) Mensaje"
                    if let (Some(o2), Some(c2)) = (message.find('('), message.find(')')) {
                        if o2 < c2 && !message[..o2].contains(' ') && message[o2 + 1..c2].trim().parse::<usize>().is_ok() {
                            message = format!("[{}] {}", &message[..c2 + 1], message[c2 + 1..].trim());
                        }
                    }
                    out.push(Diagnostic { file, line, severity: sev, message });
                    continue;
                }
            }
        }
        out.push(Diagnostic { file: String::new(), line: 0, severity: sev, message: rest.to_string() });
    }
    out
}

fn log_has_errors(log: &str) -> bool {
    log.lines().any(|l| {
        let t = l.trim_start();
        t.starts_with("**Error**") || t.starts_with("**Fatal**") || t.starts_with("Fatal:") || t.starts_with("Error:")
    })
}

fn log_has_warnings(log: &str) -> bool {
    log.lines().any(|l| {
        let t = l.trim_start();
        t.starts_with("*Warning*") || t.starts_with("Warning:")
    })
}

fn send_tool_log(tx: &Sender<Event>, text: &str) {
    for l in text.lines() {
        if !l.trim().is_empty() {
            let _ = tx.send(Event::Log(LogKind::Tool, l.to_string()));
        }
    }
}

fn job(req: &Request, work_dir: &Path, batch: Vec<String>, visible: bool) -> Job {
    Job {
        bin_dir: req.bin_dir.clone(),
        work_dir: work_dir.to_path_buf(),
        batch,
        visible,
        timeout: if visible { None } else { Some(Duration::from_secs(120)) },
    }
}

/// Lote para TASM y, opcionalmente, TLINK en una sola sesión de DOSBox
/// (arrancar DOSBox una sola vez es lo más rápido).
fn assemble_link_batch(req: &Request, name: &str, modules: &[String], assemble: bool, link: bool) -> Vec<String> {
    let mut b = Vec::new();
    if assemble {
        b.push(format!("TASM {} {}.ASM > E:\\ASM.LOG", req.tasm_args.trim(), name));
        b.push("IF ERRORLEVEL 1 GOTO ASMERR".into());
        b.push("ECHO OK> E:\\ASM.RC".into());
    }
    if link {
        // Los módulos secundarios se ensamblan siempre antes de enlazar.
        let mut objs = format!("{name}.OBJ");
        for (i, m) in modules.iter().enumerate() {
            b.push(format!("TASM {} {m} > E:\\MOD{}.LOG", req.tasm_args.trim(), i + 1));
            b.push("IF ERRORLEVEL 1 GOTO LNKERR".into());
            objs.push_str(&format!(" {}.OBJ", m.trim_end_matches(".ASM")));
        }
        b.push(format!("TLINK {} {objs} > E:\\LNK.LOG", req.tlink_args.trim()));
        b.push("IF ERRORLEVEL 1 GOTO LNKERR".into());
        b.push("ECHO OK> E:\\LNK.RC".into());
    }
    b.push("GOTO FIN".into());
    b.push(":ASMERR".into());
    b.push("ECHO ERR> E:\\ASM.RC".into());
    b.push("GOTO FIN".into());
    b.push(":LNKERR".into());
    b.push("ECHO ERR> E:\\LNK.RC".into());
    b.push(":FIN".into());
    b
}

/// Ejecuta un trabajo silencioso; si este DOSBox no soporta el modo sin
/// ventana (no produjo ningún archivo), lo reintenta visible.
fn run_tool_job(
    dosbox: &DosBox,
    req: &Request,
    work_dir: &Path,
    batch: Vec<String>,
    marker: &str,
    tx: &Sender<Event>,
    cancel: &CancelHandle,
) -> Result<crate::dosbox::JobOutput, String> {
    let mut visible = !req.silent;
    loop {
        let out = dosbox.run(&job(req, work_dir, batch.clone(), visible), cancel).map_err(|e| e.to_string())?;
        if out.cancelled || out.files.contains_key(marker) || visible {
            return Ok(out);
        }
        // Silencioso falló sin dejar rastro → reintentar con ventana.
        let _ = tx.send(Event::SilentUnsupported);
        if !out.stderr.trim().is_empty() {
            let _ = tx.send(Event::Log(LogKind::Warning, out.stderr.lines().last().unwrap_or("").to_string()));
        }
        visible = true;
    }
}

pub fn execute(req: Request, tx: Sender<Event>, cancel: CancelHandle, t: impl Fn(&str) -> String) {
    cancel.reset();
    let dosbox = req.dosbox.clone();
    let src = req.source.clone();
    let log = |k: LogKind, s: String| {
        let _ = tx.send(Event::Log(k, s));
    };

    // Copia de trabajo con nombres DOS y acentos en la página 437 (ver stage.rs).
    let mut stage = match crate::stage::Stage::new(&src) {
        Ok(s) => s,
        Err(e) => {
            log(LogKind::Error, format!("{} ({e})", t("IDE.AssembleFailedDueToSubProcess")));
            if let Some(&step) = req.steps.first() {
                let _ = tx.send(Event::StepDone { step, outcome: Outcome::SubProcessFailed, source: src.clone(), text_hash: req.text_hash });
            }
            let _ = tx.send(Event::Finished);
            return;
        }
    };
    let work = stage.dir.clone();
    let name = stage.alias.clone();
    if stage.renamed() {
        log(
            LogKind::Warning,
            t("Port.DosNameWarning")
                .replace("{file}", &stage.source_name)
                .replace("{dos}", &stage.dos_source())
                .replace("{stem}", &dos_stem(&src)),
        );
    }

    let want_asm = req.steps.contains(&Step::Assemble);
    let want_link = req.steps.contains(&Step::Build);
    let modules = if want_link { linked_modules(&src, &read_lossy(&src)) } else { Vec::new() };

    // --- Ensamblar y/o enlazar (silencioso) ---
    if want_asm || want_link {
        let started = Instant::now();
        if want_asm {
            log(LogKind::Info, format!("{} '{}' : {}", t("IDE.AssembleToObjectFile"), req.tasm_args.trim(), src.display()));
        }
        if want_link && !want_asm {
            log(LogKind::Info, format!("{} '{}'", t("IDE.BuildObjectFileToExe"), req.tlink_args.trim()));
        }
        if !modules.is_empty() {
            log(LogKind::Info, format!("{} {}", t("Port.LinkWithModules"), modules.join(", ")));
        }
        let marker = if want_asm { "ASM.RC" } else { "LNK.RC" };
        let batch = assemble_link_batch(&req, &name, &modules, want_asm, want_link);
        let res = run_tool_job(&dosbox, &req, &work, batch, marker, &tx, &cancel);
        stage.sync_back();
        let out = match res {
            Ok(o) => o,
            Err(e) => {
                log(LogKind::Error, format!("{} ({e})", t("IDE.AssembleFailedDueToSubProcess")));
                let step = if want_asm { Step::Assemble } else { Step::Build };
                let _ = tx.send(Event::StepDone { step, outcome: Outcome::SubProcessFailed, source: src.clone(), text_hash: req.text_hash });
                let _ = tx.send(Event::Finished);
                return;
            }
        };
        if out.cancelled {
            let key = if want_asm { "IDE.AssembleAbortByUser" } else { "IDE.BuildAbortByUser" };
            log(LogKind::Warning, t(key));
            let step = if want_asm { Step::Assemble } else { Step::Build };
            let _ = tx.send(Event::StepDone { step, outcome: Outcome::Aborted, source: src.clone(), text_hash: req.text_hash });
            let _ = tx.send(Event::Finished);
            return;
        }
        if out.timed_out {
            log(LogKind::Error, t("Port.Timeout"));
        }

        if want_asm {
            let alog = stage.restore_names(&out.files.get("ASM.LOG").cloned().unwrap_or_default());
            send_tool_log(&tx, &alog);
            let diags = parse_tasm(&alog);
            let _ = tx.send(Event::Diagnostics { source: src.clone(), diags });
            let rc_ok = out.files.get("ASM.RC").map(|s| s.trim() == "OK").unwrap_or(false);
            let outcome = if !out.files.contains_key("ASM.RC") {
                log(LogKind::Error, t("IDE.AssembleFailedDueToSubProcess"));
                if !out.stderr.trim().is_empty() {
                    send_tool_log(&tx, &out.stderr);
                }
                Outcome::SubProcessFailed
            } else if !rc_ok || log_has_errors(&alog) {
                for h in explain(&alog, &t) {
                    log(LogKind::Error, h);
                }
                log(LogKind::Error, t("IDE.AssembleFailedWithError"));
                Outcome::Failed
            } else if log_has_warnings(&alog) {
                log(LogKind::Warning, t("IDE.AssembleCompleteWithWarning"));
                Outcome::OkWithWarnings
            } else {
                log(LogKind::Success, t("IDE.AssembleSucessful"));
                Outcome::Ok
            };
            let _ = tx.send(Event::StepDone { step: Step::Assemble, outcome, source: src.clone(), text_hash: req.text_hash });
            if !matches!(outcome, Outcome::Ok | Outcome::OkWithWarnings) {
                let _ = tx.send(Event::Finished);
                return;
            }
        }
        if want_link {
            if want_asm {
                log(LogKind::Info, format!("{} '{}'", t("IDE.BuildObjectFileToExe"), req.tlink_args.trim()));
            }
            // Salida de TASM de los módulos secundarios (sólo si hubo algo que ver).
            let mut mod_failed = false;
            for (i, m) in modules.iter().enumerate() {
                let mlog = out.files.get(&format!("MOD{}.LOG", i + 1)).cloned().unwrap_or_default();
                if log_has_errors(&mlog) || log_has_warnings(&mlog) {
                    send_tool_log(&tx, &mlog);
                }
                if log_has_errors(&mlog) {
                    mod_failed = true;
                    log(LogKind::Error, t("Port.ModuleFailed").replace("{file}", m));
                    for h in explain(&mlog, &t) {
                        log(LogKind::Error, h);
                    }
                }
            }
            let llog = stage.restore_names(&out.files.get("LNK.LOG").cloned().unwrap_or_default());
            send_tool_log(&tx, &llog);
            let rc = out.files.get("LNK.RC").map(|s| s.trim().to_string());
            let outcome = match rc.as_deref() {
                None => {
                    log(LogKind::Error, t("IDE.BuildFailedDueToSubProcess"));
                    Outcome::SubProcessFailed
                }
                Some("OK") if !log_has_errors(&llog) && !mod_failed => {
                    log(LogKind::Success, t("IDE.BuildSucessful"));
                    Outcome::Ok
                }
                _ => {
                    for h in explain(&llog, &t) {
                        log(LogKind::Error, h);
                    }
                    log(LogKind::Error, t("IDE.BuildFailedWithError"));
                    Outcome::Failed
                }
            };
            let _ = tx.send(Event::StepDone { step: Step::Build, outcome, source: src.clone(), text_hash: req.text_hash });
            if outcome != Outcome::Ok {
                let _ = tx.send(Event::Finished);
                return;
            }
        }
        log(LogKind::Info, format!("{} {:.2} s", t("Port.Elapsed"), started.elapsed().as_secs_f32()));
    }

    // --- Ejecutar (ventana DOSBox visible) ---
    if req.steps.contains(&Step::Run) {
        let target = ["EXE", "COM"]
            .iter()
            .find_map(|ext| find_ci(&work, &format!("{name}.{ext}")).map(|_| format!("{name}.{ext}")));
        let Some(target) = target else {
            log(LogKind::Error, format!("{}: {}.EXE", t("IDE.RunFailed"), dos_stem(&src)));
            let _ = tx.send(Event::StepDone { step: Step::Run, outcome: Outcome::Failed, source: src.clone(), text_hash: req.text_hash });
            let _ = tx.send(Event::Finished);
            return;
        };
        let mut b = vec![target.clone()];
        if req.use_launcher {
            log(LogKind::Info, format!("{} {target}", t("IDE.RunExecutableWithLauncher")));
            b.push("ECHO.".into());
            b.push("ECHO.".into());
            b.push(format!("ECHO {}", req.launcher_msg));
            b.push("PAUSE >NUL".into());
        } else {
            log(LogKind::Info, format!("{} {target}", t("IDE.RunExecutable")));
        }
        b.push("ECHO OK> E:\\RUN.RC".into());
        let res = dosbox.run(&job(&req, &work, b, true), &cancel);
        stage.sync_back();
        let outcome = match res {
            Ok(o) if o.cancelled => {
                log(LogKind::Warning, t("IDE.RunAbortByUser"));
                Outcome::Aborted
            }
            Ok(o) if o.files.contains_key("RUN.RC") => {
                log(LogKind::Success, t("IDE.RunSucessful"));
                Outcome::Ok
            }
            Ok(_) => {
                // Se cerró la ventana de DOSBox antes de terminar.
                log(LogKind::Warning, t("IDE.RunAbortByUser"));
                Outcome::Aborted
            }
            Err(e) => {
                log(LogKind::Error, format!("{} ({e})", t("IDE.RunFailedDueToSubProcess")));
                Outcome::SubProcessFailed
            }
        };
        let _ = tx.send(Event::StepDone { step: Step::Run, outcome, source: src.clone(), text_hash: req.text_hash });
    }

    // --- Depurar con Turbo Debugger ---
    if req.steps.contains(&Step::Debug) {
        let exe = format!("{name}.EXE");
        if find_ci(&work, &exe).is_none() {
            log(LogKind::Error, format!("{}: {}.EXE", t("IDE.DebugFailed"), dos_stem(&src)));
        } else {
            log(LogKind::Info, format!("{} TD {exe}", t("IDE.DebugExecutable")));
            let b = vec![format!("TD {exe}"), "ECHO OK> E:\\DBG.RC".into()];
            let res = dosbox.run(&job(&req, &work, b, true), &cancel);
            stage.sync_back();
            let outcome = match res {
                Ok(o) if o.files.contains_key("DBG.RC") => {
                    log(LogKind::Success, t("IDE.DebugSucessful"));
                    Outcome::Ok
                }
                Ok(_) => {
                    log(LogKind::Warning, t("IDE.DebugAbortByUser"));
                    Outcome::Aborted
                }
                Err(e) => {
                    log(LogKind::Error, format!("{} ({e})", t("IDE.DebugFailedDueToSubProcess")));
                    Outcome::SubProcessFailed
                }
            };
            let _ = tx.send(Event::StepDone { step: Step::Debug, outcome, source: src.clone(), text_hash: req.text_hash });
        }
    }
    let _ = tx.send(Event::Finished);
}

/// Comandos de la terminal que producen texto y terminan solos: se ejecutan
/// sin ventana y su salida aparece en la terminal. El resto (TD, programas del
/// usuario...) se abre en una ventana de DOSBox.
const CAPTURED: &[&str] = &[
    "TASM", "TLINK", "DIR", "TYPE", "ECHO", "DEL", "ERASE", "COPY", "REN", "RENAME", "MD", "MKDIR", "RD",
    "RMDIR", "VER", "SET", "PATH", "MEM", "TREE", "ATTRIB",
];

pub struct TerminalRequest {
    pub command: String,
    pub work_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub dosbox: DosBox,
    pub silent: bool,
    pub launcher_msg: String,
}

pub fn execute_terminal(req: TerminalRequest, tx: Sender<Event>, cancel: CancelHandle) {
    cancel.reset();
    let first = req.command.split_whitespace().next().unwrap_or("").to_uppercase();
    let first = first.trim_end_matches(".EXE").trim_end_matches(".COM").to_string();
    let captured = CAPTURED.contains(&first.as_str()) && !req.command.contains('>');
    let mut batch = Vec::new();
    if captured {
        batch.push(format!("{} > E:\\OUT.LOG", req.command));
    } else {
        batch.push(req.command.clone());
        batch.push("ECHO.".into());
        batch.push(format!("ECHO {}", req.launcher_msg));
        batch.push("PAUSE >NUL".into());
    }
    batch.push("ECHO OK> E:\\DONE.RC".into());
    let mut visible = !captured || !req.silent;
    loop {
        let job = Job {
            bin_dir: req.bin_dir.clone(),
            work_dir: req.work_dir.clone(),
            batch: batch.clone(),
            visible,
            timeout: if visible { None } else { Some(Duration::from_secs(60)) },
        };
        match req.dosbox.run(&job, &cancel) {
            Ok(o) => {
                if !visible && !o.cancelled && !o.files.contains_key("DONE.RC") && !o.timed_out {
                    let _ = tx.send(Event::SilentUnsupported);
                    visible = true;
                    continue;
                }
                if let Some(out) = o.files.get("OUT.LOG") {
                    let _ = tx.send(Event::TerminalOutput(out.trim_end().to_string()));
                }
                if o.timed_out {
                    let _ = tx.send(Event::TerminalOutput("[timeout]".into()));
                }
            }
            Err(e) => {
                let _ = tx.send(Event::TerminalOutput(format!("[DOSBox] {e}")));
            }
        }
        break;
    }
    let _ = tx.send(Event::Finished);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tasm_errors() {
        let log = "Turbo Assembler  Version 4.1  Copyright (c) 1988, 1996 Borland International\n\
Assembling file:   MAIN.ASM\n\
**Error** MAIN.ASM(12) Undefined symbol: FOO\n\
*Warning* MAIN.ASM(7) Argument needs type override\n\
**Error** MAIN.ASM(20) PRINT(3) Illegal instruction\n\
Error messages:    2\n";
        let d = parse_tasm(log);
        assert_eq!(d.len(), 3);
        assert_eq!(d[0].line, 12);
        assert_eq!(d[0].severity, Severity::Error);
        assert_eq!(d[1].severity, Severity::Warning);
        assert_eq!(d[2].line, 20);
        assert!(d[2].message.starts_with("[PRINT(3)]"));
        assert!(log_has_errors(log));
    }

    fn manual(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/Ejemplos_Manual").join(name)
    }

    #[test]
    fn classifies_manual_examples() {
        let kind = |n: &str| source_kind(&manual(n), &read_lossy(&manual(n)));
        assert_eq!(kind("PRINCIPA.ASM"), SourceKind::Program);
        assert_eq!(kind("MODULO.ASM"), SourceKind::Module);
        assert_eq!(kind("ROTA_N.INC"), SourceKind::Include);
        assert_eq!(kind("BIOS_MAC.INC"), SourceKind::Include);
        for f in std::fs::read_dir(manual("")).unwrap().flatten() {
            let n = f.file_name().to_string_lossy().to_string();
            if n.ends_with(".ASM") && n != "MODULO.ASM" {
                assert_eq!(kind(&n), SourceKind::Program, "{n}");
            }
        }
        assert_eq!(linked_modules(&manual("PRINCIPA.ASM"), &read_lossy(&manual("PRINCIPA.ASM"))), vec!["MODULO.ASM"]);
        assert!(linked_modules(&manual("HOLA.ASM"), "").is_empty());
        assert_eq!(programs_using_module(&manual("MODULO.ASM")), vec!["PRINCIPA.ASM"]);
        assert_eq!(programs_including(&manual("ROTA_N.INC")), vec!["ROTACIO3.ASM"]);
        assert_eq!(programs_including(&manual("BIOS_MAC.INC")), vec!["PRUBIOS.ASM"]);
        assert_eq!(programs_including(&manual("DOS_MAC.INC")), vec!["PRUDOS.ASM"]);
    }

    #[test]
    fn explains_common_errors() {
        let t = |k: &str| k.to_string();
        assert_eq!(explain("Fatal: No program entry point", &t), vec!["Port.HintNoEntry"]);
        assert_eq!(explain("Error: Undefined symbol PROC2 in module PRINCIPA.ASM", &t), vec!["Port.HintUndefined"]);
        assert!(explain("Warning: DOSSEG directive ignored in module X", &t).is_empty());
    }

    #[test]
    fn statements_ignore_comments_and_labels() {
        let s = "inicio: mov ax, 1 ; end x\n; end\nfin:\n   end   ; comentario\n";
        assert_eq!(source_kind(Path::new("A.ASM"), s), SourceKind::Module);
        assert_eq!(source_kind(Path::new("A.ASM"), "        end     inicio\n"), SourceKind::Program);
        assert_eq!(extrns("extrn a: byte, b: word ; c\n"), vec!["A", "B"]);
    }
}

/// Prueba de punta a punta con DOSBox real (requiere DOSBox instalado):
/// `cargo test -- --ignored --nocapture`
/// Carpeta BIN para las pruebas con DOSBox: la del repositorio o, con
/// `RTASM_TEST_BIN`, la de una instalación (p. ej. una ruta con espacios).
#[cfg(test)]
fn test_bin() -> PathBuf {
    std::env::var_os("RTASM_TEST_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("BIN"))
}

#[cfg(test)]
mod dosbox_tests {
    use super::*;
    use std::sync::mpsc::channel;

    fn run(src_name: &str, code: &str, steps: Vec<Step>) -> (Vec<String>, Vec<(Step, Outcome)>, PathBuf) {
        let dosbox = crate::dosbox::detect("").expect("DOSBox no instalado");
        let bin = test_bin();
        let dir = std::env::temp_dir().join(format!("rtasm-test-{}-{src_name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join(src_name);
        std::fs::write(&src, code.replace('\n', "\r\n")).unwrap();
        let req = Request {
            steps,
            source: src,
            text_hash: 1,
            tasm_args: "/zi".into(),
            tlink_args: "/v".into(),
            use_launcher: false,
            silent: true,
            bin_dir: bin,
            dosbox,
            launcher_msg: "x".into(),
        };
        let (tx, rx) = channel();
        execute(req, tx, CancelHandle::default(), |k| k.to_string());
        let mut logs = Vec::new();
        let mut steps = Vec::new();
        for e in rx.try_iter() {
            match e {
                Event::Log(_, s) => logs.push(s),
                Event::StepDone { step, outcome, .. } => steps.push((step, outcome)),
                Event::SilentUnsupported => logs.push("SILENT-UNSUPPORTED".into()),
                _ => {}
            }
        }
        (logs, steps, dir)
    }

    #[test]
    #[ignore]
    fn assembles_and_links_hello() {
        let t = std::time::Instant::now();
        let (logs, steps, dir) = run("HOLA.ASM", include_str!("../examples/HOLA.ASM"), vec![Step::Assemble, Step::Build]);
        println!("{:.2}s\n{}", t.elapsed().as_secs_f32(), logs.join("\n"));
        assert_eq!(steps, vec![(Step::Assemble, Outcome::Ok), (Step::Build, Outcome::Ok)]);
        assert!(find_ci(&dir, "HOLA.EXE").is_some());
        assert!(find_ci(&dir, "HOLA.OBJ").is_some());
    }

    #[test]
    #[ignore]
    fn reports_errors_with_lines() {
        let code = "ideal\nmodel small\nstack 256\ncodeseg\ninicio: mov ax, FOO\n        movx bx, 1\n        mov ax, 04C00h\n        int 21h\n        end inicio\n";
        let (logs, steps, _) = run("MALO.ASM", code, vec![Step::Assemble, Step::Build]);
        println!("{}", logs.join("\n"));
        assert_eq!(steps, vec![(Step::Assemble, Outcome::Failed)]);
        let diags: Vec<Diagnostic> = logs.iter().flat_map(|l| parse_tasm(l)).collect();
        assert!(diags.iter().any(|d| d.line == 5), "{diags:?}");
        assert!(diags.iter().any(|d| d.line == 6), "{diags:?}");
    }

    /// Nombre largo y con acento: antes TASM respondía "Can't locate file".
    #[test]
    #[ignore]
    fn long_name_with_accents() {
        let (logs, steps, dir) = run("mi programa_año.asm", include_str!("../examples/HOLA.ASM"), vec![Step::Assemble, Step::Build]);
        println!("{}", logs.join("\n"));
        assert_eq!(steps, vec![(Step::Assemble, Outcome::Ok), (Step::Build, Outcome::Ok)]);
        assert!(find_ci(&dir, "MI PROGRAMA_AÑO.EXE").is_some(), "{:?}", std::fs::read_dir(&dir).unwrap().collect::<Vec<_>>());
        assert!(logs.iter().any(|l| l.contains("Port.DosNameWarning")));
        // Los mensajes de TASM muestran el nombre original (para el clic).
        assert!(logs.iter().any(|l| l.contains("mi programa_año.asm")));
    }

    /// PRINCIPA se enlaza con MODULO; MODULO solo explica el error.
    #[test]
    #[ignore]
    fn links_principal_with_module() {
        let dosbox = crate::dosbox::detect("").expect("DOSBox no instalado");
        let dir = std::env::temp_dir().join(format!("rtasm-mod-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for f in ["PRINCIPA.ASM", "MODULO.ASM"] {
            std::fs::copy(Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/Ejemplos_Manual").join(f), dir.join(f)).unwrap();
        }
        let go = |f: &str| {
            let req = Request {
                steps: vec![Step::Assemble, Step::Build],
                source: dir.join(f),
                text_hash: 1,
                tasm_args: "/zi".into(),
                tlink_args: "/v".into(),
                use_launcher: false,
                silent: true,
                bin_dir: test_bin(),
                dosbox: dosbox.clone(),
                launcher_msg: "x".into(),
            };
            let (tx, rx) = channel();
            execute(req, tx, CancelHandle::default(), |k| k.to_string());
            let mut logs = Vec::new();
            let mut steps = Vec::new();
            for e in rx.try_iter() {
                match e {
                    Event::Log(_, s) => logs.push(s),
                    Event::StepDone { step, outcome, .. } => steps.push((step, outcome)),
                    _ => {}
                }
            }
            println!("{}", logs.join("\n"));
            (logs, steps)
        };
        let (logs, steps) = go("PRINCIPA.ASM");
        assert_eq!(steps, vec![(Step::Assemble, Outcome::Ok), (Step::Build, Outcome::Ok)]);
        assert!(logs.iter().any(|l| l.contains("Port.LinkWithModules") && l.contains("MODULO.ASM")));
        assert!(find_ci(&dir, "PRINCIPA.EXE").is_some());
        assert!(find_ci(&dir, "MODULO.OBJ").is_some());
        let (logs, steps) = go("MODULO.ASM");
        assert_eq!(steps, vec![(Step::Assemble, Outcome::Ok), (Step::Build, Outcome::Failed)]);
        assert!(logs.iter().any(|l| l == "Port.HintNoEntry"));
    }
}

#[cfg(test)]
mod run_tests {
    use super::*;
    use crate::dosbox::Job;

    #[test]
    #[ignore]
    fn exe_runs_and_td_starts() {
        let dosbox = crate::dosbox::detect("").unwrap();
        let bin = test_bin();
        let dir = std::env::temp_dir().join(format!("rtasm-run-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("HOLA.ASM"), include_str!("../examples/HOLA.ASM").replace('\n', "\r\n")).unwrap();
        let job = |batch: Vec<&str>, secs| Job {
            bin_dir: bin.clone(),
            work_dir: dir.clone(),
            batch: batch.into_iter().map(String::from).collect(),
            visible: false,
            timeout: Some(Duration::from_secs(secs)),
        };
        let c = CancelHandle::default();
        let o = dosbox.run(&job(vec!["TASM /zi HOLA.ASM > NUL", "TLINK /v HOLA.OBJ > NUL", "HOLA.EXE > E:\\OUT.LOG"], 30), &c).unwrap();
        let out = o.files.get("OUT.LOG").cloned().unwrap_or_default();
        println!("programa: {out:?}");
        assert!(out.contains("Hola, mundo"));
        // TD es interactivo: si sigue vivo a los 6 s, arrancó bien (no hubo error DPMI).
        let o = dosbox.run(&job(vec!["TD HOLA.EXE > E:\\TD.LOG", "ECHO FIN> E:\\TD.RC"], 6), &c).unwrap();
        println!("TD timed_out={} log={:?} rc={:?}", o.timed_out, o.files.get("TD.LOG"), o.files.get("TD.RC"));
        assert!(o.timed_out, "TD terminó solo: {:?}", o.files.get("TD.LOG"));
    }
}
