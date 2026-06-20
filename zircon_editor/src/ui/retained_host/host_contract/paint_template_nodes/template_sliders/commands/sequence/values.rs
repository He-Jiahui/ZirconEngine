use super::super::super::text::{push_slider_range_min_value, push_slider_value};
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::style_selector::WorkbenchSliderStyle;

pub(super) fn push_sequence_values(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    value_rect: Option<&FrameRect>,
    track_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    percent: f32,
    range_min_percent: Option<f32>,
    style: &WorkbenchSliderStyle,
    opacity: f32,
) {
    if let Some(range_min_percent) = range_min_percent {
        push_slider_range_min_value(
            commands,
            style,
            rect,
            track_rect,
            clip,
            order,
            range_min_percent,
            opacity,
        );
    }
    if let Some(value_rect) = value_rect {
        push_slider_value(
            commands, node, style, value_rect, clip, order, percent, opacity,
        );
    }
}
