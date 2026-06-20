use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const PROPERTY_FONT_SIZE: f32 = 11.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_command(
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    text: &str,
    color: [u8; 4],
    opacity: f32,
) -> HostPaintCommand {
    HostPaintCommand::text(
        rect,
        Some(clip.clone()),
        order,
        text.to_string(),
        color,
        PROPERTY_FONT_SIZE,
        PROPERTY_FONT_SIZE * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    )
}
