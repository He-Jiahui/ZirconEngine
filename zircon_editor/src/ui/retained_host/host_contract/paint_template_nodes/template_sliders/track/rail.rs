use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchSliderStyle;
use super::super::super::template_slider_geometry::{slider_fill_span, SLIDER_TRACK_RADIUS};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_slider_track(
    commands: &mut Vec<HostPaintCommand>,
    style: &WorkbenchSliderStyle,
    track_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    percent: f32,
    range_min_percent: Option<f32>,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        track_rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.track),
        None,
        0.0,
        SLIDER_TRACK_RADIUS,
        opacity,
    ));

    let (fill_start, fill_end) = slider_fill_span(percent, range_min_percent);
    let fill_width = (track_rect.width * (fill_end - fill_start)).max(0.0);
    if fill_width <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: track_rect.x + track_rect.width * fill_start,
            y: track_rect.y,
            width: fill_width.max(1.0),
            height: track_rect.height,
        },
        Some(clip.clone()),
        order + 1,
        Some(style.fill),
        None,
        0.0,
        SLIDER_TRACK_RADIUS,
        opacity,
    ));
}
