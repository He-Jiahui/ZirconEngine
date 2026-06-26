use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SELECTION_LABEL_MUTED:
    [u8; 4] = PALETTE.text_disabled;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SELECTION_MARK_IDLE_FILL:
    [u8; 4] = PALETTE.popup;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SELECTION_MARK_IDLE_BORDER:
    [u8; 4] = PALETTE.separator_strong;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_CHECKBOX_CHECKED_FILL:
    [u8; 4] = PALETTE.surface_selected;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_RADIO_CHECKED_FILL: [u8;
    4] = PALETTE.surface_pressed;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_RADIO_CHECKED_BORDER:
    [u8; 4] = PALETTE.separator_strong;
