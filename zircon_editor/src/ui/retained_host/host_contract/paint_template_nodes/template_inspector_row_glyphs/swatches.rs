use super::super::super::data::FrameRect;
use super::super::super::paint_theme::{
    HostControlMetrics, HostMaterialPalette, current_host_metrics, current_host_palette,
};
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;

const INSPECTOR_SWATCH_SIZE: f32 = 12.0;
const INSPECTOR_MATERIAL_ICON: &str = "zircon_editor_shell/inspector/material.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_inspector_swatch(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !can_fit_inspector_swatch(rect) {
        return;
    }
    let swatch = FrameRect {
        x: rect.x + (rect.width - INSPECTOR_SWATCH_SIZE).max(0.0) * 0.5,
        y: rect.y + (rect.height - INSPECTOR_SWATCH_SIZE).max(0.0) * 0.5,
        width: INSPECTOR_SWATCH_SIZE,
        height: INSPECTOR_SWATCH_SIZE,
    };
    if push_icon_asset_pixels(
        commands,
        INSPECTOR_MATERIAL_ICON,
        &swatch,
        clip,
        order,
        None,
        opacity,
    ) {
        return;
    }
    let (background, border) = material_swatch_colors(current_host_palette());
    let (border_width, corner_radius) = material_swatch_shape(current_host_metrics());
    commands.push(HostPaintCommand::quad(
        swatch,
        Some(clip.clone()),
        order,
        Some(background),
        Some(border),
        border_width,
        corner_radius,
        opacity,
    ));
}

fn material_swatch_colors(palette: HostMaterialPalette) -> ([u8; 4], [u8; 4]) {
    (palette.accent, palette.border)
}

fn material_swatch_shape(metrics: HostControlMetrics) -> (f32, f32) {
    (
        metrics.border_width,
        metrics.radius_control.min(INSPECTOR_SWATCH_SIZE * 0.5),
    )
}

fn can_fit_inspector_swatch(rect: &FrameRect) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width >= INSPECTOR_SWATCH_SIZE
        && rect.height >= INSPECTOR_SWATCH_SIZE
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};

    #[test]
    fn inspector_material_swatch_skips_an_undersized_slot() {
        let rect = FrameRect {
            x: 4.0,
            y: 6.0,
            width: 11.0,
            height: 11.0,
        };
        let mut commands = Vec::new();

        push_inspector_swatch(&mut commands, &rect, &rect, 0, 1.0);

        assert!(commands.is_empty());
    }

    #[test]
    fn inspector_material_swatch_fallback_projects_host_palette_and_metrics() {
        let mut palette = PALETTE;
        palette.accent = [11, 12, 13, 255];
        palette.border = [21, 22, 23, 255];
        let metrics = HostControlMetrics {
            border_width: 1.5,
            radius_control: 4.0,
            ..METRICS
        };

        assert_eq!(
            material_swatch_colors(palette),
            ([11, 12, 13, 255], [21, 22, 23, 255])
        );
        assert_eq!(material_swatch_shape(metrics), (1.5, 4.0));
    }
}
