use super::super::super::super::layout::WorkbenchCommandPaletteMetrics;
use super::super::super::super::palette::WorkbenchCommandPalettePalette;
use super::super::super::super::text::command_palette_text_style;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) struct CommandPaletteSearchTextStyle {
    pub color: [u8; 4],
    pub font_size: f32,
    pub line_height: f32,
    pub paint_style: UiTextRunPaintStyle,
}

pub(super) fn command_palette_search_text_style(
    palette: &WorkbenchCommandPalettePalette,
    metrics: &WorkbenchCommandPaletteMetrics,
    placeholder: bool,
) -> CommandPaletteSearchTextStyle {
    CommandPaletteSearchTextStyle {
        color: search_text_color(palette, placeholder),
        font_size: metrics.font_size,
        line_height: metrics.line_height,
        paint_style: command_palette_text_style(),
    }
}

fn search_text_color(palette: &WorkbenchCommandPalettePalette, placeholder: bool) -> [u8; 4] {
    if placeholder {
        palette.placeholder
    } else {
        palette.text
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
    fn command_palette_search_text_color_uses_placeholder_only_for_placeholder_runs() {
        let palette = test_palette();

        assert_eq!(search_text_color(&palette, true), palette.placeholder);
        assert_eq!(search_text_color(&palette, false), palette.text);
    }
}
