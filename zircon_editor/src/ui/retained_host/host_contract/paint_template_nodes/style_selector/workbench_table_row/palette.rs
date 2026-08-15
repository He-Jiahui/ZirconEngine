#[cfg(test)]
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_ROW_BG: [u8; 4] =
    PALETTE.surface_inset;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_HEADER_BG: [u8; 4] =
    WORKBENCH_TABLE_ROW_BG;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_TAIL_BG: [u8; 4] =
    WORKBENCH_TABLE_ROW_BG;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_SELECTED_BG: [u8;
    4] = PALETTE.surface_pressed;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_HOVER_BG: [u8; 4] =
    PALETTE.surface_hover;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_SEPARATOR: [u8; 4] =
    PALETTE.separator_soft;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_HEADER_TEXT: [u8;
    4] = PALETTE.text_muted;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WorkbenchTableRowPalette {
    pub row_bg: [u8; 4],
    pub header_bg: [u8; 4],
    pub tail_bg: [u8; 4],
    pub selected_bg: [u8; 4],
    pub hover_bg: [u8; 4],
    pub separator: [u8; 4],
    pub action_muted: [u8; 4],
    pub header_text: [u8; 4],
    pub tail_value_text: [u8; 4],
    pub surface_disabled: [u8; 4],
    pub surface_pressed: [u8; 4],
    pub focus_ring: [u8; 4],
    pub text: [u8; 4],
    pub text_muted: [u8; 4],
    pub text_disabled: [u8; 4],
}

pub(super) fn workbench_table_row_palette() -> WorkbenchTableRowPalette {
    workbench_table_row_palette_from_host(current_host_palette())
}

pub(super) fn workbench_table_row_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchTableRowPalette {
    WorkbenchTableRowPalette {
        row_bg: palette.surface_inset,
        header_bg: palette.surface_inset,
        tail_bg: palette.surface_inset,
        selected_bg: palette.surface_pressed,
        hover_bg: palette.surface_hover,
        separator: palette.separator_soft,
        action_muted: palette.text_disabled,
        header_text: palette.text_muted,
        tail_value_text: palette.text_muted,
        surface_disabled: palette.surface_disabled,
        surface_pressed: palette.surface_pressed,
        focus_ring: palette.focus_ring,
        text: palette.text,
        text_muted: palette.text_muted,
        text_disabled: palette.text_disabled,
    }
}
