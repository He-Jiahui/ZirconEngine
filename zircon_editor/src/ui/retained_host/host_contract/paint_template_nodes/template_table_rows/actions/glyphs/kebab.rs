use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;

pub(super) fn push_table_kebab(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    for y in [3.0, 6.0, 9.0] {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x + 6.0,
                y: rect.y + y,
                width: 2.0,
                height: 2.0,
            },
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}
