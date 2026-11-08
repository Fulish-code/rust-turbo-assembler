//! Resaltado de sintaxis para TASM (modos MASM e IDEAL), 8086–386 + x87.

use eframe::egui::{Color32, FontId, Stroke, text::LayoutJob, TextFormat};

#[derive(Clone, Copy, PartialEq)]
pub struct Palette {
    pub text: Color32,
    pub comment: Color32,
    pub instruction: Color32,
    pub register: Color32,
    pub directive: Color32,
    pub number: Color32,
    pub string: Color32,
    pub label: Color32,
    pub operator: Color32,
    pub search_bg: Color32,
    pub background: Color32,
    pub gutter_bg: Color32,
    pub gutter_fg: Color32,
    pub gutter_fg_active: Color32,
    pub current_line: Color32,
    pub error_line: Color32,
    pub warning_line: Color32,
    pub selection: Color32,
}

impl Palette {
    pub fn dark() -> Self {
        Self {
            text: Color32::from_rgb(0xd4, 0xd8, 0xe0),
            comment: Color32::from_rgb(0x6a, 0x99, 0x55),
            instruction: Color32::from_rgb(0x56, 0x9c, 0xd6),
            register: Color32::from_rgb(0x4e, 0xc9, 0xb0),
            directive: Color32::from_rgb(0xc5, 0x86, 0xc0),
            number: Color32::from_rgb(0xb5, 0xce, 0xa8),
            string: Color32::from_rgb(0xce, 0x91, 0x78),
            label: Color32::from_rgb(0xdc, 0xdc, 0xaa),
            operator: Color32::from_rgb(0x9c, 0xa3, 0xaf),
            search_bg: Color32::from_rgba_unmultiplied(0xe2, 0xb9, 0x3b, 0x55),
            background: Color32::from_rgb(0x1b, 0x1e, 0x24),
            gutter_bg: Color32::from_rgb(0x17, 0x1a, 0x1f),
            gutter_fg: Color32::from_rgb(0x5a, 0x62, 0x70),
            gutter_fg_active: Color32::from_rgb(0xc8, 0xcd, 0xd6),
            current_line: Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, 0x0b),
            error_line: Color32::from_rgba_unmultiplied(0xf0, 0x4b, 0x4b, 0x30),
            warning_line: Color32::from_rgba_unmultiplied(0xe5, 0xb5, 0x32, 0x26),
            selection: Color32::from_rgba_unmultiplied(0x3a, 0x7b, 0xd5, 0x70),
        }
    }

    pub fn light() -> Self {
        Self {
            text: Color32::from_rgb(0x1f, 0x23, 0x28),
            comment: Color32::from_rgb(0x2e, 0x7d, 0x32),
            instruction: Color32::from_rgb(0x00, 0x4d, 0xc2),
            register: Color32::from_rgb(0x00, 0x7a, 0x7a),
            directive: Color32::from_rgb(0x8e, 0x24, 0xaa),
            number: Color32::from_rgb(0x09, 0x86, 0x58),
            string: Color32::from_rgb(0xa3, 0x15, 0x15),
            label: Color32::from_rgb(0x79, 0x5e, 0x26),
            operator: Color32::from_rgb(0x55, 0x5d, 0x68),
            search_bg: Color32::from_rgba_unmultiplied(0xff, 0xc1, 0x07, 0x70),
            background: Color32::from_rgb(0xfc, 0xfc, 0xfd),
            gutter_bg: Color32::from_rgb(0xf1, 0xf3, 0xf6),
            gutter_fg: Color32::from_rgb(0x9a, 0xa1, 0xab),
            gutter_fg_active: Color32::from_rgb(0x30, 0x35, 0x3c),
            current_line: Color32::from_rgba_unmultiplied(0x00, 0x40, 0xa0, 0x0c),
            error_line: Color32::from_rgba_unmultiplied(0xe0, 0x20, 0x20, 0x26),
            warning_line: Color32::from_rgba_unmultiplied(0xe0, 0xa0, 0x00, 0x26),
            selection: Color32::from_rgba_unmultiplied(0x2f, 0x6f, 0xd0, 0x45),
        }
    }
}

