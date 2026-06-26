use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ICON_NORMAL: [u8; 4] =
    PALETTE.text;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ICON_MUTED: [u8; 4] =
    PALETTE.text_muted;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ICON_PANEL_SURFACE:
    [u8; 4] = PALETTE.surface_pressed;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ICON_PANEL_BORDER:
    [u8; 4] = PALETTE.border;
