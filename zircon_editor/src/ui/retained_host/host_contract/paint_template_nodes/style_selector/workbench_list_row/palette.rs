use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchListRowPalette
{
    pub marked_surface: [u8; 4],
    pub pressed_surface: [u8; 4],
    pub hot_surface: [u8; 4],
    pub focus_border: [u8; 4],
    pub text: [u8; 4],
    pub text_muted: [u8; 4],
    pub text_disabled: [u8; 4],
    pub marked_adornment: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_list_row_palette(
) -> WorkbenchListRowPalette {
    workbench_list_row_palette_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_list_row_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchListRowPalette {
    WorkbenchListRowPalette {
        marked_surface: palette.surface_pressed,
        pressed_surface: palette.surface_pressed,
        hot_surface: palette.surface_hover,
        focus_border: palette.focus_ring,
        text: palette.text,
        text_muted: palette.text_muted,
        text_disabled: palette.text_disabled,
        marked_adornment: palette.focus_ring,
    }
}
