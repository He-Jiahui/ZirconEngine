use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::segments::push_segments;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_right_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        &[
            FrameRect {
                x: rect.x + 5.0,
                y: rect.y + 3.0,
                width: 2.0,
                height: 3.0,
            },
            FrameRect {
                x: rect.x + 7.0,
                y: rect.y + 6.0,
                width: 2.0,
                height: 2.0,
            },
            FrameRect {
                x: rect.x + 5.0,
                y: rect.y + 8.0,
                width: 2.0,
                height: 3.0,
            },
        ],
        clip,
        order,
        color,
        opacity,
    );
}
