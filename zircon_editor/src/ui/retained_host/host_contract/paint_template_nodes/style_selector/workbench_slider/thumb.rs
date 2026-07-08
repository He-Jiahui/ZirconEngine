use super::super::super::template_style_color::resolved_style_color;
use super::colors::declared_color;
use super::palette::WorkbenchSliderPalette;
use super::state::{is_unavailable_slider_state, slider_state_shows_thumb_halo};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(super) fn slider_thumb_color(
    node: &TemplatePaneNodeData,
    unavailable: bool,
    palette: &WorkbenchSliderPalette,
) -> [u8; 4] {
    if unavailable {
        palette.text_disabled
    } else {
        declared_color(node.icon_color).unwrap_or(palette.thumb)
    }
}

pub(super) fn slider_thumb_outline_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    _fill: [u8; 4],
    palette: &WorkbenchSliderPalette,
) -> [u8; 4] {
    if is_unavailable_slider_state(state) {
        palette.border_disabled
    } else {
        resolved_style_color(node.button_style.element.border_color.as_ref())
            .unwrap_or(palette.border)
    }
}

pub(super) fn slider_thumb_halo_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    palette: &WorkbenchSliderPalette,
) -> Option<[u8; 4]> {
    if is_unavailable_slider_state(state) {
        None
    } else {
        declared_color(node.state_layer_color)
            .or_else(|| slider_state_shows_thumb_halo(state).then_some(palette.thumb_halo))
    }
}
