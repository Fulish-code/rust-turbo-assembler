//! Temas del IDE inspirados en Visual Studio 2015 (Azul, Oscuro, Claro), los
//! mismos que ofrece el GUI Turbo Assembler original, con un acabado moderno.

use crate::config::IdeTheme;
use eframe::egui::{self, Color32, CornerRadius, Stroke, Visuals};

pub struct ThemeColors {
    pub accent: Color32,
    pub toolbar: Color32,
    pub status_bg: Color32,
    pub status_busy: Color32,
    pub status_fg: Color32,
    pub panel: Color32,
    pub log_bg: Color32,
    pub log_fg: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub tool: Color32,
    pub dim: Color32,
}

pub fn colors(t: IdeTheme) -> ThemeColors {
    match t {
        IdeTheme::Dark => ThemeColors {
            accent: Color32::from_rgb(0x3b, 0x8e, 0xea),
            toolbar: Color32::from_rgb(0x24, 0x27, 0x2e),
            status_bg: Color32::from_rgb(0x1f, 0x5f, 0xb0),
            status_busy: Color32::from_rgb(0xb0, 0x6a, 0x1f),
            status_fg: Color32::WHITE,
            panel: Color32::from_rgb(0x20, 0x23, 0x29),
            log_bg: Color32::from_rgb(0x18, 0x1b, 0x20),
            log_fg: Color32::from_rgb(0xc8, 0xcd, 0xd6),
            success: Color32::from_rgb(0x5f, 0xd0, 0x7e),
            warning: Color32::from_rgb(0xe5, 0xc0, 0x4a),
            error: Color32::from_rgb(0xf2, 0x6d, 0x6d),
            tool: Color32::from_rgb(0xa9, 0xb1, 0xbd),
            dim: Color32::from_rgb(0x6c, 0x74, 0x80),
        },
        IdeTheme::Light => ThemeColors {
            accent: Color32::from_rgb(0x00, 0x6f, 0xd6),
            toolbar: Color32::from_rgb(0xee, 0xef, 0xf2),
            status_bg: Color32::from_rgb(0x00, 0x6f, 0xd6),
            status_busy: Color32::from_rgb(0xc8, 0x6c, 0x00),
            status_fg: Color32::WHITE,
            panel: Color32::from_rgb(0xf5, 0xf6, 0xf8),
            log_bg: Color32::from_rgb(0xff, 0xff, 0xff),
            log_fg: Color32::from_rgb(0x24, 0x29, 0x2f),
            success: Color32::from_rgb(0x1a, 0x7f, 0x37),
            warning: Color32::from_rgb(0x9a, 0x67, 0x00),
            error: Color32::from_rgb(0xcf, 0x22, 0x2e),
            tool: Color32::from_rgb(0x44, 0x4d, 0x56),
            dim: Color32::from_rgb(0x8c, 0x95, 0x9f),
        },
        IdeTheme::Blue => ThemeColors {
            accent: Color32::from_rgb(0xf5, 0xcc, 0x84),
            toolbar: Color32::from_rgb(0xcf, 0xd6, 0xe5),
            status_bg: Color32::from_rgb(0x29, 0x39, 0x55),
            status_busy: Color32::from_rgb(0xca, 0x51, 0x00),
            status_fg: Color32::WHITE,
            panel: Color32::from_rgb(0xdd, 0xe3, 0xef),
            log_bg: Color32::from_rgb(0xf7, 0xf9, 0xfc),
            log_fg: Color32::from_rgb(0x1e, 0x29, 0x3b),
            success: Color32::from_rgb(0x1a, 0x7f, 0x37),
            warning: Color32::from_rgb(0x9a, 0x67, 0x00),
            error: Color32::from_rgb(0xc0, 0x1c, 0x28),
            tool: Color32::from_rgb(0x33, 0x40, 0x55),
            dim: Color32::from_rgb(0x6b, 0x77, 0x8c),
        },
    }
}

pub fn apply(ctx: &egui::Context, t: IdeTheme) {
    let c = colors(t);
    let mut v = match t {
        IdeTheme::Dark => Visuals::dark(),
        _ => Visuals::light(),
    };
    let r = CornerRadius::same(5);
    v.window_corner_radius = CornerRadius::same(8);
    v.menu_corner_radius = CornerRadius::same(6);
    v.widgets.noninteractive.corner_radius = r;
    v.widgets.inactive.corner_radius = r;
    v.widgets.hovered.corner_radius = r;
    v.widgets.active.corner_radius = r;
    v.widgets.open.corner_radius = r;
    match t {
        IdeTheme::Dark => {
            v.panel_fill = c.panel;
            v.window_fill = Color32::from_rgb(0x26, 0x29, 0x30);
            v.extreme_bg_color = Color32::from_rgb(0x16, 0x18, 0x1d);
            v.faint_bg_color = Color32::from_rgb(0x26, 0x2a, 0x31);
            v.selection.bg_fill = Color32::from_rgb(0x26, 0x4f, 0x80);
            v.selection.stroke = Stroke::new(1.0, Color32::from_rgb(0xb5, 0xd3, 0xff));
            v.hyperlink_color = c.accent;
            v.widgets.inactive.weak_bg_fill = Color32::from_rgb(0x2c, 0x30, 0x38);
            v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x36, 0x3b, 0x45);
            v.window_stroke = Stroke::new(1.0, Color32::from_rgb(0x38, 0x3d, 0x47));
        }
        IdeTheme::Light => {
            v.panel_fill = c.panel;
            v.window_fill = Color32::WHITE;
            v.extreme_bg_color = Color32::WHITE;
            v.faint_bg_color = Color32::from_rgb(0xf0, 0xf2, 0xf5);
            v.selection.bg_fill = Color32::from_rgb(0xc7, 0xdd, 0xfb);
            v.selection.stroke = Stroke::new(1.0, Color32::from_rgb(0x00, 0x3d, 0x80));
            v.hyperlink_color = c.accent;
        }
        IdeTheme::Blue => {
            v.panel_fill = c.panel;
            v.window_fill = Color32::from_rgb(0xf2, 0xf5, 0xfa);
            v.extreme_bg_color = Color32::WHITE;
            v.faint_bg_color = Color32::from_rgb(0xe6, 0xeb, 0xf4);
            v.selection.bg_fill = Color32::from_rgb(0xfd, 0xe6, 0xb8);
            v.selection.stroke = Stroke::new(1.0, Color32::from_rgb(0x29, 0x39, 0x55));
            v.hyperlink_color = Color32::from_rgb(0x1e, 0x4e, 0x9a);
            v.widgets.noninteractive.bg_fill = c.panel;
            v.widgets.inactive.weak_bg_fill = Color32::from_rgb(0xc4, 0xcd, 0xe0);
            v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0xfd, 0xf4, 0xbf);
            v.widgets.active.weak_bg_fill = Color32::from_rgb(0xf5, 0xcc, 0x84);
            v.window_stroke = Stroke::new(1.0, Color32::from_rgb(0x8e, 0x9b, 0xbc));
        }
    }
    let th = if t == IdeTheme::Dark { egui::Theme::Dark } else { egui::Theme::Light };
    ctx.set_theme(th);
    ctx.set_visuals_of(th, v);
    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 5.0);
        s.spacing.button_padding = egui::vec2(8.0, 4.0);
        s.spacing.menu_margin = egui::Margin::same(6);
        s.interaction.tooltip_delay = 0.35;
    });
}
