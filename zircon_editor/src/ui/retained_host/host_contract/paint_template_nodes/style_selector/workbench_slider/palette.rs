use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SLIDER_TRACK: [u8; 4] =
    PALETTE.surface_inset;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SLIDER_TRACK_DISABLED:
    [u8; 4] = PALETTE.border_disabled;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SLIDER_FILL: [u8; 4] =
    PALETTE.separator_strong;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SLIDER_TEXT: [u8; 4] =
    PALETTE.text_muted;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SLIDER_THUMB: [u8; 4] =
    PALETTE.text;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SLIDER_HALO: [u8; 4] =
    [216, 227, 231, 26];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SLIDER_TICK: [u8; 4] =
    PALETTE.separator_soft;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SLIDER_VALUE_SURFACE:
    [u8; 4] = PALETTE.popup;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SLIDER_VALUE_BORDER:
    [u8; 4] = PALETTE.border;
