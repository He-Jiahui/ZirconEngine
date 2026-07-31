use super::super::super::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract) const WELCOME_BACKGROUND: [u8; 4] =
    [19, 23, 30, 255];
pub(in crate::ui::retained_host::host_contract) const WELCOME_SURFACE: [u8; 4] = PALETTE.surface;
pub(in crate::ui::retained_host::host_contract) const WELCOME_SURFACE_INSET: [u8; 4] =
    PALETTE.surface_inset;
pub(in crate::ui::retained_host::host_contract) const WELCOME_TEXT: [u8; 4] = PALETTE.text;
pub(in crate::ui::retained_host::host_contract) const WELCOME_MUTED_TEXT: [u8; 4] =
    PALETTE.text_muted;
pub(in crate::ui::retained_host::host_contract) const WELCOME_WARNING: [u8; 4] = PALETTE.warning;
pub(in crate::ui::retained_host::host_contract) const WELCOME_SUCCESS: [u8; 4] =
    [88, 168, 112, 255];
