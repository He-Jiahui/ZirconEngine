use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_inset_rect(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    inset: f32,
    opacity: f32,
) {
    let x = rect.x + inset;
    let y = rect.y + inset;
    let width = (rect.width - inset * 2.0).max(1.0);
    let height = (rect.height - inset * 2.0).max(1.0);
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x,
            y,
            width,
            height: 1.0,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x,
            y: y + height - 1.0,
            width,
            height: 1.0,
        },
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x,
            y,
            width: 1.0,
            height,
        },
        Some(clip.clone()),
        order + 2,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: x + width - 1.0,
            y,
            width: 1.0,
            height,
        },
        Some(clip.clone()),
        order + 3,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
}
