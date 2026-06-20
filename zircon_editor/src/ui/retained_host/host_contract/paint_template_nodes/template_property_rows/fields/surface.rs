use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::PROPERTY_FIELD_RADIUS;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

pub(super) fn push_property_value_field_surface(
    commands: &mut Vec<HostPaintCommand>,
    field_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    border: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        field_rect.clone(),
        Some(clip.clone()),
        order,
        Some(PALETTE.surface_inset),
        Some(border),
        1.0,
        PROPERTY_FIELD_RADIUS,
        opacity,
    ));
}
