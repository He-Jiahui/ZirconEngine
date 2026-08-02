use crate::ui::retained_host::host_contract::paint_theme::{
    HostMaterialPalette, current_host_palette,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchIconButtonPalette
{
    pub normal: [u8; 4],
    pub muted: [u8; 4],
    pub panel_surface: [u8; 4],
    pub panel_border: [u8; 4],
    pub surface_disabled: [u8; 4],
    pub error_container: [u8; 4],
    pub surface_selected: [u8; 4],
    pub surface_pressed: [u8; 4],
    pub surface_hover: [u8; 4],
    pub surface: [u8; 4],
    pub border_disabled: [u8; 4],
    pub error: [u8; 4],
    pub focus_ring: [u8; 4],
    pub border: [u8; 4],
    pub text_disabled: [u8; 4],
    pub text: [u8; 4],
    pub accent: [u8; 4],
    pub shell_background: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_icon_button_palette()
-> WorkbenchIconButtonPalette {
    workbench_icon_button_palette_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_icon_button_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchIconButtonPalette {
    WorkbenchIconButtonPalette {
        normal: palette.text,
        muted: palette.text_muted,
        panel_surface: palette.surface_pressed,
        panel_border: palette.border,
        surface_disabled: palette.surface_disabled,
        error_container: palette.error_container,
        surface_selected: palette.surface_selected,
        surface_pressed: palette.surface_pressed,
        surface_hover: palette.surface_hover,
        surface: palette.surface,
        border_disabled: palette.border_disabled,
        error: palette.error,
        focus_ring: palette.focus_ring,
        border: palette.border,
        text_disabled: palette.text_disabled,
        text: palette.text,
        accent: palette.accent,
        shell_background: palette.shell_background,
    }
}
