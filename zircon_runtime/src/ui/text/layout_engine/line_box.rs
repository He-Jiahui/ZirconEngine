use crate::graphics::text::layout::{
    justify_line_advances, measure_line_width, measured_grapheme_widths, tab_aligned_advances,
};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextAlign, UiTextDirection};

use super::candidate_line::CandidateLine;
use super::direction::is_rtl_direction;

pub(super) const MIN_TEXT_FONT_SIZE: f32 = 1.0;

pub(super) fn resolve_line_widths(
    line: &CandidateLine,
    style: &UiResolvedStyle,
    frame_width: f32,
    is_last_line: bool,
) -> (f32, Vec<f32>, f32) {
    let natural_advances = line_grapheme_advances(&line.text, style);
    let natural_width = natural_advances.iter().sum();
    if should_justify_line(line, style, frame_width, is_last_line) {
        if let Some(justified_advances) =
            justify_line_advances(&line.text, &natural_advances, natural_width, frame_width)
        {
            let justified_width = justified_advances.iter().sum();
            return (justified_width, justified_advances, frame_width);
        }
    }

    let line_width = natural_width.min(frame_width);
    (natural_width, natural_advances, line_width)
}

fn line_grapheme_advances(text: &str, style: &UiResolvedStyle) -> Vec<f32> {
    let advances = measured_grapheme_widths(text, style);
    tab_aligned_advances(text, &advances, style, measure_line_width(" ", style))
}

fn should_justify_line(
    line: &CandidateLine,
    style: &UiResolvedStyle,
    frame_width: f32,
    is_last_line: bool,
) -> bool {
    matches!(style.text_align, UiTextAlign::Justify)
        && !is_last_line
        && frame_width > 0.0
        && !line.text.trim().is_empty()
        && !line.ellipsized
}

pub(super) fn text_advance(font_size: f32) -> f32 {
    (font_size.max(MIN_TEXT_FONT_SIZE) * 0.56).max(MIN_TEXT_FONT_SIZE)
}

pub(super) fn aligned_x(
    frame: UiFrame,
    line_width: f32,
    align: UiTextAlign,
    direction: UiTextDirection,
) -> f32 {
    match align {
        UiTextAlign::Left => frame.x,
        UiTextAlign::Center => frame.x + (frame.width - line_width) * 0.5,
        UiTextAlign::Right => frame.right() - line_width,
        UiTextAlign::Start if is_rtl_direction(direction) => frame.right() - line_width,
        UiTextAlign::Start => frame.x,
        UiTextAlign::End if is_rtl_direction(direction) => frame.x,
        UiTextAlign::End => frame.right() - line_width,
        UiTextAlign::Justify => frame.x,
    }
}
