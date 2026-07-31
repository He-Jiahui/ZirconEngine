use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchSliderStyle;
use super::super::super::template_slider_geometry::workbench_slider_metrics;
use zircon_runtime_interface::ui::surface::ui_slider_tick_count_for_track;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_slider_ticks(
    commands: &mut Vec<HostPaintCommand>,
    track_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    tick_count: usize,
    style: &WorkbenchSliderStyle,
    opacity: f32,
) {
    let tick_count = ui_slider_tick_count_for_track(tick_count, track_rect.width);
    if tick_count < 2 {
        return;
    }
    let metrics = workbench_slider_metrics();
    let last = tick_count - 1;
    for index in 0..tick_count {
        let fraction = index as f32 / last as f32;
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: track_rect.x + track_rect.width * fraction - metrics.tick_width * 0.5,
                y: track_rect.y + metrics.tick_offset_y,
                width: metrics.tick_width,
                height: metrics.tick_height,
            },
            Some(clip.clone()),
            order,
            Some(style.tick),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}
