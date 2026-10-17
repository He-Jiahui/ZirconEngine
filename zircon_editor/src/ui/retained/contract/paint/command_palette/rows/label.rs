//! 候选主标题的单行绘制入口；先判断行可见性，再把借用文本交给拥有内容的命令。

use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::{command_palette_metrics, row_label_rect};

#[path = "label/style.rs"]
mod style;

use style::command_row_label_text_style;

pub(super) fn push_command_row_label(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    text: &str,
    color: [u8; 4],
    opacity: f32,
) {
    push_command_row_label_with_detail(
        commands, row_rect, clip, order, text, false, color, opacity,
    );
}

pub(super) fn push_command_row_label_with_detail(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    text: &str,
    has_detail: bool,
    color: [u8; 4],
    opacity: f32,
) {
    if text.is_empty() || intersect(row_rect, clip).is_none() {
        return;
    }
    let metrics = command_palette_metrics();
    let style = command_row_label_text_style(color, &metrics);
    let mut slot = row_label_rect(row_rect);
    if has_detail {
        let detail = super::super::layout::row_detail_rect(row_rect);
        slot.width = slot
            .width
            .min((detail.x - metrics.row_text_x - slot.x).max(0.0));
    }
    let Some(slot_clip) = intersect(&slot, clip) else {
        return;
    };
    commands.push(HostPaintCommand::text(
        slot,
        Some(slot_clip),
        order,
        text.to_string(),
        style.color,
        style.font_size,
        style.line_height,
        style.paint_style,
        opacity,
    ));
}
