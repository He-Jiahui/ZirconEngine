use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_STATUS_FLAT_TRANSPARENT:
    [u8; 4] = [0, 0, 0, 0];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_STATUS_ICON_COLOR: [u8;
    4] = PALETTE.text_muted;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_STATUS_ICON_MUTED: [u8;
    4] = PALETTE.text_disabled;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_STATUS_NO_ERRORS_FILL:
    [u8; 4] = PALETTE.success;
