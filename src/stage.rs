//! Carpeta de trabajo para DOS.
//!
//! TASM, TLINK y TD son programas DOS: sólo entienden nombres 8.3
//! (`PROGRAMA.ASM`) y la página de códigos 437. Los archivos del usuario
//! pueden tener nombres largos (`programa1.asm`) y estar en UTF-8 (acentos en
//! los comentarios), así que cada tarea trabaja sobre una copia en una carpeta
//! temporal:
//!
//! * el archivo activo se copia con un nombre DOS válido (su *alias*);
//! * los demás archivos con nombre DOS válido (módulos, `.INC`, `.OBJ`...) se
//!   copian con su nombre, para que funcionen `include` y el enlazado;
//! * los fuentes se convierten de UTF-8 a la página 437 (así TD muestra bien
//!   «CÓDIGO» en lugar de «C├│DIGO»);
//! * al terminar, lo que TASM/TLINK/el programa crearon o cambiaron vuelve a
//!   la carpeta del usuario, con el nombre original (`PROGRAMA1.EXE`).
//!
//! Los archivos del usuario nunca se modifican.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

/// Extensiones de código fuente: se convierten a la página 437.
const SOURCE_EXT: &[&str] = &["ASM", "INC", "MAC", "EQU", "DEF"];
/// Archivos más grandes no se copian (la carpeta puede ser, p. ej., Documentos).
const MAX_FILE: u64 = 4 * 1024 * 1024;
const MAX_FILES: usize = 400;

static STAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct Stage {
    /// Carpeta temporal que se monta como D:.
    pub dir: PathBuf,
    src_dir: PathBuf,
    /// Nombre DOS (sin extensión) del archivo activo, p. ej. `PROGRAMA`.
    pub alias: String,
    /// Nombre original (sin extensión) en mayúsculas, p. ej. `PROGRAMA1`.
    stem: String,
    /// Nombre de archivo original del activo, p. ej. `programa1.asm`.
    pub source_name: String,
    /// Tamaño y fecha de cada archivo copiado, para saber qué cambió.
    snapshot: HashMap<String, (u64, Option<SystemTime>)>,
}

/// ¿Es un nombre DOS 8.3 válido (y en ASCII)?
pub fn is_dos_name(name: &str) -> bool {
    let (stem, ext) = match name.rsplit_once('.') {
        Some((s, e)) => (s, e),
        None => (name, ""),
    };
    let ok = |s: &str| s.chars().all(|c| c.is_ascii_alphanumeric() || "_-~!#$%&'(){}@^`".contains(c));
    !stem.is_empty() && stem.len() <= 8 && ext.len() <= 3 && ok(stem) && ok(ext)
}

/// Nombre DOS para un nombre largo: letras y números, máximo 8 caracteres.
/// `taken` son los nombres (sin extensión, en mayúsculas) que ya existen.
pub fn dos_alias(stem: &str, taken: &[String]) -> String {
    let upper = stem.to_uppercase();
    if is_dos_name(&upper) && !upper.contains('.') {
        return upper;
    }
    let clean: String = crate::dosbox::to_dos_ascii(&upper)
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    let clean = if clean.is_empty() { "PROGRAMA".to_string() } else { clean };
    let base: String = clean.chars().take(8).collect();
    if !taken.contains(&base) {
        return base;
    }
    let short: String = clean.chars().take(6).collect();
    (1..100).map(|n| format!("{short}~{n}")).find(|a| !taken.contains(a)).unwrap_or(base)
}

/// UTF-8 → página de códigos 437. Si el archivo no es UTF-8 válido (ya está
/// en una página de DOS), se deja tal cual.
pub fn to_cp437(bytes: &[u8]) -> Vec<u8> {
    let Ok(text) = std::str::from_utf8(bytes) else { return bytes.to_vec() };
    if text.is_ascii() {
        return bytes.to_vec();
    }
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let table: HashMap<char, u8> = crate::dosbox::CP437_HIGH.chars().enumerate().map(|(i, c)| (c, 0x80 + i as u8)).collect();
    text.chars()
        .map(|c| {
            if c.is_ascii() {
                c as u8
            } else if let Some(b) = table.get(&c) {
                *b
            } else {
                crate::dosbox::to_dos_ascii(&c.to_string()).bytes().next().unwrap_or(b'?')
            }
        })
        .collect()
}

