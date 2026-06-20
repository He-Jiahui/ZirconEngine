use super::super::super::track::{push_slider_ticks, push_slider_track};
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::style_selector::WorkbenchSliderStyle;

pub(super) fn push_sequence_track(
    commands: &mut Vec<HostPaintCommand>,
    track_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    percent: f32,
    range_min_percent: Option<f32>,
    tick_count: Option<usize>,
    style: &WorkbenchSliderStyle,
    opacity: f32,
) {
    push_slider_track(
        commands,
        style,
        track_rect,
        clip,
        order,
        percent,
        range_min_percent,
        opacity,
    );
    if let Some(tick_count) = tick_count {
        push_slider_ticks(
            commands,
            track_rect,
            clip,
            order + 2,
            tick_count,
            style,
            opacity,
        );
    }
}
