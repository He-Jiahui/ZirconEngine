use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_avatar_fallback_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let head_size = (rect.width.min(rect.height) * 0.24).max(2.0);
    let head = FrameRect {
        x: rect.x + (rect.width - head_size) * 0.5,
        y: rect.y + rect.height * 0.24,
        width: head_size,
        height: head_size,
    };
    commands.push(HostPaintCommand::quad(
        head.clone(),
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        head_size * 0.5,
        opacity,
    ));

    let body_width = rect.width * 0.52;
    let body_height = rect.height * 0.22;
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + (rect.width - body_width) * 0.5,
            y: rect.y + rect.height * 0.55,
            width: body_width,
            height: body_height,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        body_height * 0.5,
        opacity,
    ));
}
