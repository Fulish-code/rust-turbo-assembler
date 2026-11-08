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

fn job(req: &Request, batch: Vec<String>, visible: bool) -> Job {
    Job {
        bin_dir: req.bin_dir.clone(),
        work_dir: req.source.parent().map(Path::to_path_buf).unwrap_or_else(|| ".".into()),
        batch,
        visible,
        timeout: if visible { None } else { Some(Duration::from_secs(120)) },
    }
}

/// Lote para TASM y, opcionalmente, TLINK en una sola sesión de DOSBox
/// (arrancar DOSBox una sola vez es lo más rápido).
fn assemble_link_batch(req: &Request, assemble: bool, link: bool) -> Vec<String> {
    let name = dos_stem(&req.source);
    let mut b = Vec::new();
    if assemble {
        b.push(format!("TASM {} {}.ASM > E:\\ASM.LOG", req.tasm_args.trim(), name));
        b.push("IF ERRORLEVEL 1 GOTO ASMERR".into());
        b.push("ECHO OK> E:\\ASM.RC".into());
    }
    if link {
        b.push(format!("TLINK {} {}.OBJ > E:\\LNK.LOG", req.tlink_args.trim(), name));
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
    batch: Vec<String>,
    marker: &str,
    tx: &Sender<Event>,
    cancel: &CancelHandle,
) -> Result<crate::dosbox::JobOutput, String> {
    let mut visible = !req.silent;
    loop {
        let out = dosbox.run(&job(req, batch.clone(), visible), cancel).map_err(|e| e.to_string())?;
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
    let dir = src.parent().map(Path::to_path_buf).unwrap_or_default();
    let name = dos_stem(&src);
    let log = |k: LogKind, s: String| {
        let _ = tx.send(Event::Log(k, s));
    };

    let want_asm = req.steps.contains(&Step::Assemble);
    let want_link = req.steps.contains(&Step::Build);

    // --- Ensamblar y/o enlazar (silencioso) ---
    if want_asm || want_link {
        let started = Instant::now();
        if want_asm {
            log(LogKind::Info, format!("{} '{}' : {}", t("IDE.AssembleToObjectFile"), req.tasm_args.trim(), src.display()));
        }
        if want_link && !want_asm {
            log(LogKind::Info, format!("{} '{}'", t("IDE.BuildObjectFileToExe"), req.tlink_args.trim()));
        }
        let marker = if want_asm { "ASM.RC" } else { "LNK.RC" };
        let out = match run_tool_job(&dosbox, &req, assemble_link_batch(&req, want_asm, want_link), marker, &tx, &cancel) {
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
            let alog = out.files.get("ASM.LOG").cloned().unwrap_or_default();
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
            let llog = out.files.get("LNK.LOG").cloned().unwrap_or_default();
            send_tool_log(&tx, &llog);
            let rc = out.files.get("LNK.RC").map(|s| s.trim().to_string());
            let outcome = match rc.as_deref() {
                None => {
                    log(LogKind::Error, t("IDE.BuildFailedDueToSubProcess"));
                    Outcome::SubProcessFailed
                }
                Some("OK") if !log_has_errors(&llog) => {
                    log(LogKind::Success, t("IDE.BuildSucessful"));
                    Outcome::Ok
                }
                _ => {
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
            .find_map(|ext| find_ci(&dir, &format!("{name}.{ext}")).map(|_| format!("{name}.{ext}")));
        let Some(target) = target else {
            log(LogKind::Error, format!("{}: {name}.EXE", t("IDE.RunFailed")));
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
        let res = dosbox.run(&job(&req, b, true), &cancel);
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
        if find_ci(&dir, &exe).is_none() {
            log(LogKind::Error, format!("{}: {exe}", t("IDE.DebugFailed")));
        } else {
            log(LogKind::Info, format!("{} TD {exe}", t("IDE.DebugExecutable")));
            let b = vec![format!("TD {exe}"), "ECHO OK> E:\\DBG.RC".into()];
            let outcome = match dosbox.run(&job(&req, b, true), &cancel) {
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
