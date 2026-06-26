use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_text::measure_fallback_text_width;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;
use zircon_runtime_interface::ui::style::ButtonInteractionState;

pub(super) fn button_label_font_size(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size.min(rect.height.max(1.0))
    } else {
        METRICS.font_body
    }
}

pub(super) fn button_label_line_height(font_size: f32) -> f32 {
    METRICS.line_height(font_size)
}

pub(super) fn measured_label_width(label: &str, font_size: f32) -> f32 {
    measure_fallback_text_width(label, font_size) + METRICS.text_clip_guard
}

pub(super) fn content_offset_y(interaction: ButtonInteractionState) -> f32 {
    match interaction {
        ButtonInteractionState::Pressed => METRICS.button_pressed_offset_y,
        _ => 0.0,
    }
}
