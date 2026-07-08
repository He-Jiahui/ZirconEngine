use super::super::super::layout::WorkbenchCommandPaletteMetrics;
use super::super::super::palette::WorkbenchCommandPalettePalette;

pub(super) struct CommandPalettePanelSurfaceStyle {
    pub fill: [u8; 4],
    pub border: [u8; 4],
    pub border_width: f32,
    pub radius: f32,
}

pub(super) fn command_palette_panel_surface_style(
    palette: &WorkbenchCommandPalettePalette,
    metrics: &WorkbenchCommandPaletteMetrics,
) -> CommandPalettePanelSurfaceStyle {
    CommandPalettePanelSurfaceStyle {
        fill: palette.panel_surface,
        border: palette.panel_border,
        border_width: metrics.border_width,
        radius: metrics.panel_radius,
    }
}
