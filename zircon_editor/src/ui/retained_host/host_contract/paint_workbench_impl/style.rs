use super::super::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract) const TOP_BAR: [u8; 4] = PALETTE.popup;
pub(in crate::ui::retained_host::host_contract) const CENTER_BAND: [u8; 4] = [23, 27, 34, 255];
pub(in crate::ui::retained_host::host_contract) const SIDE_PANEL: [u8; 4] = PALETTE.surface_inset;
pub(in crate::ui::retained_host::host_contract) const DOCUMENT_PANEL: [u8; 4] = [13, 16, 22, 255];
pub(in crate::ui::retained_host::host_contract) const VIEWPORT_PANEL: [u8; 4] = [7, 10, 15, 255];
pub(in crate::ui::retained_host::host_contract) const TOOLBAR: [u8; 4] = PALETTE.surface;
pub(in crate::ui::retained_host::host_contract) const STATUS_BAR: [u8; 4] = PALETTE.surface_hover;
pub(in crate::ui::retained_host::host_contract) const FLOATING_SHADOW: [u8; 4] = [4, 6, 10, 180];
pub(in crate::ui::retained_host::host_contract) const FLOATING_PANEL: [u8; 4] = PALETTE.surface;
pub(in crate::ui::retained_host::host_contract) const PANE_EMPTY: [u8; 4] = PALETTE.surface_inset;
pub(in crate::ui::retained_host::host_contract) const SEPARATOR: [u8; 4] = PALETTE.border;
pub(in crate::ui::retained_host::host_contract) const ACCENT: [u8; 4] = PALETTE.focus_ring;
pub(in crate::ui::retained_host::host_contract) const MUTED_TEXT: [u8; 4] = PALETTE.text_muted;
