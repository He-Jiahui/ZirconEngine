#[cfg(test)]
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_STATUS_NO_ERRORS_FILL:
    [u8; 4] = PALETTE.success;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WorkbenchStatusControlPalette {
    pub flat_transparent: [u8; 4],
    pub surface_disabled: [u8; 4],
    pub surface_pressed: [u8; 4],
    pub surface_selected: [u8; 4],
    pub surface_hover: [u8; 4],
    pub border_disabled: [u8; 4],
    pub focus_ring: [u8; 4],
    pub text: [u8; 4],
    pub text_muted: [u8; 4],
    pub text_disabled: [u8; 4],
    pub success: [u8; 4],
    pub warning: [u8; 4],
    pub info: [u8; 4],
    pub error: [u8; 4],
    pub icon_color: [u8; 4],
    pub icon_muted: [u8; 4],
    pub no_errors_fill: [u8; 4],
}

pub(super) fn workbench_status_control_palette() -> WorkbenchStatusControlPalette {
    workbench_status_control_palette_from_host(current_host_palette())
}

pub(super) fn workbench_status_control_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchStatusControlPalette {
    WorkbenchStatusControlPalette {
        flat_transparent: [0, 0, 0, 0],
        surface_disabled: palette.surface_disabled,
        surface_pressed: palette.surface_pressed,
        surface_selected: palette.surface_selected,
        surface_hover: palette.surface_hover,
        border_disabled: palette.border_disabled,
        focus_ring: palette.focus_ring,
        text: palette.text,
        text_muted: palette.text_muted,
        text_disabled: palette.text_disabled,
        success: palette.success,
        warning: palette.warning,
        info: palette.info,
        error: palette.error,
        icon_color: palette.text_muted,
        icon_muted: palette.text_disabled,
        no_errors_fill: palette.success,
    }
}
