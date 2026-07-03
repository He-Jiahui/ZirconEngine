use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const PRIMARY_SURFACE: [u8;
    4] = PALETTE.surface_pressed;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const OUTLINED_SURFACE: [u8;
    4] = PALETTE.surface_pressed;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const OUTLINED_BORDER: [u8;
    4] = PALETTE.border;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const OUTLINED_TEXT: [u8; 4] =
    PALETTE.text;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DANGER_SURFACE: [u8;
    4] = PALETTE.surface_pressed;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DANGER_BORDER: [u8; 4] =
    PALETTE.border;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ADD_COMPONENT_TEXT:
    [u8; 4] = PALETTE.text_muted;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ADD_COMPONENT_GLYPH:
    [u8; 4] = PALETTE.text;