fn ext_upper(name: &str) -> String {
    name.rsplit_once('.').map(|(_, e)| e.to_uppercase()).unwrap_or_default()
}

fn stem_upper(name: &str) -> String {
    name.rsplit_once('.').map(|(s, _)| s).unwrap_or(name).to_uppercase()
}

fn file_state(p: &Path) -> Option<(u64, Option<SystemTime>)> {
    let m = std::fs::metadata(p).ok()?;
    m.is_file().then(|| (m.len(), m.modified().ok()))
}

impl Stage {
    pub fn new(source: &Path) -> std::io::Result<Stage> {
        let src_dir = source.parent().map(Path::to_path_buf).unwrap_or_else(|| ".".into());
        let source_name = source.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let stem = stem_upper(&source_name);

        let mut entries: Vec<(String, PathBuf)> = std::fs::read_dir(&src_dir)?
            .flatten()
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
            .collect();
        entries.sort();
        let taken: Vec<String> = entries
            .iter()
            .filter(|(n, _)| is_dos_name(n) && !n.eq_ignore_ascii_case(&source_name))
            .map(|(n, _)| stem_upper(n))
            .collect();
        let alias = dos_alias(&stem, &taken);

        let n = STAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("rust-tasm-d-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir)?;
        let mut stage = Stage { dir, src_dir, alias, stem, source_name, snapshot: HashMap::new() };

        let mut copied = 0;
        for (name, path) in &entries {
            let target = if name.eq_ignore_ascii_case(&stage.source_name) {
                // El archivo activo siempre es .ASM para TASM (aunque sea .INC).
                format!("{}.ASM", stage.alias)
            } else if stem_upper(name) == stage.stem && !ext_upper(name).is_empty() {
                // Salidas anteriores del activo (PROGRAMA1.OBJ/.EXE...).
                if SOURCE_EXT.contains(&ext_upper(name).as_str()) {
                    continue;
                }
                format!("{}.{}", stage.alias, ext_upper(name))
            } else if is_dos_name(name) && stem_upper(name) != stage.alias {
                name.to_uppercase()
            } else {
                continue;
            };
            let Some((len, _)) = file_state(path) else { continue };
            if len > MAX_FILE || copied >= MAX_FILES {
                continue;
            }
            let to = stage.dir.join(&target);
            if SOURCE_EXT.contains(&ext_upper(&target).as_str()) {
                std::fs::write(&to, to_cp437(&std::fs::read(path)?))?;
            } else {
                std::fs::copy(path, &to)?;
            }
            copied += 1;
        }
        stage.take_snapshot();
        Ok(stage)
    }

    /// ¿El nombre DOS es distinto del original? (nombre largo, espacios...)
    pub fn renamed(&self) -> bool {
        format!("{}.ASM", self.alias) != self.source_name.to_uppercase()
    }

    pub fn dos_source(&self) -> String {
        format!("{}.ASM", self.alias)
    }

    fn take_snapshot(&mut self) {
        self.snapshot.clear();
        if let Ok(rd) = std::fs::read_dir(&self.dir) {
            for e in rd.flatten() {
                if let Some(st) = file_state(&e.path()) {
                    self.snapshot.insert(e.file_name().to_string_lossy().to_uppercase(), st);
                }
            }
        }
    }

    /// Copia a la carpeta del usuario lo que se creó o cambió en la carpeta
    /// de trabajo (sin tocar los fuentes) y devuelve los nombres copiados.
    pub fn sync_back(&mut self) -> Vec<String> {
        let mut copied = Vec::new();
        let Ok(rd) = std::fs::read_dir(&self.dir) else { return copied };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_uppercase();
            let Some(st) = file_state(&e.path()) else { continue };
            if self.snapshot.get(&name) == Some(&st) {
                continue;
            }
            let ext = ext_upper(&name);
            let is_staged_source = self.snapshot.contains_key(&name) && SOURCE_EXT.contains(&ext.as_str());
            if is_staged_source || name == self.dos_source() {
                continue;
            }
            let real = if stem_upper(&name) == self.alias && !ext.is_empty() {
                format!("{}.{ext}", self.stem)
            } else {
                name.clone()
            };
            // Si ya existe con otras mayúsculas (hola.exe), se reemplaza ese.
            let to = crate::toolchain::find_ci(&self.src_dir, &real).unwrap_or_else(|| self.src_dir.join(&real));
            if std::fs::copy(e.path(), &to).is_ok() {
                copied.push(real);
            }
        }
        self.take_snapshot();
        copied
    }

