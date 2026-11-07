//! Backend DOSBox: equivalente multiplataforma de `guiTasmX64.exe` + `runner`
//! del GUI Turbo Assembler original. Monta la carpeta BIN como `C:`, la
//! carpeta del código como `D:` y una carpeta temporal de intercambio como `E:`
//! (ahí quedan las bitácoras y los códigos de salida).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flavor {
    Staging,
    X,
    Classic,
}

impl Flavor {
    pub fn label(self) -> &'static str {
        match self {
            Flavor::Staging => "DOSBox Staging",
            Flavor::X => "DOSBox-X",
            Flavor::Classic => "DOSBox",
        }
    }
}

#[derive(Clone, Debug)]
pub struct DosBox {
    pub exe: PathBuf,
    pub flavor: Flavor,
    pub version: String,
}

fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        for cand in [name.to_string(), format!("{name}.exe")] {
            let p = dir.join(&cand);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

fn extra_locations() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(home) = dirs::home_dir() {
        // Instalaciones portables comunes en Linux.
        for base in [home.join(".local/opt"), home.join("Applications"), home.join("opt")] {
            if let Ok(rd) = std::fs::read_dir(&base) {
                for e in rd.flatten() {
                    let n = e.file_name().to_string_lossy().to_lowercase();
                    if n.starts_with("dosbox") {
                        for exe in ["dosbox", "dosbox-x", "dosbox.exe"] {
                            v.push(e.path().join(exe));
                        }
                    }
                }
            }
        }
    }
    // macOS
    v.push("/Applications/DOSBox Staging.app/Contents/MacOS/dosbox".into());
    v.push("/Applications/dosbox-staging.app/Contents/MacOS/dosbox".into());
    v.push("/Applications/dosbox-x.app/Contents/MacOS/dosbox-x".into());
    v.push("/Applications/DOSBox.app/Contents/MacOS/DOSBox".into());
    // Windows
    for pf in ["C:\\Program Files", "C:\\Program Files (x86)"] {
        v.push(format!("{pf}\\DOSBox Staging\\dosbox.exe").into());
        v.push(format!("{pf}\\DOSBox-X\\dosbox-x.exe").into());
        v.push(format!("{pf}\\DOSBox-0.74-3\\DOSBox.exe").into());
    }
    // Junto al propio ejecutable (distribución "todo en uno").
    if let Ok(me) = std::env::current_exe() {
        if let Some(dir) = me.parent() {
            for exe in ["dosbox/dosbox", "dosbox/dosbox.exe", "dosbox-staging/dosbox"] {
                v.push(dir.join(exe));
            }
        }
    }
    v
}

