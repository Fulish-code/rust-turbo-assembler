//! Capturas de la propia ventana para pruebas: `RTASM_SHOT=archivo.png`.
//! Útil en Wayland, donde las herramientas externas no pueden capturar.
//! No hace nada si la variable no está definida.

use eframe::egui;
use std::path::PathBuf;

pub struct DevShot {
    path: Option<PathBuf>,
    frames: u32,
    requested: bool,
}

impl DevShot {
    pub fn from_env() -> Self {
        let path = std::env::var_os("RTASM_SHOT").map(PathBuf::from);
        let frames = std::env::var("RTASM_SHOT_FRAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(40);
        Self { path, frames, requested: false }
    }

    pub fn active(&self) -> bool {
        self.path.is_some()
    }

    /// `busy`: hay una tarea en curso; la captura espera a que termine.
    pub fn tick(&mut self, ctx: &egui::Context, busy: bool) {
        let Some(path) = self.path.clone() else { return };
        ctx.request_repaint();
        if busy {
            return;
        }
        if !self.requested {
            if self.frames == 0 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
                self.requested = true;
            } else {
                self.frames -= 1;
            }
            return;
        }
        let shot = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = shot {
            let mut rgba = Vec::with_capacity(img.pixels.len() * 4);
            for p in &img.pixels {
                rgba.extend_from_slice(&p.to_srgba_unmultiplied());
            }
            let _ = std::fs::write(&path, encode_png(img.size[0] as u32, img.size[1] as u32, &rgba));
            self.path = None;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn crc32(data: &[u8]) -> u32 {
    let mut c = 0xffff_ffffu32;
    for &b in data {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xedb8_8320 ^ (c >> 1) } else { c >> 1 };
        }
    }
    !c
}

/// PNG RGBA de 8 bits con bloques deflate sin compresión.
fn encode_png(w: u32, h: u32, rgba: &[u8]) -> Vec<u8> {
    let mut raw = Vec::with_capacity((w as usize * 4 + 1) * h as usize);
    for y in 0..h as usize {
        raw.push(0);
        raw.extend_from_slice(&rgba[y * w as usize * 4..(y + 1) * w as usize * 4]);
    }
    let mut z = vec![0x78, 0x01];
    let chunks: Vec<&[u8]> = raw.chunks(65_535).collect();
    for (i, ch) in chunks.iter().enumerate() {
        z.push(if i + 1 == chunks.len() { 1 } else { 0 });
        let len = ch.len() as u16;
        z.extend_from_slice(&len.to_le_bytes());
        z.extend_from_slice(&(!len).to_le_bytes());
        z.extend_from_slice(ch);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &x in &raw {
        a = (a + x as u32) % 65_521;
        b = (b + a) % 65_521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());

    let mut out = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut chunk = |kind: &[u8], data: &[u8]| {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut c = kind.to_vec();
        c.extend_from_slice(data);
        out.extend_from_slice(&c);
        out.extend_from_slice(&crc32(&c).to_be_bytes());
    };
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(b"IHDR", &ihdr);
    chunk(b"IDAT", &z);
    chunk(b"IEND", &[]);
    out
}
