use crate::config::ThemeChoice;
use eframe::egui::{
    self, Color32, FontData, FontDefinitions, FontFamily, FontId, Rounding, Stroke, TextStyle,
    Visuals,
};
use std::path::PathBuf;

pub const ACCENT: Color32 = Color32::from_rgb(0x7e, 0xe7, 0x87);
pub const WARN: Color32 = Color32::from_rgb(0xf2, 0xc9, 0x7d);
pub const ERROR: Color32 = Color32::from_rgb(0xff, 0x7b, 0x72);
pub const OK: Color32 = Color32::from_rgb(0x7e, 0xe7, 0x87);

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub is_dark: bool,
    pub panel_bg: Color32,
    pub bar_bg: Color32,
    pub editor_bg: Color32,
    pub faint_bg: Color32,
    pub border: Color32,
    pub text: Color32,
    pub weak_text: Color32,
    pub gutter: Color32,
    pub selection: Color32,
    pub keyword: Color32,
    pub type_name: Color32,
    pub function: Color32,
    pub string: Color32,
    pub number: Color32,
    pub comment: Color32,
    pub preprocessor: Color32,
    pub constant: Color32,
}

pub fn palette(theme: ThemeChoice) -> Palette {
    match theme {
        ThemeChoice::Dark => Palette {
            is_dark: true,
            panel_bg: Color32::from_rgb(0x16, 0x19, 0x1f),
            bar_bg: Color32::from_rgb(0x1b, 0x1f, 0x27),
            editor_bg: Color32::from_rgb(0x0e, 0x11, 0x17),
            faint_bg: Color32::from_rgb(0x20, 0x25, 0x2e),
            border: Color32::from_rgb(0x2c, 0x33, 0x3f),
            text: Color32::from_rgb(0xd7, 0xde, 0xe9),
            weak_text: Color32::from_rgb(0x8b, 0x97, 0xa8),
            gutter: Color32::from_rgb(0x4b, 0x58, 0x6b),
            selection: Color32::from_rgb(0x2f, 0x5d, 0x3e),
            keyword: Color32::from_rgb(0xff, 0x7b, 0x9c),
            type_name: Color32::from_rgb(0x6f, 0xd3, 0xc7),
            function: Color32::from_rgb(0x8a, 0xb4, 0xff),
            string: Color32::from_rgb(0xa5, 0xe0, 0x75),
            number: Color32::from_rgb(0xf5, 0xc2, 0x6b),
            comment: Color32::from_rgb(0x6b, 0x7a, 0x90),
            preprocessor: Color32::from_rgb(0xff, 0xa6, 0x57),
            constant: Color32::from_rgb(0xe5, 0xc0, 0x7b),
        },
        ThemeChoice::Light => Palette {
            is_dark: false,
            panel_bg: Color32::from_rgb(0xf3, 0xf5, 0xf8),
            bar_bg: Color32::from_rgb(0xe9, 0xed, 0xf3),
            editor_bg: Color32::from_rgb(0xff, 0xff, 0xff),
            faint_bg: Color32::from_rgb(0xe7, 0xeb, 0xf1),
            border: Color32::from_rgb(0xd3, 0xd9, 0xe2),
            text: Color32::from_rgb(0x25, 0x2b, 0x35),
            weak_text: Color32::from_rgb(0x6b, 0x74, 0x82),
            gutter: Color32::from_rgb(0xa5, 0xad, 0xb9),
            selection: Color32::from_rgb(0xc6, 0xea, 0xce),
            keyword: Color32::from_rgb(0xaf, 0x00, 0xdb),
            type_name: Color32::from_rgb(0x1d, 0x7f, 0x99),
            function: Color32::from_rgb(0x6f, 0x42, 0xc1),
            string: Color32::from_rgb(0x0a, 0x7d, 0x33),
            number: Color32::from_rgb(0xb3, 0x5c, 0x00),
            comment: Color32::from_rgb(0x6e, 0x77, 0x81),
            preprocessor: Color32::from_rgb(0xa8, 0x52, 0x00),
            constant: Color32::from_rgb(0x95, 0x38, 0x00),
        },
    }
}

/// Install a CJK capable font as fallback so Chinese text and comments render.
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    if let Some((name, bytes)) = load_cjk_font() {
        fonts
            .font_data
            .insert(name.clone(), FontData::from_owned(bytes));
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            fonts.families.entry(family).or_default().push(name.clone());
        }
    }
    ctx.set_fonts(fonts);
}

fn load_cjk_font() -> Option<(String, Vec<u8>)> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(windir) = std::env::var("WINDIR") {
        let fonts = PathBuf::from(windir).join("Fonts");
        candidates.extend([
            fonts.join("msyh.ttc"),
            fonts.join("msyhbd.ttc"),
            fonts.join("Deng.ttf"),
            fonts.join("simhei.ttf"),
            fonts.join("simsun.ttc"),
        ]);
    }
    candidates.extend([
        PathBuf::from("/System/Library/Fonts/PingFang.ttc"),
        PathBuf::from("/System/Library/Fonts/Hiragino Sans GB.ttc"),
        PathBuf::from("/Library/Fonts/Arial Unicode.ttf"),
        PathBuf::from("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"),
        PathBuf::from("/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc"),
        PathBuf::from("/usr/share/fonts/truetype/wqy/wqy-microhei.ttc"),
        PathBuf::from("/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf"),
    ]);

    for path in candidates {
        if let Ok(bytes) = std::fs::read(&path) {
            if bytes.len() > 1024 {
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_lowercase())
                    .unwrap_or_else(|| "cjk".to_owned());
                return Some((name, bytes));
            }
        }
    }
    None
}

pub fn apply(ctx: &egui::Context, theme: ThemeChoice, editor_font_size: f32) {
    let p = palette(theme);
    let mut visuals = if p.is_dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    visuals.panel_fill = p.panel_bg;
    visuals.window_fill = p.panel_bg;
    visuals.extreme_bg_color = p.editor_bg;
    visuals.faint_bg_color = p.faint_bg;
    visuals.override_text_color = Some(p.text);
    visuals.hyperlink_color = ACCENT;
    visuals.selection.bg_fill = p.selection;
    visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.window_stroke = Stroke::new(1.0_f32, p.border);
    visuals.window_rounding = Rounding::same(10.0);
    visuals.menu_rounding = Rounding::same(8.0);

    visuals.widgets.noninteractive.bg_fill = p.panel_bg;
    visuals.widgets.noninteractive.weak_bg_fill = p.panel_bg;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.border);
    visuals.widgets.inactive.bg_fill = p.faint_bg;
    visuals.widgets.inactive.weak_bg_fill = p.faint_bg;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, p.border);
    visuals.widgets.hovered.bg_fill = p.border;
    visuals.widgets.hovered.weak_bg_fill = p.border;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.active.bg_fill = p.border;
    visuals.widgets.active.weak_bg_fill = p.border;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.open.bg_fill = p.faint_bg;

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(11.0, 5.0);
    style.spacing.menu_margin = egui::Margin::same(8.0);
    style.spacing.interact_size.y = 26.0;
    style.visuals.widgets.noninteractive.rounding = Rounding::same(6.0);
    style.visuals.widgets.inactive.rounding = Rounding::same(6.0);
    style.visuals.widgets.hovered.rounding = Rounding::same(6.0);
    style.visuals.widgets.active.rounding = Rounding::same(6.0);
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(14.0));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(14.0));
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(20.0));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(11.5));
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::monospace(editor_font_size));
    ctx.set_style(style);
}
