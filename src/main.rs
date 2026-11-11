//! Rust Turbo Assembler — port multiplataforma del GUI Turbo Assembler.
#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod config;
mod devshot;
mod dosbox;
mod editor;
mod highlight;
mod i18n;
mod stage;
mod theme;
mod toolchain;
mod update;

use eframe::egui;

/// Ícono de la ventana (el mismo de los paquetes: assets/icon-256.png).
fn icon() -> egui::IconData {
    eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon-256.png")).unwrap_or_default()
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
