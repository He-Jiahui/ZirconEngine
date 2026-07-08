use super::super::super::super::layout::WorkbenchCommandPaletteMetrics;
use super::super::super::super::palette::WorkbenchCommandPalettePalette;

pub(super) struct CommandPaletteSearchSurfaceStyle {
    pub fill: [u8; 4],
    pub border: [u8; 4],
    pub border_width: f32,
    pub radius: f32,
}

pub(super) fn command_palette_search_surface_style(
    palette: &WorkbenchCommandPalettePalette,
    metrics: &WorkbenchCommandPaletteMetrics,
    focused: bool,
) -> CommandPaletteSearchSurfaceStyle {
    CommandPaletteSearchSurfaceStyle {
        fill: palette.search_surface,
        border: search_border_color(palette, focused),
        border_width: metrics.border_width,
        radius: metrics.search_radius,
    }
}

fn search_border_color(palette: &WorkbenchCommandPalettePalette, focused: bool) -> [u8; 4] {
    if focused {
        palette.search_focus_border
    } else {
        palette.search_idle_border
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_palette() -> WorkbenchCommandPalettePalette {
        WorkbenchCommandPalettePalette {
            panel_surface: [1, 2, 3, 255],
            panel_border: [4, 5, 6, 255],
            search_surface: [7, 8, 9, 255],
            search_idle_border: [10, 11, 12, 255],
            search_focus_border: [13, 14, 15, 255],
            search_icon: [16, 17, 18, 255],
            text: [19, 20, 21, 255],
            placeholder: [22, 23, 24, 255],
            empty_text: [25, 26, 27, 255],
            match_indicator: [28, 29, 30, 255],
            match_indicator_disabled: [31, 32, 33, 255],
        }
    }

    #[test]
    fn command_palette_search_border_uses_focus_ring_only_when_focused() {
        let palette = test_palette();

        assert_eq!(
            search_border_color(&palette, false),
            palette.search_idle_border
        );
        assert_eq!(
            search_border_color(&palette, true),
            palette.search_focus_border
        );
    }
}