const INSTRUCTIONS: &[&str] = &[
    // 8086/8088
    "aaa", "aad", "aam", "aas", "adc", "add", "and", "call", "cbw", "clc", "cld", "cli", "cmc", "cmp",
    "cmpsb", "cmpsw", "cmps", "cwd", "daa", "das", "dec", "div", "esc", "hlt", "idiv", "imul", "in", "inc",
    "int", "into", "iret", "ja", "jae", "jb", "jbe", "jc", "jcxz", "je", "jg", "jge", "jl", "jle", "jmp",
    "jna", "jnae", "jnb", "jnbe", "jnc", "jne", "jng", "jnge", "jnl", "jnle", "jno", "jnp", "jns", "jnz",
    "jo", "jp", "jpe", "jpo", "js", "jz", "lahf", "lds", "lea", "les", "lock", "lodsb", "lodsw", "lods",
    "loop", "loope", "loopne", "loopnz", "loopz", "mov", "movsb", "movsw", "movs", "mul", "neg", "nop",
    "not", "or", "out", "pop", "popf", "push", "pushf", "rcl", "rcr", "rep", "repe", "repne", "repnz",
    "repz", "ret", "retf", "retn", "rol", "ror", "sahf", "sal", "sar", "sbb", "scasb", "scasw", "scas",
    "shl", "shr", "stc", "std", "sti", "stosb", "stosw", "stos", "sub", "test", "wait", "xchg", "xlat",
    "xlatb", "xor",
    // 80186/286
    "bound", "enter", "insb", "insw", "ins", "leave", "outsb", "outsw", "outs", "popa", "pusha", "arpl",
    "clts", "lar", "lgdt", "lidt", "lldt", "lmsw", "lsl", "ltr", "sgdt", "sidt", "sldt", "smsw", "str",
    "verr", "verw",
    // 386/486
    "bsf", "bsr", "bt", "btc", "btr", "bts", "cdq", "cmpsd", "cwde", "insd", "iretd", "jecxz", "lfs",
    "lgs", "lss", "lodsd", "movsd", "movsx", "movzx", "outsd", "popad", "popfd", "pushad", "pushfd",
    "scasd", "seta", "setae", "setb", "setbe", "setc", "sete", "setg", "setge", "setl", "setle", "setna",
    "setnae", "setnb", "setnbe", "setnc", "setne", "setng", "setnge", "setnl", "setnle", "setno", "setnp",
    "setns", "setnz", "seto", "setp", "setpe", "setpo", "sets", "setz", "shld", "shrd", "stosd", "bswap",
    "cmpxchg", "xadd", "invd", "wbinvd", "invlpg", "cpuid", "rdtsc",
    // x87
    "f2xm1", "fabs", "fadd", "faddp", "fbld", "fbstp", "fchs", "fclex", "fcom", "fcomp", "fcompp", "fcos",
    "fdecstp", "fdiv", "fdivp", "fdivr", "fdivrp", "ffree", "fiadd", "ficom", "ficomp", "fidiv", "fidivr",
    "fild", "fimul", "fincstp", "finit", "fist", "fistp", "fisub", "fisubr", "fld", "fld1", "fldcw",
    "fldenv", "fldl2e", "fldl2t", "fldlg2", "fldln2", "fldpi", "fldz", "fmul", "fmulp", "fnclex", "fninit",
    "fnop", "fnsave", "fnstcw", "fnstenv", "fnstsw", "fpatan", "fprem", "fprem1", "fptan", "frndint",
    "frstor", "fsave", "fscale", "fsin", "fsincos", "fsqrt", "fst", "fstcw", "fstenv", "fstp", "fstsw",
    "fsub", "fsubp", "fsubr", "fsubrp", "ftst", "fucom", "fucomp", "fucompp", "fwait", "fxam", "fxch",
    "fxtract", "fyl2x", "fyl2xp1",
];

const REGISTERS: &[&str] = &[
    "al", "ah", "ax", "eax", "bl", "bh", "bx", "ebx", "cl", "ch", "cx", "ecx", "dl", "dh", "dx", "edx",
    "si", "esi", "di", "edi", "bp", "ebp", "sp", "esp", "cs", "ds", "es", "ss", "fs", "gs", "ip", "eip",
    "st", "cr0", "cr2", "cr3", "dr0", "dr1", "dr2", "dr3", "dr6", "dr7", "tr6", "tr7",
];

