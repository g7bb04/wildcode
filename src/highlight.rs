use crate::config::ThemeChoice;
use crate::theme::palette;
use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{FontId, Ui};

const KEYWORDS: &[&str] = &[
    "alignas",
    "alignof",
    "and",
    "asm",
    "auto",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "constexpr",
    "consteval",
    "constinit",
    "const_cast",
    "continue",
    "decltype",
    "default",
    "delete",
    "do",
    "dynamic_cast",
    "else",
    "enum",
    "explicit",
    "export",
    "extern",
    "final",
    "for",
    "friend",
    "goto",
    "if",
    "inline",
    "mutable",
    "namespace",
    "new",
    "noexcept",
    "not",
    "operator",
    "or",
    "override",
    "private",
    "protected",
    "public",
    "register",
    "reinterpret_cast",
    "return",
    "sizeof",
    "static",
    "static_assert",
    "static_cast",
    "struct",
    "switch",
    "template",
    "this",
    "throw",
    "try",
    "typedef",
    "typeid",
    "typename",
    "union",
    "using",
    "virtual",
    "volatile",
    "while",
    "xor",
];

const TYPES: &[&str] = &[
    "bool",
    "byte",
    "char",
    "double",
    "float",
    "int",
    "long",
    "short",
    "signed",
    "unsigned",
    "void",
    "wchar_t",
    "size_t",
    "ssize_t",
    "boolean",
    "word",
    "String",
    "int8_t",
    "int16_t",
    "int32_t",
    "int64_t",
    "uint8_t",
    "uint16_t",
    "uint32_t",
    "uint64_t",
    "HardwareSerial",
    "Stream",
    "Print",
    "File",
    "IPAddress",
    "WiFiClient",
    "WiFiServer",
    "WiFiUDP",
    "Serial",
    "Serial1",
    "Serial2",
    "Serial3",
    "SerialBT",
    "Wire",
    "SPI",
    "ESP",
];

const CONSTANTS: &[&str] = &[
    "true",
    "false",
    "nullptr",
    "NULL",
    "HIGH",
    "LOW",
    "INPUT",
    "OUTPUT",
    "INPUT_PULLUP",
    "INPUT_PULLDOWN",
    "LED_BUILTIN",
    "PI",
    "HALF_PI",
    "TWO_PI",
    "DEG_TO_RAD",
    "RAD_TO_DEG",
    "RISING",
    "FALLING",
    "CHANGE",
    "ONLOW",
    "ONHIGH",
    "MSBFIRST",
    "LSBFIRST",
];

/// Turn C/C++ source into a colored `LayoutJob` used by the editor and by
/// read-only code views. The lexer is deliberately small but handles the
/// constructs that matter for Arduino sketches: comments, strings, numbers,
/// preprocessor lines and identifiers.
pub fn code_layout(_ui: &Ui, code: &str, font_size: f32, theme: ThemeChoice) -> LayoutJob {
    build_layout(code, font_size, theme)
}

pub fn build_layout(code: &str, font_size: f32, theme: ThemeChoice) -> LayoutJob {
    let p = palette(theme);
    let font_id = FontId::monospace(font_size);
    let bytes = code.as_bytes();
    let mut job = LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;

    let mut i = 0usize;
    while i < bytes.len() {
        let start = i;
        let b = bytes[i];

        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            append(&mut job, &code[start..i], p.comment, &font_id, true);
        } else if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i < bytes.len() {
                if bytes[i] == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            append(&mut job, &code[start..i], p.comment, &font_id, true);
        } else if b == b'"' || b == b'\'' {
            let quote = b;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i = (i + 2).min(bytes.len());
                } else if bytes[i] == quote {
                    i += 1;
                    break;
                } else if bytes[i] == b'\n' && quote == b'\'' {
                    break;
                } else {
                    i += 1;
                }
            }
            append(&mut job, &code[start..i], p.string, &font_id, false);
        } else if b == b'#' && at_line_start(code, i) {
            i += 1;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            append(&mut job, &code[start..i], p.preprocessor, &font_id, false);
        } else if is_ident_start(b) {
            i += 1;
            while i < bytes.len() && is_ident_continue(bytes[i]) {
                i += 1;
            }
            let word = &code[start..i];
            let color = classify(word, code, i, &p);
            append(&mut job, word, color, &font_id, false);
        } else if b.is_ascii_digit() {
            i += 1;
            while i < bytes.len() {
                let next = bytes[i];
                if next.is_ascii_alphanumeric() || next == b'.' || next == b'_' {
                    i += 1;
                } else {
                    break;
                }
            }
            append(&mut job, &code[start..i], p.number, &font_id, false);
        } else {
            let ch = code[i..].chars().next().unwrap_or(' ');
            i += ch.len_utf8();
            append(&mut job, &code[start..i], p.text, &font_id, false);
        }
    }

    job
}

fn append(
    job: &mut LayoutJob,
    text: &str,
    color: eframe::egui::Color32,
    font_id: &FontId,
    italics: bool,
) {
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: font_id.clone(),
            color,
            italics,
            ..Default::default()
        },
    );
}

fn at_line_start(code: &str, index: usize) -> bool {
    let line_start = code[..index].rfind('\n').map(|pos| pos + 1).unwrap_or(0);
    code[line_start..index]
        .chars()
        .all(|ch| ch == ' ' || ch == '\t')
}

fn classify(
    word: &str,
    code: &str,
    after: usize,
    palette: &crate::theme::Palette,
) -> eframe::egui::Color32 {
    if KEYWORDS.contains(&word) {
        return palette.keyword;
    }
    if TYPES.contains(&word) {
        return palette.type_name;
    }
    if CONSTANTS.contains(&word) {
        return palette.constant;
    }

    let rest = &code[after..];
    let next = rest.chars().find(|ch| !ch.is_whitespace());
    let prev = code[..after - word.len()]
        .chars()
        .rev()
        .find(|ch| !ch.is_whitespace());

    if next == Some('(') {
        return palette.function;
    }
    if prev == Some('.') || prev == Some(':') {
        return palette.text;
    }
    if word.len() > 1
        && word
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
    {
        return palette.constant;
    }
    palette.text
}

fn is_ident_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_ident_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizer_preserves_text() {
        for code in [
            "void setup() {\n  Serial.begin(115200);\n}\n",
            "// 中文注释\nconst char* s = \"你好\";\n",
            "/* multi\nline */ int x = 0x1F;",
            "String s = \"unterminated",
            "#include <WiFi.h>\r\nvoid loop() {}",
        ] {
            let job = build_layout(code, 14.0, ThemeChoice::Dark);
            let joined: String = job
                .sections
                .iter()
                .map(|section| &job.text[section.byte_range.clone()])
                .collect();
            assert_eq!(joined, code);
        }
    }
}
