use super::super::super::template_style_color::resolved_style_color;
use super::colors::declared_color;
use super::palette::{WORKBENCH_SLIDER_HALO, WORKBENCH_SLIDER_THUMB};
use super::state::{is_unavailable_slider_state, is_workbench_slider_state_hot};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_thumb_color(
    node: &TemplatePaneNodeData,
    unavailable: bool,
) -> [u8; 4] {
    if unavailable {
        PALETTE.text_disabled
    } else {
        declared_color(node.icon_color).unwrap_or(WORKBENCH_SLIDER_THUMB)
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_thumb_outline_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    _fill: [u8; 4],
) -> [u8; 4] {
    if is_unavailable_slider_state(state) {
        PALETTE.border_disabled
    } else {
        resolved_style_color(node.button_style.element.border_color.as_ref())
            .unwrap_or(PALETTE.border)
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_thumb_halo_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> Option<[u8; 4]> {
    if is_unavailable_slider_state(state) {
        None
    } else {
        declared_color(node.state_layer_color)
            .or_else(|| is_workbench_slider_state_hot(state).then_some(WORKBENCH_SLIDER_HALO))
    }
}