const DIRECTIVES: &[&str] = &[
    ".model", ".stack", ".data", ".data?", ".code", ".const", ".fardata", ".fardata?", ".startup", ".exit",
    ".8086", ".186", ".286", ".286c", ".286p", ".386", ".386c", ".386p", ".486", ".486p", ".8087", ".287",
    ".387", ".radix", ".list", ".nolist", ".xlist", ".lall", ".sall", ".xall", ".if", ".else", ".elseif",
    ".endif", ".while", ".endw", ".repeat", ".until", ".break", ".continue", "model", "stack", "dataseg",
    "codeseg", "udataseg", "const", "segment", "ends", "assume", "proc", "endp", "end", "db", "dw", "dd",
    "df", "dp", "dq", "dt", "equ", "macro", "endm", "local", "include", "includelib", "org", "even",
    "align", "offset", "ptr", "byte", "word", "dword", "fword", "pword", "qword", "tbyte", "near", "far",
    "short", "seg", "dup", "public", "extrn", "extern", "global", "label", "group", "struc", "struct",
    "union", "record", "ideal", "dosseg", "abs", "masm", "jumps", "nojumps", "locals", "nolocals", "uses", "arg", "small",
    "tiny", "medium", "compact", "large", "huge", "flat", "use16", "use32", "para", "page", "public",
    "common", "at", "memory", "type", "size", "length", "sizeof", "lengthof", "this", "high", "low",
    "mod", "shl", "shr", "eq", "ne", "lt", "le", "gt", "ge", "if", "ifdef", "ifndef", "else", "endif",
    "ife", "ifb", "ifnb", "rept", "irp", "irpc", "exitm", "purge", "comment", "title", "subttl", "name",
    "startupcode", "exitcode", "stdcall", "pascal", "c", "syscall", "basic", "fortran", "nolanguage",
    "invoke", "proto", "typedef", "%out", "@data", "@code", "@stack",
];

#[derive(Clone, Copy)]
enum Kind {
    Text,
    Comment,
    Instruction,
    Register,
    Directive,
    Number,
    Str,
    Label,
    Operator,
}

fn classify_word(word: &str, followed_by_colon: bool, first_on_line: bool) -> Kind {
    let lw = word.to_ascii_lowercase();
    if word.as_bytes()[0].is_ascii_digit() {
        return Kind::Number;
    }
    if REGISTERS.contains(&lw.as_str()) {
        return Kind::Register;
    }
    if INSTRUCTIONS.contains(&lw.as_str()) {
        return Kind::Instruction;
    }
    if DIRECTIVES.contains(&lw.as_str()) {
        return Kind::Directive;
    }
    if followed_by_colon || first_on_line {
        return Kind::Label;
    }
    Kind::Text
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '@' | '$' | '?' | '.' | '%')
}

/// Encuentra coincidencias (en índices de *bytes*) para resaltar la búsqueda.
pub fn find_matches(text: &str, needle: &str, match_case: bool, whole_word: bool) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    if needle.is_empty() {
        return out;
    }
    let (hay, ned) = if match_case {
        (std::borrow::Cow::Borrowed(text), std::borrow::Cow::Borrowed(needle))
    } else {
        // to_ascii_lowercase conserva las longitudes en bytes.
        (std::borrow::Cow::Owned(text.to_ascii_lowercase()), std::borrow::Cow::Owned(needle.to_ascii_lowercase()))
    };
    let mut start = 0;
    while let Some(pos) = hay[start..].find(ned.as_ref()) {
        let a = start + pos;
        let b = a + ned.len();
        let ok = !whole_word || {
            let before = text[..a].chars().next_back().map(is_word_char).unwrap_or(false);
            let after = text[b..].chars().next().map(is_word_char).unwrap_or(false);
            !before && !after
        };
        if ok {
            out.push((a, b));
        }
        start = b.max(a + 1);
        while start < text.len() && !text.is_char_boundary(start) {
            start += 1;
        }
        if out.len() > 50_000 {
            break;
        }
    }
    out
}

