//! Editor de código: documento + widget con números de línea, línea actual,
//! marcas de error/advertencia, búsqueda resaltada y operaciones de texto.

use crate::highlight::{self, Palette};
use crate::toolchain::{Diagnostic, Severity};
use eframe::egui::{
    self, Align, Align2, Color32, FontId, Id, Key, Modifiers, Rect, Sense, Shape, TextBuffer, Vec2,
    text::{CCursor, CCursorRange, LayoutJob},
    text_edit::TextEditState,
};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;

pub fn hash_text(s: &str) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EditOp {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Delete,
    SelectAll,
    ToggleComment,
    MoveUp,
    MoveDown,
    Upper,
    Lower,
    Outdent,
}

pub struct Document {
    pub id: u64,
    pub path: Option<PathBuf>,
    pub untitled_n: usize,
    pub text: String,
    pub saved_hash: u64,
    pub assembled_hash: Option<u64>,
    pub built_hash: Option<u64>,
    pub diagnostics: Vec<Diagnostic>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub selection_chars: usize,
    /// Peticiones pendientes de la interfaz.
    pub goto_line: Option<usize>,
    pub select_range: Option<(usize, usize)>,
    pub ops: Vec<EditOp>,
    pub focus: bool,
    pub replace_text: Option<String>,
    layout_cache: Option<(u64, Arc<LayoutJob>)>,
}

impl Document {
    pub fn new(id: u64, untitled_n: usize, path: Option<PathBuf>, text: String) -> Self {
        let text = text.replace("\r\n", "\n");
        let saved_hash = if path.is_some() { hash_text(&text) } else { hash_text("") };
        Self {
            id,
            path,
            untitled_n,
            text,
            saved_hash,
            assembled_hash: None,
            built_hash: None,
            diagnostics: Vec::new(),
            cursor_line: 1,
            cursor_col: 1,
            selection_chars: 0,
            goto_line: None,
            select_range: None,
            ops: Vec::new(),
            focus: true,
            replace_text: None,
            layout_cache: None,
        }
    }

    pub fn text_id(&self) -> Id {
        Id::new(("rtasm-editor", self.id))
    }

    pub fn is_dirty(&self) -> bool {
        hash_text(&self.text) != self.saved_hash
    }

    pub fn title(&self, untitled: &str) -> String {
        match &self.path {
            Some(p) => p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
            None => format!("{untitled} {}", self.untitled_n),
        }
    }

    pub fn is_assembled(&self) -> bool {
        self.assembled_hash == Some(hash_text(&self.text))
    }

    pub fn is_built(&self) -> bool {
        self.built_hash == Some(hash_text(&self.text))
    }

    pub fn line_count(&self) -> usize {
        self.text.bytes().filter(|&b| b == b'\n').count() + 1
    }

    /// Selección actual del widget (en caracteres), si existe.
    pub fn selected_text(&self, ctx: &egui::Context) -> String {
        if let Some(st) = TextEditState::load(ctx, self.text_id()) {
            if let Some(r) = st.cursor.char_range() {
                let [a, b] = r.sorted_cursors();
                return self.text.char_range(a.index..b.index).to_string();
            }
        }
        String::new()
    }
}

fn char_to_byte(s: &str, ci: usize) -> usize {
    s.char_indices().nth(ci).map(|(b, _)| b).unwrap_or(s.len())
}

fn byte_to_char(s: &str, bi: usize) -> usize {
    s[..bi.min(s.len())].chars().count()
}

/// Índice de carácter del inicio de la línea `line` (1-based).
pub fn line_start_char(s: &str, line: usize) -> usize {
    if line <= 1 {
        return 0;
    }
    let mut n = 1;
    for (ci, c) in s.chars().enumerate() {
        if c == '\n' {
            n += 1;
            if n == line {
                return ci + 1;
            }
        }
    }
    s.chars().count()
}

/// Rango de líneas completas [inicio_byte, fin_byte) que cubre la selección.
fn line_span(text: &str, a: usize, b: usize) -> (usize, usize) {
    let start = text[..a].rfind('\n').map(|p| p + 1).unwrap_or(0);
    let mut endb = b;
    if b > a && text[..b].ends_with('\n') {
        endb = b - 1;
    }
    let end = text[endb..].find('\n').map(|p| endb + p).unwrap_or(text.len());
    (start, end)
}

