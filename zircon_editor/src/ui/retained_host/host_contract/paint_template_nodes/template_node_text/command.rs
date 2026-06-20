use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_text_command(
    commands: &mut Vec<HostPaintCommand>,
    text_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: String,
    color: [u8; 4],
    font_size: f32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: text_rect.x,
            y: text_rect.y,
            width: text_rect.width,
            height: text_rect.height,
        },
        Some(clip.clone()),
        order,
        label,
        color,
        font_size,
        font_size * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
