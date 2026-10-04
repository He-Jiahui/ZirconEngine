//! 数值显示框保持独立于拇指焦点反馈的边线；按压时普通数值框使用填充色，范围端值框仍保持主题边线。

use super::palette::WorkbenchSliderPalette;
use super::state::is_unavailable_slider_state;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(super) fn slider_value_surface(unavailable: bool, palette: &WorkbenchSliderPalette) -> [u8; 4] {
    if unavailable {
        palette.surface_disabled
    } else {
        palette.value_surface
    }
}

pub(super) fn slider_value_border(
    state: UiPainterResolvedState,
    fill: [u8; 4],
    palette: &WorkbenchSliderPalette,
) -> [u8; 4] {
    if is_unavailable_slider_state(state) {
        return palette.border_disabled;
    }
    if state == UiPainterResolvedState::Pressed {
        fill
    } else {
        palette.value_border
    }
}

pub(super) fn slider_range_value_border(
    state: UiPainterResolvedState,
    palette: &WorkbenchSliderPalette,
) -> [u8; 4] {
    if is_unavailable_slider_state(state) {
        palette.border_disabled
    } else {
        palette.value_border
    }
}
