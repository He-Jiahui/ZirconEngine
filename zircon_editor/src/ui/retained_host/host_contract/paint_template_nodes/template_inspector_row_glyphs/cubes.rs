use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::segments::push_inspector_segments;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_inspector_cube_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_inspector_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            FrameRect {
                x: rect.x + 3.0,
                y: rect.y + 3.0,
                width: rect.width - 6.0,
                height: rect.height - 6.0,
            },
            FrameRect {
                x: rect.x + 5.0,
                y: rect.y + 1.0,
                width: rect.width - 6.0,
                height: 2.0,
            },
            FrameRect {
                x: rect.x + rect.width - 3.0,
                y: rect.y + 4.0,
                width: 2.0,
                height: rect.height - 7.0,
            },
        ],
        1.0,
    );
}
