use super::super::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract) const OVERLAY: [u8; 4] = [3, 5, 9, 168];
pub(in crate::ui::retained_host::host_contract) const DIALOG: [u8; 4] = PALETTE.surface;
pub(in crate::ui::retained_host::host_contract) const DIALOG_INSET: [u8; 4] = PALETTE.surface_inset;
pub(in crate::ui::retained_host::host_contract) const BUTTON: [u8; 4] = PALETTE.surface_hover;
pub(in crate::ui::retained_host::host_contract) const BUTTON_DISABLED: [u8; 4] = [31, 36, 45, 255];
pub(in crate::ui::retained_host::host_contract) const TEXT: [u8; 4] = PALETTE.text;
pub(in crate::ui::retained_host::host_contract) const MUTED: [u8; 4] = PALETTE.text_muted;
pub(in crate::ui::retained_host::host_contract) const WARNING: [u8; 4] = PALETTE.warning;
pub(in crate::ui::retained_host::host_contract) const ACCENT: [u8; 4] = PALETTE.focus_ring;