    /// Cambia el nombre DOS del activo por el original en un texto
    /// (salida de TASM, diagnósticos) para que los clics lleven al archivo.
    pub fn restore_names(&self, text: &str) -> String {
        if !self.renamed() {
            return text.to_string();
        }
        let dos = self.dos_source();
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(i) = rest.to_ascii_uppercase().find(&dos) {
            // Sólo si es un nombre completo (no parte de otro).
            let before_ok = i == 0 || !rest.as_bytes()[i - 1].is_ascii_alphanumeric();
            out.push_str(&rest[..i]);
            out.push_str(if before_ok { &self.source_name } else { &rest[i..i + dos.len()] });
            rest = &rest[i + dos.len()..];
        }
        out.push_str(rest);
        out
    }
}

impl Drop for Stage {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dos_names() {
        assert!(is_dos_name("HOLA.ASM"));
        assert!(is_dos_name("mayor3_2.asm"));
        assert!(!is_dos_name("programa1.asm"));
        assert!(!is_dos_name("mi prog.asm"));
        assert!(!is_dos_name("año.asm"));
        assert_eq!(dos_alias("hola", &[]), "HOLA");
        assert_eq!(dos_alias("programa1", &[]), "PROGRAMA");
        assert_eq!(dos_alias("programa1", &["PROGRAMA".into()]), "PROGRA~1");
        assert_eq!(dos_alias("mi programa", &[]), "MIPROGRA");
        assert_eq!(dos_alias("año", &[]), "ANO");
    }

    #[test]
    fn converts_to_cp437() {
        assert_eq!(to_cp437("; CÓDIGO renglón".as_bytes()), b"; CODIGO rengl\xa2n".to_vec());
        assert_eq!(to_cp437(b"mov ax, 1"), b"mov ax, 1".to_vec());
        // Ya en una página de DOS (no es UTF-8): no se toca.
        assert_eq!(to_cp437(b"rengl\xa2n"), b"rengl\xa2n".to_vec());
    }

    #[test]
    fn stages_and_syncs_back() {
        let dir = std::env::temp_dir().join(format!("rtasm-stage-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("programa1.asm"), "; Código\r\n").unwrap();
        std::fs::write(dir.join("MACROS.INC"), "; macro\r\n").unwrap();
        std::fs::write(dir.join("notas largas.txt"), "x").unwrap();
        let mut st = Stage::new(&dir.join("programa1.asm")).unwrap();
        assert_eq!(st.alias, "PROGRAMA");
        assert!(st.renamed());
        assert_eq!(std::fs::read(st.dir.join("PROGRAMA.ASM")).unwrap(), b"; C\xa2digo\r\n");
        assert!(st.dir.join("MACROS.INC").exists());
        assert!(!st.dir.join("notas largas.txt").exists());
        std::fs::write(st.dir.join("PROGRAMA.EXE"), "MZ").unwrap();
        std::fs::write(st.dir.join("PROGRAMA.ASM"), "cambiado").unwrap();
        assert_eq!(st.sync_back(), vec!["PROGRAMA1.EXE".to_string()]);
        assert!(dir.join("PROGRAMA1.EXE").exists());
        assert_eq!(std::fs::read_to_string(dir.join("programa1.asm")).unwrap(), "; Código\r\n");
        assert_eq!(st.restore_names("**Error** PROGRAMA.ASM(3) x"), "**Error** programa1.asm(3) x");
        // Una segunda tarea encuentra el EXE con el nombre DOS.
        let st2 = Stage::new(&dir.join("programa1.asm")).unwrap();
        assert!(st2.dir.join("PROGRAMA.EXE").exists());
        drop((st, st2));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
