use super::super::super::paint_theme::{HostMaterialPalette, current_host_palette};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchCommandPalettePalette
{
    pub panel_surface: [u8; 4],
    pub panel_border: [u8; 4],
    pub search_surface: [u8; 4],
    pub search_idle_border: [u8; 4],
    pub search_focus_border: [u8; 4],
    pub search_icon: [u8; 4],
    pub text: [u8; 4],
    pub placeholder: [u8; 4],
    pub empty_text: [u8; 4],
    pub match_indicator: [u8; 4],
    pub match_indicator_disabled: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn command_palette_palette()
-> WorkbenchCommandPalettePalette {
    command_palette_palette_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn command_palette_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchCommandPalettePalette {
    WorkbenchCommandPalettePalette {
        panel_surface: palette.popup,
        panel_border: palette.border,
        search_surface: palette.surface_inset,
        search_idle_border: palette.border,
        search_focus_border: palette.focus_ring,
        search_icon: palette.text_muted,
        text: palette.text,
        placeholder: palette.text_muted,
        empty_text: palette.text_muted,
        match_indicator: palette.accent,
        match_indicator_disabled: palette.text_disabled,
    }
}
