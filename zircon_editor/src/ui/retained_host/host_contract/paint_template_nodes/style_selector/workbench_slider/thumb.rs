//! 拇指填色、边线和 halo 为独立通道；已声明的 halo 在可用状态可持续显示，否则按可见焦点或热态回退。
//! 此处只选择颜色，halo 尺寸与绘制位置由模板绘制和几何端负责。

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
