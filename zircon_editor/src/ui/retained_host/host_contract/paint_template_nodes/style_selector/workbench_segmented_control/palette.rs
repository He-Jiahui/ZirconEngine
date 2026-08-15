#[cfg(test)]
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchSegmentedControlPalette
{
    pub idle_background: [u8; 4],
    pub hot_background: [u8; 4],
    pub pressed_background: [u8; 4],
    pub disabled_background: [u8; 4],
    pub border: [u8; 4],
    pub active_border: [u8; 4],
    pub focus_border: [u8; 4],
    pub disabled_border: [u8; 4],
    pub selected_background: [u8; 4],
    pub selected_border: [u8; 4],
    pub selected_underline: [u8; 4],
    pub selected_text: [u8; 4],
    pub idle_text: [u8; 4],
    pub disabled_text: [u8; 4],
    pub group_label: [u8; 4],
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SEGMENT_IDLE_BACKGROUND:
    [u8; 4] = PALETTE.surface;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SEGMENT_SELECTED_BACKGROUND:
    [u8; 4] = PALETTE.surface_pressed;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SEGMENT_GROUP_LABEL_COLOR: [u8; 4] = PALETTE.text_muted;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_segmented_control_palette(
) -> WorkbenchSegmentedControlPalette {
    workbench_segmented_control_palette_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_segmented_control_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchSegmentedControlPalette {
    WorkbenchSegmentedControlPalette {
        idle_background: palette.surface,
        hot_background: palette.surface_hover,
        pressed_background: palette.surface_pressed,
        disabled_background: palette.surface_disabled,
        border: palette.border,
        active_border: palette.accent,
        focus_border: palette.focus_ring,
        disabled_border: palette.border_disabled,
        selected_background: palette.surface_pressed,
        selected_border: palette.accent,
        selected_underline: palette.accent,
        selected_text: palette.text,
        idle_text: palette.text_muted,
        disabled_text: palette.text_disabled,
        group_label: palette.text_muted,
    }
}
