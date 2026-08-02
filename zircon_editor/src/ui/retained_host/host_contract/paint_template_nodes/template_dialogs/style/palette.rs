use crate::ui::retained_host::host_contract::paint_theme::{
    HostMaterialPalette, current_host_palette,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchDialogPalette
{
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub active_border: [u8; 4],
    pub title: [u8; 4],
    pub body: [u8; 4],
    pub action: [u8; 4],
    pub info: [u8; 4],
    pub info_border: [u8; 4],
    pub warning: [u8; 4],
    pub warning_border: [u8; 4],
    pub error: [u8; 4],
    pub error_border: [u8; 4],
    pub disabled_surface: [u8; 4],
    pub disabled_border: [u8; 4],
    pub disabled_text: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_palette()
-> WorkbenchDialogPalette {
    dialog_palette_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchDialogPalette {
    WorkbenchDialogPalette {
        surface: palette.popup,
        border: palette.border,
        active_border: palette.focus_ring,
        title: palette.text,
        body: palette.text_muted,
        action: palette.accent,
        info: palette.info,
        info_border: palette.info_container,
        warning: palette.warning,
        warning_border: palette.warning_container,
        error: palette.error,
        error_border: palette.error_container,
        disabled_surface: palette.surface_disabled,
        disabled_border: palette.border_disabled,
        disabled_text: palette.text_disabled,
    }
}