fn probe(exe: &Path) -> Option<DosBox> {
    if !exe.is_file() {
        return None;
    }
    let out = Command::new(exe)
        .arg("--version")
        .stdin(Stdio::null())
        .env("SDL_VIDEODRIVER", "dummy")
        .env("SDL_AUDIODRIVER", "dummy")
        .output()
        .ok();
    let text = out
        .map(|o| format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
        .unwrap_or_default();
    let low = text.to_lowercase();
    let name = exe.file_name().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default();
    let flavor = if low.contains("staging") || name.contains("staging") {
        Flavor::Staging
    } else if low.contains("dosbox-x") || name.contains("dosbox-x") {
        Flavor::X
    } else {
        Flavor::Classic
    };
    let version = text
        .lines()
        .map(str::trim)
        .find(|l| l.to_lowercase().contains("dosbox") || l.to_lowercase().contains("version"))
        .unwrap_or("")
        .chars()
        .take(80)
        .collect();
    Some(DosBox { exe: exe.to_path_buf(), flavor, version })
}

/// Busca DOSBox: primero la ruta configurada, luego el PATH y ubicaciones típicas.
pub fn detect(custom: &str) -> Option<DosBox> {
    let custom = custom.trim();
    if !custom.is_empty() {
        let p = PathBuf::from(custom);
        if let Some(d) = probe(&p) {
            return Some(d);
        }
        if let Some(d) = which(custom).and_then(|p| probe(&p)) {
            return Some(d);
        }
    }
    for name in ["dosbox-staging", "dosbox-x", "dosbox"] {
        if let Some(d) = which(name).and_then(|p| probe(&p)) {
            return Some(d);
        }
    }
    extra_locations().iter().find_map(|p| probe(p))
}

/// Un trabajo para DOSBox: una lista de comandos DOS ejecutados con `D:` como
/// directorio actual.
pub struct Job {
    pub bin_dir: PathBuf,
    pub work_dir: PathBuf,
    /// Contenido de `E:\JOB.BAT` (sin `@ECHO OFF`, se añade solo).
    pub batch: Vec<String>,
    /// `false` = silencioso (sin ventana, sin sonido).
    pub visible: bool,
    pub timeout: Option<Duration>,
}

pub struct JobOutput {
    /// Archivos que el lote escribió en `E:` (nombre en MAYÚSCULAS → texto).
    pub files: HashMap<String, String>,
    pub cancelled: bool,
    pub timed_out: bool,
    pub stderr: String,
}

/// Permite cancelar el trabajo en curso desde la interfaz.
#[derive(Clone, Default)]
pub struct CancelHandle {
    flag: Arc<AtomicBool>,
    child: Arc<Mutex<Option<Child>>>,
}

impl CancelHandle {
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
        if let Ok(mut c) = self.child.lock() {
            if let Some(ch) = c.as_mut() {
                let _ = ch.kill();
            }
        }
    }
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
    pub fn reset(&self) {
        self.flag.store(false, Ordering::SeqCst);
    }
}

static JOB_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Decodifica la salida de DOS (página de códigos 437) a texto.
pub fn decode_cp437(bytes: &[u8]) -> String {
    const HI: &str = "ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáíóúñÑªº¿⌐¬½¼¡«»░▒▓│┤╡╢╖╕╣║╗╝╜╛┐└┴┬├─┼╞╟╚╔╩╦╠═╬╧╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀αßΓπΣσµτΦΘΩδ∞φε∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■\u{a0}";
    let hi: Vec<char> = HI.chars().collect();
    let mut s = String::with_capacity(bytes.len());
    for &b in bytes {
        match b {
            b'\r' | 0x1a => {}
            0x00..=0x7f => s.push(b as char),
            _ => s.push(hi.get((b - 0x80) as usize).copied().unwrap_or('?')),
        }
    }
    s
}

/// Texto Unicode → bytes DOS seguros para un .BAT (los acentos se simplifican).
fn to_dos_ascii(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'Á' => 'A',
            'É' => 'E',
            'Í' => 'I',
            'Ó' => 'O',
            'Ú' => 'U',
            'ñ' => 'n',
            'Ñ' => 'N',
            c if c.is_ascii() => c,
            _ => '?',
        })
        .collect()
}

/// Ruta que DOSBox pueda montar sin problemas: en Unix se usa un enlace
/// simbólico ASCII dentro de la carpeta temporal (evita espacios/acentos).
fn mount_path(tmp: &Path, alias: &str, real: &Path) -> PathBuf {
    #[cfg(unix)]
    {
        let link = tmp.join(alias);
        if std::os::unix::fs::symlink(real, &link).is_ok() {
            return link;
        }
    }
    let _ = (tmp, alias);
    real.to_path_buf()
}

impl DosBox {
    fn base_args(&self) -> Vec<String> {
        match self.flavor {
            Flavor::Staging => vec!["--noprimaryconf".into(), "--nolocalconf".into()],
            Flavor::X => vec!["-nopromptfolder".into(), "-fastlaunch".into()],
            Flavor::Classic => vec![],
        }
    }

