use super::super::super::paint_theme::PALETTE;

pub(super) const WELCOME_BACKGROUND: [u8; 4] = [19, 23, 30, 255];
pub(super) const WELCOME_SURFACE: [u8; 4] = PALETTE.surface;
pub(super) const WELCOME_SURFACE_INSET: [u8; 4] = PALETTE.surface_inset;
pub(super) const WELCOME_SURFACE_HOVERED: [u8; 4] = PALETTE.surface_hover;
pub(super) const WELCOME_TEXT: [u8; 4] = PALETTE.text;
pub(super) const WELCOME_MUTED_TEXT: [u8; 4] = PALETTE.text_muted;
pub(super) const WELCOME_WARNING: [u8; 4] = PALETTE.warning;
pub(super) const WELCOME_SUCCESS: [u8; 4] = [88, 168, 112, 255];
pub(super) const WELCOME_ACTION_DISABLED_SURFACE: [u8; 4] = PALETTE.surface_disabled;
pub(super) const WELCOME_ACTION_DISABLED_TEXT: [u8; 4] = PALETTE.text_disabled;
pub(super) const WELCOME_PRIMARY_ACTION: [u8; 4] = PALETTE.accent;
