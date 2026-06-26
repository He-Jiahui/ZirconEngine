use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const PRIMARY_SURFACE: [u8;
    4] = [31, 48, 53, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const PRIMARY_SURFACE_HOVER: [u8; 4] = [38, 61, 67, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const PRIMARY_SURFACE_PRESSED: [u8; 4] = PALETTE.surface_selected;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const PRIMARY_TEXT: [u8; 4] =
    PALETTE.text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const OUTLINED_SURFACE: [u8;
    4] = PALETTE.surface_pressed;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const OUTLINED_SURFACE_HOVER: [u8; 4] = PALETTE.surface_hover;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const OUTLINED_SURFACE_PRESSED: [u8; 4] = PALETTE.surface;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const OUTLINED_BORDER: [u8;
    4] = PALETTE.border;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const OUTLINED_TEXT: [u8; 4] =
    PALETTE.text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TERTIARY_TEXT: [u8; 4] =
    PALETTE.text_muted;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TERTIARY_SURFACE: [u8;
    4] = [0, 0, 0, 0];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TERTIARY_BORDER: [u8;
    4] = [0, 0, 0, 0];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DANGER_SURFACE: [u8;
    4] = PALETTE.error_container;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DANGER_SURFACE_HOVER:
    [u8; 4] = PALETTE.error_container;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DANGER_SURFACE_PRESSED: [u8; 4] = PALETTE.surface_pressed;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DANGER_BORDER: [u8; 4] =
    PALETTE.error;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DANGER_TEXT: [u8; 4] =
    PALETTE.error;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ADD_COMPONENT_TEXT:
    [u8; 4] = PALETTE.text_muted;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ADD_COMPONENT_GLYPH:
    [u8; 4] = PALETTE.text;
