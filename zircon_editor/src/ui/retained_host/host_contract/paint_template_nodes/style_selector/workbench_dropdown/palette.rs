use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DROPDOWN_SURFACE: [u8; 4] =
    PALETTE.surface_inset;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DROPDOWN_HOVER_SURFACE: [u8; 4] = PALETTE.popup;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DROPDOWN_OPEN_SURFACE: [u8; 4] = PALETTE.surface_pressed;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DROPDOWN_DISABLED_SURFACE: [u8; 4] = PALETTE.surface_disabled;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DROPDOWN_BORDER: [u8; 4] =
    PALETTE.border;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DROPDOWN_FOCUS_BORDER:
    [u8; 4] = PALETTE.focus_ring;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DROPDOWN_TEXT: [u8; 4] = PALETTE.text;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DROPDOWN_PLACEHOLDER:
    [u8; 4] = PALETTE.text_disabled;
