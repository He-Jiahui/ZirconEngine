use super::super::super::super::palette::WorkbenchCommandPalettePalette;

pub(super) struct CommandPaletteSearchIconStyle {
    pub tint: Option<[u8; 4]>,
}

pub(super) fn command_palette_search_icon_style(
    palette: &WorkbenchCommandPalettePalette,
) -> CommandPaletteSearchIconStyle {
    CommandPaletteSearchIconStyle {
        tint: Some(palette.search_icon),
    }
}
