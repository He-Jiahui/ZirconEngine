use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_text::measure_fallback_text_width;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_status_control_geometry::{
    status_chip_text_rect, status_font_size, status_line_height,
};
use crate::ui::retained_host::host_contract::paint_theme::METRICS;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) fn push_status_chip_text(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: &str,
    color: [u8; 4],
    opacity: f32,
) {
    let base = status_chip_text_rect(rect);
    match split_status_chip_text(label) {
        StatusChipText::LabelAndValue { label, value } => {
            let value_rect = right_aligned_text_rect(base.clone(), &value);
            let label_rect = leading_label_rect(base, &value_rect);
            push_text(commands, label_rect, clip, order, label, color, opacity);
            push_text(commands, value_rect, clip, order, value, color, opacity);
        }
        StatusChipText::Value(value) => {
            let value_rect = right_aligned_text_rect(base, &value);
            push_text(commands, value_rect, clip, order, value, color, opacity);
        }
    }
}

enum StatusChipText {
    LabelAndValue { label: String, value: String },
    Value(String),
}

fn split_status_chip_text(label: &str) -> StatusChipText {
    let label = label.trim();
    if let Some((leading, value)) = label.split_once(':') {
        let leading = leading.trim();
        let value = value.trim();
        if !leading.is_empty() && !value.is_empty() {
            return StatusChipText::LabelAndValue {
                label: format!("{leading}:"),
                value: value.to_string(),
            };
        }
    }
    StatusChipText::Value(label.to_string())
}

fn leading_label_rect(base: FrameRect, value_rect: &FrameRect) -> FrameRect {
    FrameRect {
        width: (value_rect.x - base.x - METRICS.gap_s).max(1.0),
        ..base
    }
}

fn right_aligned_text_rect(base: FrameRect, text: &str) -> FrameRect {
    let measured_width =
        measure_fallback_text_width(text, status_font_size()) + METRICS.text_clip_guard;
    let width = measured_width.min(base.width).max(1.0);
    FrameRect {
        x: base.x + (base.width - width).max(0.0),
        width,
        ..base
    }
}

fn push_text(
    commands: &mut Vec<HostPaintCommand>,
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    text: String,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::text(
        rect,
        Some(clip.clone()),
        order,
        text,
        color,
        status_font_size(),
        status_line_height(),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
