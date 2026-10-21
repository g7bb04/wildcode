use crate::config::ThemeChoice;
use crate::highlight;
use crate::theme::palette;
use eframe::egui::{
    self, FontId, FontSelection, Label, RichText, Rounding, ScrollArea, Stroke, TextEdit,
    TextWrapMode,
};

pub fn show(
    ui: &mut egui::Ui,
    code: &mut String,
    font_size: f32,
    theme: ThemeChoice,
) -> egui::Response {
    let p = palette(theme);
    let font_id = FontId::monospace(font_size);
    let line_count = visible_lines(code);
    let digits = line_count.to_string().len();
    let mut numbers = String::with_capacity(line_count * (digits + 1));
    for line in 1..=line_count {
        if line > 1 {
            numbers.push('\n');
        }
        numbers.push_str(&format!("{line:>digits$}"));
    }

    let available = ui.available_size();
    let row_height = ui.fonts(|fonts| fonts.row_height(&font_id));
    let desired_rows = ((available.y - 10.0) / row_height).floor() as usize;
    let desired_rows = desired_rows.clamp(8, 120);
    let editor_width = (available.x - (digits as f32 * font_size * 0.62) - 44.0).max(360.0);

    let output = egui::Frame::none()
        .fill(p.editor_bg)
        .stroke(Stroke::new(1.0_f32, p.border))
        .rounding(Rounding::same(8.0))
        .inner_margin(egui::Margin::same(0.0))
        .show(ui, |ui| {
            ui.set_min_height(available.y.max(120.0));
            ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    let response = ui
                        .horizontal_top(|ui| {
                            ui.add_space(8.0);
                            ui.vertical(|ui| {
                                ui.add_space(4.0);
                                ui.add(
                                    Label::new(
                                        RichText::new(&numbers)
                                            .font(font_id.clone())
                                            .color(p.gutter),
                                    )
                                    .selectable(false),
                                );
                            });

                            ui.add_space(8.0);
                            let mut layouter = |ui: &egui::Ui, text: &str, _wrap_width: f32| {
                                let job = highlight::code_layout(ui, text, font_size, theme);
                                ui.fonts(|fonts| fonts.layout_job(job))
                            };
                            ui.add(
                                TextEdit::multiline(code)
                                    .id(egui::Id::new("wild_code_editor"))
                                    .code_editor()
                                    .font(FontSelection::FontId(font_id.clone()))
                                    .margin(egui::Margin::symmetric(8.0, 4.0))
                                    .desired_width(editor_width)
                                    .desired_rows(desired_rows)
                                    .frame(false)
                                    .lock_focus(true)
                                    .layouter(&mut layouter),
                            )
                        })
                        .inner;

                    ui.add_space(8.0);
                    response
                })
                .inner
        })
        .inner;

    output
}

fn visible_lines(code: &str) -> usize {
    code.matches('\n').count() + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_trailing_line() {
        assert_eq!(visible_lines(""), 1);
        assert_eq!(visible_lines("a"), 1);
        assert_eq!(visible_lines("a\n"), 2);
        assert_eq!(visible_lines("a\nb\n"), 3);
    }
}