/// Aplica una operación de texto. Devuelve la nueva selección (en caracteres).
fn apply_op(ctx: &egui::Context, text: &mut String, op: EditOp, sel: (usize, usize)) -> Option<(usize, usize)> {
    let (ca, cb) = (sel.0.min(sel.1), sel.0.max(sel.1));
    let (a, b) = (char_to_byte(text, ca), char_to_byte(text, cb));
    match op {
        EditOp::SelectAll => Some((0, text.chars().count())),
        EditOp::Copy => {
            if a < b {
                ctx.copy_text(text[a..b].to_string());
            }
            None
        }
        EditOp::Cut | EditOp::Delete => {
            if a < b {
                if op == EditOp::Cut {
                    ctx.copy_text(text[a..b].to_string());
                }
                text.replace_range(a..b, "");
            }
            Some((ca, ca))
        }
        EditOp::Upper | EditOp::Lower => {
            let (s, e) = if a < b { (a, b) } else { line_span(text, a, b) };
            let seg = if op == EditOp::Upper { text[s..e].to_uppercase() } else { text[s..e].to_lowercase() };
            text.replace_range(s..e, &seg);
            Some((byte_to_char(text, s), byte_to_char(text, s + seg.len())))
        }
        EditOp::ToggleComment | EditOp::Outdent => {
            let (s, e) = line_span(text, a, b);
            let lines: Vec<&str> = text[s..e].split('\n').collect();
            let new: Vec<String> = match op {
                EditOp::ToggleComment => {
                    let all_commented = lines.iter().filter(|l| !l.trim().is_empty()).all(|l| l.trim_start().starts_with(';'));
                    lines
                        .iter()
                        .map(|l| {
                            if l.trim().is_empty() {
                                l.to_string()
                            } else if all_commented {
                                let i = l.find(';').unwrap();
                                let rest = &l[i + 1..];
                                format!("{}{}", &l[..i], rest.strip_prefix(' ').unwrap_or(rest))
                            } else {
                                format!("; {l}")
                            }
                        })
                        .collect()
                }
                _ => lines
                    .iter()
                    .map(|l| {
                        if let Some(r) = l.strip_prefix('\t') {
                            r.to_string()
                        } else {
                            let n = l.len() - l.trim_start_matches(' ').len();
                            l[n.min(8)..].to_string()
                        }
                    })
                    .collect(),
            };
            let joined = new.join("\n");
            text.replace_range(s..e, &joined);
            Some((byte_to_char(text, s), byte_to_char(text, s + joined.len())))
        }
        EditOp::MoveUp | EditOp::MoveDown => {
            let (s, e) = line_span(text, a, b);
            if op == EditOp::MoveUp {
                if s == 0 {
                    return None;
                }
                let ps = text[..s - 1].rfind('\n').map(|p| p + 1).unwrap_or(0);
                let block = text[s..e].to_string();
                let prev = text[ps..s - 1].to_string();
                text.replace_range(ps..e, &format!("{block}\n{prev}"));
                let len_prev = prev.chars().count() + 1;
                Some((ca.saturating_sub(len_prev), cb.saturating_sub(len_prev)))
            } else {
                if e >= text.len() {
                    return None;
                }
                let ne = text[e + 1..].find('\n').map(|p| e + 1 + p).unwrap_or(text.len());
                let block = text[s..e].to_string();
                let next = text[e + 1..ne].to_string();
                text.replace_range(s..ne, &format!("{next}\n{block}"));
                let len_next = next.chars().count() + 1;
                Some((ca + len_next, cb + len_next))
            }
        }
        EditOp::Undo | EditOp::Redo | EditOp::Paste => None,
    }
}

pub struct EditorStyle {
    pub palette: Palette,
    pub font_size: f32,
    pub word_wrap: bool,
    pub auto_indent: bool,
}

pub struct FindQuery<'a> {
    pub needle: &'a str,
    pub match_case: bool,
    pub whole_word: bool,
}

