//! Rust Turbo Assembler — port multiplataforma del GUI Turbo Assembler.
#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod config;
mod devshot;
mod dosbox;
mod editor;
mod highlight;
mod i18n;
mod theme;
mod toolchain;

use eframe::egui;

/// Ícono de la ventana dibujado en tiempo de ejecución (sin archivos extra).
fn icon() -> egui::IconData {
    let n = 64usize;
    let mut rgba = vec![0u8; n * n * 4];
    for y in 0..n {
        for x in 0..n {
            let i = (y * n + x) * 4;
            // Cuadro redondeado azul con un "chip" claro en el centro.
            let (fx, fy) = (x as f32 - 31.5, y as f32 - 31.5);
            let r = 12.0;
            let dx = (fx.abs() - (31.5 - r)).max(0.0);
            let dy = (fy.abs() - (31.5 - r)).max(0.0);
            if dx * dx + dy * dy > r * r {
                continue;
            }
            let (mut cr, mut cg, mut cb) = (0x1f, 0x5f, 0xb0);
            let chip = fx.abs() < 15.0 && fy.abs() < 15.0;
            let pins = (fx.abs() < 13.0 && (fy.abs() > 15.0 && fy.abs() < 22.0) && ((x / 5) % 2 == 0))
                || (fy.abs() < 13.0 && (fx.abs() > 15.0 && fx.abs() < 22.0) && ((y / 5) % 2 == 0));
            if chip {
                (cr, cg, cb) = (0xe8, 0xee, 0xf6);
                if fx.abs() < 8.0 && fy.abs() < 8.0 {
                    (cr, cg, cb) = (0xf5, 0xa6, 0x23);
                }
            } else if pins {
                (cr, cg, cb) = (0xc8, 0xd4, 0xe6);
            }
            rgba[i..i + 4].copy_from_slice(&[cr, cg, cb, 255]);
        }
    }
    egui::IconData { rgba, width: n as u32, height: n as u32 }
}

fn main() -> eframe::Result<()> {
    let settings = config::Settings::load();
    let mut vp = egui::ViewportBuilder::default()
        .with_title(app::APP_NAME)
        .with_app_id("rust-tasm")
        .with_min_inner_size([640.0, 420.0])
        .with_inner_size([1180.0, 760.0])
        .with_icon(icon())
        .with_drag_and_drop(true);
    if settings.restore_window {
        if let Some([x, y, w, h]) = settings.window {
            if w > 200.0 && h > 200.0 {
                vp = vp.with_inner_size([w, h]).with_position([x, y]);
            }
        }
    }
    if settings.launch_maximized {
        vp = vp.with_maximized(true);
    }
    let options = eframe::NativeOptions { viewport: vp, centered: settings.window.is_none(), ..Default::default() };
    eframe::run_native(app::APP_NAME, options, Box::new(|cc| Ok(Box::new(app::App::new(cc)))))
}
