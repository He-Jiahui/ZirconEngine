use super::super::super::thumb::push_slider_thumb;
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::style_selector::WorkbenchSliderStyle;

pub(super) fn push_sequence_thumbs(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    track_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    percent: f32,
    range_min_percent: Option<f32>,
    style: &WorkbenchSliderStyle,
    opacity: f32,
) {
    if let Some(range_min_percent) = range_min_percent {
        push_slider_thumb(
            commands,
            node,
            style,
            track_rect,
            clip,
            order + 3,
            range_min_percent,
            opacity,
        );
    }
    push_slider_thumb(
        commands,
        node,
        style,
        track_rect,
        clip,
        order + 4,
        percent,
        opacity,
    );
}
