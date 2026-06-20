use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tooltip_arrow(
    commands: &mut Vec<HostPaintCommand>,
    bubble: &FrameRect,
    clip: &FrameRect,
    order: i32,
    arrow_size: f32,
    fill: [u8; 4],
    border: [u8; 4],
    opacity: f32,
) {
    let size = arrow_size.round().max(4.0) as u32;
    let x = bubble.x + bubble.width * 0.5 - size as f32 * 0.5;
    let y = bubble.y + bubble.height - 1.0;
    push_diamond(commands, x, y, size, clip, order, border, opacity);

    let fill_size = size.saturating_sub(2).max(2);
    push_diamond(
        commands,
        x + 1.0,
        y + 1.0,
        fill_size,
        clip,
        order + 1,
        fill,
        opacity,
    );
}

fn push_diamond(
    commands: &mut Vec<HostPaintCommand>,
    x: f32,
    y: f32,
    size: u32,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let size = size.max(2);
    let center = (size as f32 - 1.0) * 0.5;
    for row in 0..size {
        let distance = (row as f32 - center).abs();
        let row_width = (size as f32 - distance * 2.0).ceil().max(1.0);
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: x + (size as f32 - row_width) * 0.5,
                y: y + row as f32,
                width: row_width,
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
    }
}
