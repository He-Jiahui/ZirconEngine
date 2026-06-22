use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchSliderStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_slider_ticks(
    commands: &mut Vec<HostPaintCommand>,
    track_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    tick_count: usize,
    style: &WorkbenchSliderStyle,
    opacity: f32,
) {
    if tick_count < 2 {
        return;
    }
    let last = tick_count - 1;
    for index in 0..tick_count {
        let fraction = index as f32 / last as f32;
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: track_rect.x + track_rect.width * fraction - 0.5,
                y: track_rect.y + 8.0,
                width: 1.0,
                height: 4.0,
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
