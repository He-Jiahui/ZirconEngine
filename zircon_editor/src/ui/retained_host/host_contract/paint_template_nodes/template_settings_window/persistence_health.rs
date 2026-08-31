use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::{HostControlMetrics, HostMaterialPalette};
use super::super::super::settings_window_geometry::SettingsWindowLayout;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::geometry::inset_rect;

const PERSISTENCE_RETRY_ICON: &str = "refresh-outline";

pub(super) fn push_persistence_health(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    layout: &SettingsWindowLayout,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: HostMaterialPalette,
    metrics: HostControlMetrics,
) {
    if node.settings_persistence_retry_scope.is_empty()
        || node.settings_persistence_status_text.is_empty()
    {
        return;
    }
    commands.push(HostPaintCommand::text(
        layout.persistence_status.clone(),
        Some(clip.clone()),
        order,
        node.settings_persistence_status_text.to_string(),
        palette.warning,
        metrics.font_body,
        metrics.line_height(metrics.font_body),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        layout.persistence_retry.clone(),
        Some(clip.clone()),
        order,
        Some(palette.warning_container),
        Some(palette.warning),
        metrics.border_width,
        metrics.radius_control,
        opacity,
    ));
    let icon = inset_rect(&layout.persistence_retry, metrics.gap_m, metrics.gap_m);
    push_icon_asset_pixels(
        commands,
        PERSISTENCE_RETRY_ICON,
        &icon,
        clip,
        order + 1,
        Some(palette.warning),
        opacity,
    );
}
