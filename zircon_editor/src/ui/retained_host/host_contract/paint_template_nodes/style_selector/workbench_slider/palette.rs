#[cfg(test)]
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SLIDER_TRACK: [u8; 4] =
    PALETTE.surface_inset;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SLIDER_TRACK_DISABLED:
    [u8; 4] = PALETTE.border_disabled;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SLIDER_FILL: [u8; 4] =
    PALETTE.separator_strong;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SLIDER_THUMB: [u8; 4] =
    PALETTE.text;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WorkbenchSliderPalette {
    pub track: [u8; 4],
    pub track_disabled: [u8; 4],
    pub fill: [u8; 4],
    pub label_text: [u8; 4],
    pub value_text: [u8; 4],
    pub thumb: [u8; 4],
    pub thumb_halo: [u8; 4],
    pub tick: [u8; 4],
    pub value_surface: [u8; 4],
    pub value_border: [u8; 4],
    pub surface_disabled: [u8; 4],
    pub border: [u8; 4],
    pub border_disabled: [u8; 4],
    pub text_disabled: [u8; 4],
    pub warning: [u8; 4],
    pub error: [u8; 4],
}

pub(super) fn workbench_slider_palette() -> WorkbenchSliderPalette {
    workbench_slider_palette_from_host(current_host_palette())
}

pub(super) fn workbench_slider_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchSliderPalette {
    WorkbenchSliderPalette {
        track: palette.surface_inset,
        track_disabled: palette.border_disabled,
        fill: palette.separator_strong,
        label_text: palette.text_muted,
        value_text: palette.text_muted,
        thumb: palette.text,
        thumb_halo: with_alpha(palette.focus_ring, 26),
        tick: palette.separator_soft,
        value_surface: palette.popup,
        value_border: palette.border,
        surface_disabled: palette.surface_disabled,
        border: palette.border,
        border_disabled: palette.border_disabled,
        text_disabled: palette.text_disabled,
        warning: palette.warning,
        error: palette.error,
    }
}

const fn with_alpha(mut color: [u8; 4], alpha: u8) -> [u8; 4] {
    color[3] = alpha;
    color
}
