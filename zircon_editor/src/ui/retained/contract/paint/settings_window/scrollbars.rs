use super::super::super::data::FrameRect;
use super::super::super::paint_theme::{HostControlMetrics, HostMaterialPalette};
use super::super::super::settings_window_geometry::SettingsWindowLayout;
use super::super::render_commands::HostPaintCommand;

pub(super) fn push_preferences_scrollbars(
    commands: &mut Vec<HostPaintCommand>,
    layout: &SettingsWindowLayout,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: HostMaterialPalette,
    metrics: HostControlMetrics,
) {
    push_scrollbar(
        commands,
        layout.category_scrollbar_track.as_ref(),
        layout.category_scrollbar_thumb.as_ref(),
        clip,
        order,
        opacity,
        palette,
        metrics,
    );
    push_scrollbar(
        commands,
        layout.setting_scrollbar_track.as_ref(),
        layout.setting_scrollbar_thumb.as_ref(),
        clip,
        order + 2,
        opacity,
        palette,
        metrics,
    );
}

fn push_scrollbar(
    commands: &mut Vec<HostPaintCommand>,
    track: Option<&FrameRect>,
    thumb: Option<&FrameRect>,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: HostMaterialPalette,
    metrics: HostControlMetrics,
) {
    let (Some(track), Some(thumb)) = (track, thumb) else {
        return;
    };
    let radius = metrics.radius_control.min(track.width * 0.5);
    commands.push(HostPaintCommand::quad(
        track.clone(),
        Some(clip.clone()),
        order,
        Some(palette.track),
        None,
        0.0,
        radius,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        thumb.clone(),
        Some(clip.clone()),
        order + 1,
        Some(palette.surface_hover),
        None,
        0.0,
        radius,
        opacity,
    ));
}
