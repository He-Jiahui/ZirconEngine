use super::super::super::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract) const WELCOME_BACKGROUND: [u8; 4] =
    [19, 23, 30, 255];
pub(in crate::ui::retained_host::host_contract) const WELCOME_SURFACE: [u8; 4] = PALETTE.surface;
pub(in crate::ui::retained_host::host_contract) const WELCOME_SURFACE_INSET: [u8; 4] =
    PALETTE.surface_inset;
pub(in crate::ui::retained_host::host_contract) const WELCOME_SURFACE_HOVERED: [u8; 4] =
    PALETTE.surface_hover;
pub(in crate::ui::retained_host::host_contract) const WELCOME_TEXT: [u8; 4] = PALETTE.text;
pub(in crate::ui::retained_host::host_contract) const WELCOME_MUTED_TEXT: [u8; 4] =
    PALETTE.text_muted;
pub(in crate::ui::retained_host::host_contract) const WELCOME_WARNING: [u8; 4] = PALETTE.warning;
pub(in crate::ui::retained_host::host_contract) const WELCOME_SUCCESS: [u8; 4] =
    [88, 168, 112, 255];
pub(in crate::ui::retained_host::host_contract) const WELCOME_ACTION_DISABLED_SURFACE: [u8; 4] =
    PALETTE.surface_disabled;
pub(in crate::ui::retained_host::host_contract) const WELCOME_ACTION_DISABLED_TEXT: [u8; 4] =
    PALETTE.text_disabled;
pub(in crate::ui::retained_host::host_contract) const WELCOME_PRIMARY_ACTION: [u8; 4] =
    PALETTE.accent;
