//! 为滑块标签与数值框提供文字角色；节点标签色只覆盖标签，数值文字维持主题角色，不可用状态优先。

use super::colors::declared_color;
use super::palette::WorkbenchSliderPalette;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(super) fn slider_label_color(
    node: &TemplatePaneNodeData,
    unavailable: bool,
    palette: &WorkbenchSliderPalette,
) -> [u8; 4] {
    if unavailable {
        palette.text_disabled
    } else {
        declared_color(node.label_color).unwrap_or(palette.label_text)
    }
}

pub(super) fn slider_value_text(unavailable: bool, palette: &WorkbenchSliderPalette) -> [u8; 4] {
    if unavailable {
        palette.text_disabled
    } else {
        palette.value_text
    }
}
