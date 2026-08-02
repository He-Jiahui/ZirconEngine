#[cfg(test)]
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::host_contract::paint_theme::{
    HostMaterialPalette, current_host_palette,
};

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SELECTION_LABEL_MUTED:
    [u8; 4] = PALETTE.text_muted;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SELECTION_MARK_IDLE_FILL:
    [u8; 4] = PALETTE.popup;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SELECTION_MARK_IDLE_BORDER:
    [u8; 4] = PALETTE.separator_strong;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_CHECKBOX_CHECKED_FILL:
    [u8; 4] = PALETTE.surface_selected;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_RADIO_CHECKED_FILL: [u8;
    4] = PALETTE.surface_pressed;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_RADIO_CHECKED_BORDER:
    [u8; 4] = PALETTE.separator_strong;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WorkbenchSelectionControlPalette {
    pub surface_disabled: [u8; 4],
    pub mark_idle_fill: [u8; 4],
    pub mark_idle_border: [u8; 4],
    pub checkbox_checked_fill: [u8; 4],
    pub radio_checked_fill: [u8; 4],
    pub radio_checked_border: [u8; 4],
    pub toggle_checked_surface: [u8; 4],
    pub toggle_pressed_surface: [u8; 4],
    pub toggle_hover_surface: [u8; 4],
    pub toggle_track: [u8; 4],
    pub toggle_checked_border: [u8; 4],
    pub border: [u8; 4],
    pub border_disabled: [u8; 4],
    pub focus_ring: [u8; 4],
    pub accent: [u8; 4],
    pub text: [u8; 4],
    pub text_muted: [u8; 4],
    pub text_disabled: [u8; 4],
}

pub(super) fn workbench_selection_control_palette() -> WorkbenchSelectionControlPalette {
    workbench_selection_control_palette_from_host(current_host_palette())
}

pub(super) fn workbench_selection_control_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchSelectionControlPalette {
    WorkbenchSelectionControlPalette {
        surface_disabled: palette.surface_disabled,
        mark_idle_fill: palette.popup,
        mark_idle_border: palette.separator_strong,
        checkbox_checked_fill: palette.surface_selected,
        radio_checked_fill: palette.surface_pressed,
        radio_checked_border: palette.separator_strong,
        toggle_checked_surface: palette.surface_selected,
        toggle_pressed_surface: palette.surface_pressed,
        toggle_hover_surface: palette.surface_hover,
        toggle_track: palette.track,
        toggle_checked_border: palette.separator_strong,
        border: palette.border,
        border_disabled: palette.border_disabled,
        focus_ring: palette.focus_ring,
        accent: palette.accent,
        text: palette.text,
        text_muted: palette.text_muted,
        text_disabled: palette.text_disabled,
    }
}