/// Dibuja el editor del documento. Devuelve `true` si el texto cambió.
pub fn show(ui: &mut egui::Ui, doc: &mut Document, style: &EditorStyle, find: Option<FindQuery<'_>>) -> bool {
    let ctx = ui.ctx().clone();
    let id = doc.text_id();
    let pal = style.palette;
    let font = FontId::monospace(style.font_size);

    // --- Operaciones pendientes (menús / atajos) ---
    let ops = std::mem::take(&mut doc.ops);
    let mut changed = false;
    if !ops.is_empty() || doc.select_range.is_some() || doc.goto_line.is_some() || doc.replace_text.is_some() {
        let mut st = TextEditState::load(&ctx, id).unwrap_or_default();
        let cur = st.cursor.char_range().map(|r| (r.secondary.index.0, r.primary.index.0)).unwrap_or((0, 0));
        let mut sel = Some(cur);
        for op in ops {
            match op {
                EditOp::Undo | EditOp::Redo => {
                    let mut undoer = st.undoer();
                    let current = (st.cursor.char_range().unwrap_or_default(), doc.text.clone());
                    let res = if op == EditOp::Undo { undoer.undo(&current).cloned() } else { undoer.redo(&current).cloned() };
                    if let Some((range, t)) = res {
                        doc.text = t;
                        sel = Some((range.secondary.index.0, range.primary.index.0));
                        changed = true;
                    }
                    st.set_undoer(undoer);
                }
                EditOp::Paste => {
                    ctx.memory_mut(|m| m.request_focus(id));
                    ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaste);
                }
                _ => {
                    // Guarda un punto de deshacer antes de modificar.
                    let mut undoer = st.undoer();
                    undoer.add_undo(&(st.cursor.char_range().unwrap_or_default(), doc.text.clone()));
                    st.set_undoer(undoer);
                    let before = doc.text.len();
                    let s = sel.unwrap_or(cur);
                    if let Some(ns) = apply_op(&ctx, &mut doc.text, op, s) {
                        sel = Some(ns);
                    }
                    changed |= doc.text.len() != before || matches!(op, EditOp::Upper | EditOp::Lower | EditOp::MoveUp | EditOp::MoveDown);
                }
            }
        }
        if let Some(rep) = doc.replace_text.take() {
            let (a, b) = sel.unwrap_or(cur);
            let (a, b) = (a.min(b), a.max(b));
            let (ba, bb) = (char_to_byte(&doc.text, a), char_to_byte(&doc.text, b));
            let mut undoer = st.undoer();
            undoer.add_undo(&(st.cursor.char_range().unwrap_or_default(), doc.text.clone()));
            st.set_undoer(undoer);
            doc.text.replace_range(ba..bb, &rep);
            let n = rep.chars().count();
            sel = Some((a, a + n));
            changed = true;
        }
        if let Some(line) = doc.goto_line.take() {
            let c = line_start_char(&doc.text, line);
            sel = Some((c, c));
            doc.focus = true;
        }
        if let Some(r) = doc.select_range.take() {
            sel = Some(r);
            doc.focus = true;
        }
        if let Some((a, b)) = sel {
            st.cursor.set_char_range(Some(CCursorRange::two(CCursor::new(a), CCursor::new(b))));
        }
        st.store(&ctx, id);
        if sel.is_some() {
            ui.memory_mut(|m| m.request_focus(id));
            // Desplazar hasta el cursor tras el próximo dibujo.
            ctx.data_mut(|d| d.insert_temp(id.with("scroll"), true));
        }
    }
    if doc.focus {
        ui.memory_mut(|m| m.request_focus(id));
        doc.focus = false;
    }

    let digits = doc.line_count().to_string().len().max(3);
    let char_w = ui.fonts_mut(|f| f.glyph_width(&font, '0'));
    let gutter_w = char_w * digits as f32 + 22.0;
    let row_h = ui.fonts_mut(|f| f.row_height(&font));

    // --- Resaltado con caché ---
    let matches = find
        .as_ref()
        .map(|f| highlight::find_matches(&doc.text, f.needle, f.match_case, f.whole_word))
        .unwrap_or_default();
    let wrap = style.word_wrap;
    let cache = &mut doc.layout_cache;
    let mut layouter = |ui: &egui::Ui, buf: &dyn TextBuffer, wrap_width: f32| -> Arc<egui::Galley> {
        let text = buf.as_str();
        let mut h = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut h);
        matches.hash(&mut h);
        style.font_size.to_bits().hash(&mut h);
        (pal.text.to_array(), pal.background.to_array()).hash(&mut h);
        let w = if wrap { wrap_width } else { f32::INFINITY };
        w.to_bits().hash(&mut h);
        let key = h.finish();
        let job = match cache {
            Some((k, j)) if *k == key => j.clone(),
            _ => {
                let j = Arc::new(highlight::layout(text, &pal, font.clone(), w, &matches));
                *cache = Some((key, j.clone()));
                j
            }
        };
        ui.fonts_mut(|f| f.layout_job((*job).clone()))
    };


    let mut text_changed = changed;
    let scroll = if wrap { egui::ScrollArea::vertical() } else { egui::ScrollArea::both() };
    scroll
        .id_salt(("rtasm-scroll", doc.id))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let (gutter_rect, _) = ui.allocate_exact_size(Vec2::new(gutter_w, row_h), Sense::hover());
                let bg_idx = ui.painter().add(Shape::Noop);
                let avail = ui.available_size();
                let out = egui::TextEdit::multiline(&mut doc.text)
                    .id(id)
                    .font(font.clone())
                    .code_editor()
                    .frame(egui::Frame::NONE)
                    .lock_focus(true)
                    .desired_width(if wrap { avail.x } else { f32::INFINITY })
                    .min_size(Vec2::new(avail.x, avail.y.max(row_h)))
                    .layouter(&mut layouter)
                    .show(ui);

                // Sangría automática: al pulsar Enter, copiar la sangría de la línea anterior.
                if out.response.changed() {
                    text_changed = true;
                    let enter = ui.input(|i| i.key_pressed(Key::Enter) && i.modifiers.is_none());
                    if enter && style.auto_indent {
                        if let Some(r) = out.cursor_range {
                            let ci = r.primary.index.0;
                            let bi = char_to_byte(&doc.text, ci);
                            if bi > 0 && doc.text[..bi].ends_with('\n') {
                                let prev_start = doc.text[..bi - 1].rfind('\n').map(|p| p + 1).unwrap_or(0);
                                let indent: String = doc.text[prev_start..bi - 1]
                                    .chars()
                                    .take_while(|c| *c == ' ' || *c == '\t')
                                    .collect();
                                if !indent.is_empty() {
                                    doc.text.insert_str(bi, &indent);
                                    let mut st = out.state.clone();
                                    let n = ci + indent.chars().count();
                                    st.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(n))));
                                    st.store(ui.ctx(), id);
                                }
                            }
                        }
                    }
                }

                // Posición del cursor (para la barra de estado).
                let cursor = out.cursor_range.or_else(|| out.state.cursor.char_range());
                let mut cur_line = 0usize;
                if let Some(r) = cursor {
                    let ci = r.primary.index.0;
                    let bi = char_to_byte(&doc.text, ci);
                    cur_line = doc.text[..bi].bytes().filter(|&b| b == b'\n').count();
                    let ls = doc.text[..bi].rfind('\n').map(|p| p + 1).unwrap_or(0);
                    doc.cursor_line = cur_line + 1;
                    doc.cursor_col = doc.text[ls..bi].chars().count() + 1;
                    doc.selection_chars = r.primary.index.0.abs_diff(r.secondary.index.0);
                    // Desplazamiento solicitado (ir a línea / buscar).
                    let want_scroll = ui.ctx().data_mut(|d| d.remove_temp::<bool>(id.with("scroll"))).unwrap_or(false);
                    if want_scroll {
                        let rect = out.galley.pos_from_cursor(r.primary).translate(out.galley_pos.to_vec2());
                        ui.scroll_to_rect(rect.expand(row_h * 2.0), Some(Align::Center));
                    }
                }

                // Fondo: línea actual + líneas con diagnóstico; números de línea.
                let clip = ui.clip_rect();
                let full_left = gutter_rect.left();
                let full_right = clip.right().max(out.galley_pos.x + out.galley.rect.width());
                let mut bg = Vec::new();
                bg.push(Shape::rect_filled(
                    Rect::from_x_y_ranges(full_left..=full_left + gutter_w, clip.top()..=clip.bottom().max(out.galley_pos.y + out.galley.rect.height())),
                    0.0,
                    pal.gutter_bg,
                ));
                let painter = ui.painter().clone();
                let mut line = 0usize;
                let mut new_line = true;
                for row in &out.galley.rows {
                    let top = out.galley_pos.y + row.pos.y;
                    let h = row.row.size.y.max(row_h);
                    let visible = top + h >= clip.top() && top <= clip.bottom();
                    if visible {
                        let rr = Rect::from_x_y_ranges(full_left + gutter_w..=full_right, top..=top + h);
                        let sev = doc
                            .diagnostics
                            .iter()
                            .filter(|d| d.line == line + 1)
                            .map(|d| d.severity)
                            .min_by_key(|s| if *s == Severity::Error { 0 } else { 1 });
                        match sev {
                            Some(Severity::Error) => bg.push(Shape::rect_filled(rr, 0.0, pal.error_line)),
                            Some(Severity::Warning) => bg.push(Shape::rect_filled(rr, 0.0, pal.warning_line)),
                            None if line == cur_line && cursor.is_some() => bg.push(Shape::rect_filled(rr, 0.0, pal.current_line)),
                            None => {}
                        }
                        if new_line {
                            let color = if line == cur_line { pal.gutter_fg_active } else { pal.gutter_fg };
                            painter.text(
                                egui::pos2(full_left + gutter_w - 12.0, top),
                                Align2::RIGHT_TOP,
                                (line + 1).to_string(),
                                font.clone(),
                                color,
                            );
                            if let Some(s) = sev {
                                let c = if s == Severity::Error { Color32::from_rgb(0xf0, 0x4b, 0x4b) } else { Color32::from_rgb(0xe5, 0xb5, 0x32) };
                                painter.circle_filled(egui::pos2(full_left + 6.0, top + h / 2.0), 3.0, c);
                            }
                        }
                    }
                    new_line = row.ends_with_newline;
                    if row.ends_with_newline {
                        line += 1;
                    }
                    if top > clip.bottom() {
                        break;
                    }
                }
                ui.painter().set(bg_idx, Shape::Vec(bg));

                // Tooltip con el mensaje al pasar sobre una línea con error.
                if let Some(pos) = out.response.hover_pos() {
                    let rel = pos - out.galley_pos;
                    let cc = out.galley.cursor_from_pos(rel);
                    let bi = char_to_byte(&doc.text, cc.index.0);
                    let ln = doc.text[..bi].bytes().filter(|&b| b == b'\n').count() + 1;
                    let msgs: Vec<String> = doc.diagnostics.iter().filter(|d| d.line == ln).map(|d| d.message.clone()).collect();
                    if !msgs.is_empty() {
                        out.response.response.clone().on_hover_text(msgs.join("\n"));
                    }
                }
            });
        });

    // Atajos propios del editor (solo con el foco en este documento).
    if ui.memory(|m| m.has_focus(id)) {
        let ctrl = Modifiers::COMMAND;
        let shortcuts = [
            (Modifiers::ALT, Key::ArrowUp, EditOp::MoveUp),
            (Modifiers::ALT, Key::ArrowDown, EditOp::MoveDown),
            (ctrl, Key::Slash, EditOp::ToggleComment),
            (ctrl | Modifiers::SHIFT, Key::U, EditOp::Upper),
            (ctrl, Key::U, EditOp::Lower),
            (Modifiers::SHIFT, Key::Tab, EditOp::Outdent),
        ];
        for (m, k, op) in shortcuts {
            if ui.input_mut(|i| i.consume_key(m, k)) {
                doc.ops.push(op);
                ui.ctx().request_repaint();
            }
        }
    }
    text_changed
}
