use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::FrameRect;
use super::super::super::paint_theme::{HostControlMetrics, HostMaterialPalette};
use super::super::render_commands::HostPaintCommand;
use super::geometry::inset_rect;

#[allow(clippy::too_many_arguments)]
pub(super) fn push_settings_field_control(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    value_text: String,
    focused: bool,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: HostMaterialPalette,
    metrics: HostControlMetrics,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(palette.surface_inset),
        Some(if focused {
            palette.accent
        } else {
            palette.border
        }),
        metrics.border_width,
        metrics.radius_control,
        opacity,
    ));
    commands.push(HostPaintCommand::text(
        inset_rect(rect, metrics.gap_m, 0.0),
        Some(clip.clone()),
        order + 1,
        value_text,
        palette.text,
        metrics.font_body,
        metrics.line_height(metrics.font_body),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
