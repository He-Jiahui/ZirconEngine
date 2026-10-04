use zircon_runtime::ui::surface::measure_text_size;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextAlign, UiTextDirection, UiTextRunPaintStyle,
};

use super::super::super::super::{
    data::FrameRect,
    paint_text::{font_face_for_paint_style, runtime_text_style_for_face},
};

mod metrics;

use self::metrics::{center_aligned_text_x, measured_text_width, right_aligned_text_x};

const STRONG_TEXT_FONT_WEIGHT_THRESHOLD: u16 = 600;

/// 对齐位置按宿主实际测量宽度计算；Runtime 字重映射到当前宿主字体偏好，绘制阶段不重新推断。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn aligned_text_x(
    frame: &FrameRect,
    text: &str,
    style: &UiResolvedStyle,
) -> f32 {
    let measure_style = retained_runtime_measure_style(style);
    let measured_width = measured_text_width(measure_text_size(text, &measure_style).width);
    match resolved_text_align(style.text_align, style.text_direction) {
        UiTextAlign::Left | UiTextAlign::Justify => frame.x,
        UiTextAlign::Center => center_aligned_text_x(frame.x, frame.width, measured_width),
        UiTextAlign::Right => right_aligned_text_x(frame.x, frame.width, measured_width),
        UiTextAlign::Start | UiTextAlign::End => frame.x,
    }
}

fn retained_runtime_measure_style(style: &UiResolvedStyle) -> UiResolvedStyle {
    let paint_style = text_paint_style_from_resolved_style(style);
    let mut measure_style = runtime_text_style_for_face(
        font_face_for_paint_style(paint_style),
        style.font_size,
        style.line_height,
        style.wrap,
        style.text_overflow,
    );
    measure_style.text_align = style.text_align;
    measure_style.text_direction = style.text_direction;
    measure_style.text_writing_mode = style.text_writing_mode;
    measure_style.text_render_mode = style.text_render_mode;
    measure_style
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_paint_style_from_resolved_style(
    style: &UiResolvedStyle,
) -> UiTextRunPaintStyle {
    text_paint_style_from_font_weight(style.font_weight)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_paint_style_from_font_weight(
    font_weight: u16,
) -> UiTextRunPaintStyle {
    UiTextRunPaintStyle {
        strong: UiResolvedStyle::normalized_font_weight(font_weight)
            >= STRONG_TEXT_FONT_WEIGHT_THRESHOLD,
        ..UiTextRunPaintStyle::default()
    }
}

fn resolved_text_align(align: UiTextAlign, direction: UiTextDirection) -> UiTextAlign {
    match align {
        UiTextAlign::Start => match direction {
            UiTextDirection::RightToLeft => UiTextAlign::Right,
            UiTextDirection::Auto | UiTextDirection::LeftToRight | UiTextDirection::Mixed => {
                UiTextAlign::Left
            }
        },
        UiTextAlign::End => match direction {
            UiTextDirection::RightToLeft => UiTextAlign::Left,
            UiTextDirection::Auto | UiTextDirection::LeftToRight | UiTextDirection::Mixed => {
                UiTextAlign::Right
            }
        },
        UiTextAlign::Left | UiTextAlign::Center | UiTextAlign::Right | UiTextAlign::Justify => {
            align
        }
    }
}

#[cfg(test)]
#[path = "tests/text.rs"]
mod tests;