/// Construye el `LayoutJob` coloreado del texto completo.
pub fn layout(text: &str, pal: &Palette, font: FontId, wrap_width: f32, matches: &[(usize, usize)]) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;
    let fmt = |k: Kind| -> TextFormat {
        let color = match k {
            Kind::Text => pal.text,
            Kind::Comment => pal.comment,
            Kind::Instruction => pal.instruction,
            Kind::Register => pal.register,
            Kind::Directive => pal.directive,
            Kind::Number => pal.number,
            Kind::Str => pal.string,
            Kind::Label => pal.label,
            Kind::Operator => pal.operator,
        };
        TextFormat { font_id: font.clone(), color, italics: matches!(k, Kind::Comment), ..Default::default() }
    };

    // Segmentos (inicio, fin, tipo) en bytes.
    let mut segs: Vec<(usize, usize, Kind)> = Vec::with_capacity(text.len() / 3);
    let bytes = text.as_bytes();
    let mut i = 0;
    let mut line_start = true; // primer token "real" de la línea
    let mut in_comment_block: Option<char> = None; // COMMENT ^ ... ^
    while i < bytes.len() {
        let c = bytes[i] as char;
        if let Some(delim) = in_comment_block {
            let end = text[i..].find(delim).map(|p| i + p + 1).unwrap_or(text.len());
            let end = text[end..].find('\n').map(|p| end + p).unwrap_or(text.len());
            segs.push((i, end, Kind::Comment));
            i = end;
            in_comment_block = None;
            continue;
        }
        if c == '\n' {
            segs.push((i, i + 1, Kind::Text));
            i += 1;
            line_start = true;
            continue;
        }
        if c == ' ' || c == '\t' || c == '\r' {
            let s = i;
            while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\r') {
                i += 1;
            }
            segs.push((s, i, Kind::Text));
            continue;
        }
        if c == ';' {
            let e = text[i..].find('\n').map(|p| i + p).unwrap_or(text.len());
            segs.push((i, e, Kind::Comment));
            i = e;
            continue;
        }
        if c == '\'' || c == '"' {
            let s = i;
            i += 1;
            while i < bytes.len() && bytes[i] != b'\n' {
                if bytes[i] as char == c {
                    // '' dentro de cadena = comilla escapada
                    if i + 1 < bytes.len() && bytes[i + 1] as char == c {
                        i += 2;
                        continue;
                    }
                    i += 1;
                    break;
                }
                i += 1;
            }
            segs.push((s, i, Kind::Str));
            line_start = false;
            continue;
        }
        if c.is_ascii() && is_word_char(c) {
            let s = i;
            while i < bytes.len() && (bytes[i] as char).is_ascii() && is_word_char(bytes[i] as char) {
                i += 1;
            }
            let word = &text[s..i];
            let colon = bytes.get(i) == Some(&b':');
            let mut kind = classify_word(word, colon, false);
            if line_start && matches!(kind, Kind::Text) {
                // Etiqueta de datos al inicio ("msg db ...") o nombre de PROC/SEGMENT.
                kind = Kind::Label;
            }
            if word.eq_ignore_ascii_case("comment") {
                // COMMENT <delim> ... <delim>
                let rest = text[i..].trim_start_matches([' ', '\t']);
                if let Some(d) = rest.chars().next().filter(|d| !d.is_whitespace()) {
                    segs.push((s, i, Kind::Directive));
                    in_comment_block = Some(d);
                    let off = text.len() - rest.len() + d.len_utf8();
                    segs.push((i, off, Kind::Comment));
                    i = off;
                    line_start = false;
                    continue;
                }
            }
            segs.push((s, i, kind));
            line_start = false;
            continue;
        }
        // Cualquier otro carácter (operadores, unicode, etc.).
        let s = i;
        let len = text[i..].chars().next().map(char::len_utf8).unwrap_or(1);
        i += len;
        segs.push((s, i, if c.is_ascii_punctuation() { Kind::Operator } else { Kind::Text }));
        if c != ':' {
            line_start = false;
        }
    }

    // Fusiona con las coincidencias de búsqueda (fondo resaltado).
    let mut m = 0;
    for (s, e, k) in segs {
        let base = fmt(k);
        let mut pos = s;
        while pos < e {
            while m < matches.len() && matches[m].1 <= pos {
                m += 1;
            }
            let (next, hl) = match matches.get(m) {
                Some(&(ms, me)) if ms <= pos => (me.min(e), true),
                Some(&(ms, _)) if ms < e => (ms, false),
                _ => (e, false),
            };
            let mut f = base.clone();
            if hl {
                f.background = pal.search_bg;
                f.underline = Stroke::NONE;
            }
            job.append(&text[pos..next], 0.0, f);
            pos = next;
        }
    }
    job
}
