//! Localización: lee los mismos archivos `.lang` (JSON) del GUI Turbo Assembler
//! original. Busca primero en `locales/` junto al ejecutable (así puedes editar o
//! añadir idiomas) y, si no existen, usa las copias incrustadas en el binario.

use serde_json::Value;
use std::path::Path;

/// Idiomas incrustados en el binario: (código, contenido).
const EMBEDDED: &[(&str, &str)] = &[
    ("ES", include_str!("../locales/ES.lang")),
    ("EN", include_str!("../locales/EN.lang")),
    ("FR", include_str!("../locales/FR.lang")),
    ("DE", include_str!("../locales/DE.lang")),
    ("PL", include_str!("../locales/PL.lang")),
    ("RU", include_str!("../locales/RU.lang")),
    ("HI", include_str!("../locales/HI.lang")),
    ("ZH-HANT", include_str!("../locales/ZH-HANT.lang")),
];

pub struct Locale {
    pub code: String,
    pub name: String,
    data: Value,
}

pub struct I18n {
    pub locales: Vec<Locale>,
    current: usize,
    fallback: usize,
}

fn parse(text: &str) -> Option<Value> {
    serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()
}

fn locale_from(code_hint: &str, data: Value) -> Locale {
    let code = data
        .pointer("/Locale/Code")
        .and_then(Value::as_str)
        .unwrap_or(code_hint)
        .to_uppercase();
    let name = data
        .pointer("/Locale/Name")
        .and_then(Value::as_str)
        .unwrap_or(&code)
        .to_string();
    Locale { code, name, data }
}

impl I18n {
    pub fn load(extra_dir: Option<&Path>, wanted: &str) -> Self {
        let mut locales: Vec<Locale> = Vec::new();
        // 1) Archivos en disco (tienen prioridad y permiten idiomas nuevos).
        if let Some(dir) = extra_dir {
            if let Ok(rd) = std::fs::read_dir(dir) {
                let mut paths: Vec<_> = rd.flatten().map(|e| e.path()).collect();
                paths.sort();
                for p in paths {
                    if p.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("lang")) != Some(true) {
                        continue;
                    }
                    if let Some(v) = std::fs::read_to_string(&p).ok().as_deref().and_then(parse) {
                        let hint = p.file_stem().and_then(|s| s.to_str()).unwrap_or("??");
                        locales.push(locale_from(hint, v));
                    }
                }
            }
        }
        // 2) Incrustados, solo si no se cargaron ya desde disco.
        for (code, text) in EMBEDDED {
            if locales.iter().any(|l| l.code == *code) {
                continue;
            }
            if let Some(v) = parse(text) {
                locales.push(locale_from(code, v));
            }
        }
        let fallback = locales.iter().position(|l| l.code == "EN").unwrap_or(0);
        let mut me = Self { locales, current: fallback, fallback };
        me.set(wanted);
        me
    }

    pub fn set(&mut self, code: &str) {
        if let Some(i) = self.locales.iter().position(|l| l.code.eq_ignore_ascii_case(code)) {
            self.current = i;
        }
    }

    /// Busca `"Seccion.Clave"`. Si falta en el idioma actual usa inglés y, si
    /// tampoco existe, devuelve la propia clave (útil para detectar huecos).
    pub fn t(&self, key: &str) -> String {
        let ptr = format!("/{}", key.replace('.', "/"));
        for idx in [self.current, self.fallback] {
            if let Some(s) = self.locales[idx].data.pointer(&ptr).and_then(Value::as_str) {
                // El original usa '&' para aceleradores de WinForms ("&View").
                return s.replace('&', "");
            }
        }
        key.rsplit('.').next().unwrap_or(key).to_string()
    }
}

/// Idioma del sistema (LANG=es_MX.UTF-8 → "ES").
pub fn system_language() -> String {
    for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(v) = std::env::var(var) {
            let v = v.to_uppercase();
            if v.len() >= 2 && v != "C" && v != "POSIX" {
                if v.starts_with("ZH") {
                    return "ZH-HANT".into();
                }
                return v[..2].to_string();
            }
        }
    }
    #[cfg(windows)]
    {
        // Windows no usa LANG: se pregunta la configuración regional ("es-MX").
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetUserDefaultLocaleName(name: *mut u16, len: i32) -> i32;
        }
        let mut buf = [0u16; 85];
        let n = unsafe { GetUserDefaultLocaleName(buf.as_mut_ptr(), buf.len() as i32) };
        if n > 1 {
            let v = String::from_utf16_lossy(&buf[..(n - 1) as usize]).to_uppercase();
            if v.starts_with("ZH") {
                return "ZH-HANT".into();
            }
            if v.len() >= 2 {
                return v[..2].to_string();
            }
        }
    }
    "EN".into()
}
