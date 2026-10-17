#[cfg(test)]
use super::super::super::paint_theme::PALETTE;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) const WELCOME_BACKGROUND: [u8; 4] =
    PALETTE.shell_background;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) const WELCOME_SURFACE: [u8; 4] = PALETTE.surface;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) const WELCOME_SURFACE_INSET: [u8; 4] =
    PALETTE.surface_inset;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) const WELCOME_TEXT: [u8; 4] = PALETTE.text;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) const WELCOME_MUTED_TEXT: [u8; 4] =
    PALETTE.text_muted;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) const WELCOME_WARNING: [u8; 4] = PALETTE.warning;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) const WELCOME_SUCCESS: [u8; 4] = PALETTE.success;
