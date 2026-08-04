use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_row_metrics::workbench_row_palette;
use super::super::layout::property_row_metrics;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_geometry::intersect;

pub(super) fn push_property_value_field_surface(
    commands: &mut Vec<HostPaintCommand>,
    field_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    border: [u8; 4],
    opacity: f32,
) {
    if intersect(field_rect, clip).is_none() {
        return;
    }
    let metrics = property_row_metrics();
    let palette = workbench_row_palette();
    commands.push(HostPaintCommand::quad(
        field_rect.clone(),
        Some(clip.clone()),
        order,
        Some(palette.property_field_surface),
        Some(border),
        metrics.property_field_border_width,
        metrics.property_field_radius,
        opacity,
    ));
}
