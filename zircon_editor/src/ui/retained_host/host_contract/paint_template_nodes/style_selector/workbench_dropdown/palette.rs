use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchDropdownPalette
{
    pub surface: [u8; 4],
    pub hover_surface: [u8; 4],
    pub open_surface: [u8; 4],
    pub disabled_surface: [u8; 4],
    pub border: [u8; 4],
    pub focus_border: [u8; 4],
    pub hover_border: [u8; 4],
    pub disabled_border: [u8; 4],
    pub error_border: [u8; 4],
    pub text: [u8; 4],
    pub placeholder: [u8; 4],
    pub disabled_text: [u8; 4],
    pub chevron: [u8; 4],
    pub active_chevron: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_dropdown_palette(
) -> WorkbenchDropdownPalette {
    workbench_dropdown_palette_from_host(current_host_palette())
}

pub(super) fn workbench_dropdown_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchDropdownPalette {
    WorkbenchDropdownPalette {
        surface: palette.surface_inset,
        hover_surface: palette.surface_hover,
        open_surface: palette.accent_soft,
        disabled_surface: palette.surface_disabled,
        border: palette.border,
        focus_border: palette.focus_ring,
        hover_border: palette.border,
        disabled_border: palette.border_disabled,
        error_border: palette.error,
        text: palette.text,
        placeholder: palette.text_disabled,
        disabled_text: palette.text_disabled,
        chevron: palette.text_muted,
        active_chevron: palette.focus_ring,
    }
}
