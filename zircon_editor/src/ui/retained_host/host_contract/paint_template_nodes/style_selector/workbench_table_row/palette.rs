use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_ROW_BG: [u8; 4] =
    PALETTE.surface_inset;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_HEADER_BG: [u8; 4] =
    WORKBENCH_TABLE_ROW_BG;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_TAIL_BG: [u8; 4] =
    WORKBENCH_TABLE_ROW_BG;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_SELECTED_BG: [u8;
    4] = PALETTE.surface_pressed;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_HOVER_BG: [u8; 4] =
    PALETTE.surface_pressed;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_SEPARATOR: [u8; 4] =
    PALETTE.separator_soft;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_ACTION_MUTED: [u8; 4] = PALETTE.text_disabled;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_HEADER_TEXT: [u8;
    4] = PALETTE.text_muted;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_TAIL_VALUE_TEXT: [u8; 4] = PALETTE.text_muted;