    fn conf(&self, job: &Job, c: &Path, d: &Path, e: &Path) -> String {
        let mut s = String::new();
        s.push_str("[sdl]\nfullscreen=false\nautolock=false\n");
        if job.visible {
            if self.flavor == Flavor::Staging {
                s.push_str("windowresolution=default\n");
            }
        } else if self.flavor == Flavor::Staging {
            s.push_str("output=texture\n");
        }
        s.push_str("\n[dosbox]\nmemsize=16\n");
        if self.flavor == Flavor::Staging {
            s.push_str("startup_verbosity=quiet\n");
        }
        s.push_str("\n[cpu]\ncore=auto\ncputype=auto\ncycles=max\n");
        s.push_str(&format!("\n[mixer]\nnosound={}\n", if job.visible { "false" } else { "true" }));
        s.push_str("\n[autoexec]\n@ECHO OFF\n");
        for (drive, p) in [("C", c), ("D", d), ("E", e)] {
            s.push_str(&format!("MOUNT {drive} \"{}\" >NUL\n", p.display()));
        }
        s.push_str("PATH Z:\\;C:\\\nSET TEMP=E:\\\nD:\nCLS\nCALL E:\\JOB.BAT\nEXIT\n");
        s
    }

    /// Ejecuta el trabajo y espera a que termine (llamar desde un hilo de fondo).
    pub fn run(&self, job: &Job, cancel: &CancelHandle) -> std::io::Result<JobOutput> {
        let n = JOB_COUNTER.fetch_add(1, Ordering::SeqCst);
        let tmp = std::env::temp_dir().join(format!("rust-tasm-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let io_dir = tmp.join("io");
        std::fs::create_dir_all(&io_dir)?;

        let mut bat = String::from("@ECHO OFF\r\n");
        for line in &job.batch {
            bat.push_str(&to_dos_ascii(line));
            bat.push_str("\r\n");
        }
        std::fs::write(io_dir.join("JOB.BAT"), bat)?;

        let c = mount_path(&tmp, "c", &job.bin_dir);
        let d = mount_path(&tmp, "d", &job.work_dir);
        let conf_path = tmp.join("dosbox.conf");
        std::fs::write(&conf_path, self.conf(job, &c, &d, &io_dir))?;

        let mut cmd = Command::new(&self.exe);
        cmd.args(self.base_args())
            .arg("-conf")
            .arg(&conf_path)
            .arg("-exit")
            .current_dir(&tmp)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        if !job.visible {
            cmd.env("SDL_VIDEODRIVER", "dummy").env("SDL_AUDIODRIVER", "dummy");
        }
        let child = cmd.spawn()?;
        let stderr_pipe = {
            let mut guard = cancel.child.lock().unwrap();
            *guard = Some(child);
            guard.as_mut().and_then(|c| c.stderr.take())
        };
        // Leer stderr en otro hilo para que el búfer no se llene.
        let err_thread = stderr_pipe.map(|mut p| {
            std::thread::spawn(move || {
                let mut buf = Vec::new();
                let _ = std::io::Read::read_to_end(&mut p, &mut buf);
                String::from_utf8_lossy(&buf).into_owned()
            })
        });

        let start = Instant::now();
        let mut timed_out = false;
        let _status = loop {
            {
                let mut guard = cancel.child.lock().unwrap();
                let Some(ch) = guard.as_mut() else { break None };
                match ch.try_wait() {
                    Ok(Some(st)) => break Some(st),
                    Ok(None) => {}
                    Err(_) => break None,
                }
                if cancel.is_cancelled() {
                    let _ = ch.kill();
                    let _ = ch.wait();
                    break None;
                }
                if let Some(t) = job.timeout {
                    if start.elapsed() > t {
                        timed_out = true;
                        let _ = ch.kill();
                        let _ = ch.wait();
                        break None;
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(15));
        };
        *cancel.child.lock().unwrap() = None;
        let stderr = err_thread.and_then(|t| t.join().ok()).unwrap_or_default();

        let mut files = HashMap::new();
        if let Ok(rd) = std::fs::read_dir(&io_dir) {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_uppercase();
                if name == "JOB.BAT" {
                    continue;
                }
                if let Ok(bytes) = std::fs::read(e.path()) {
                    files.insert(name, decode_cp437(&bytes));
                }
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);
        Ok(JobOutput {
            files,
            // El código de salida de DOSBox no refleja el del programa DOS;
            // por eso se usan los archivos *.RC del lote.
            cancelled: cancel.is_cancelled(),
            timed_out,
            stderr,
        })
    }
}
