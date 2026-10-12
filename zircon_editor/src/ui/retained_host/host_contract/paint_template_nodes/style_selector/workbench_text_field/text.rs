//! 正文按占位标志选择文字角色，步进器与分隔线保持独立；禁用或加载统一使用不可用配方。

use super::colors::{authored_field_color, declared_style_color};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

use super::palette::workbench_text_field_palette;
use super::state::is_unavailable_text_field_state;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_field_text(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    label_is_placeholder: bool,
) -> [u8; 4] {
    let palette = workbench_text_field_palette();
    if is_unavailable_text_field_state(state) {
        authored_field_color(node, &["disabled_foreground_color"]).unwrap_or(palette.disabled_text)
    } else {
        authored_field_color(
            node,
            if label_is_placeholder {
                &["placeholder_color"]
            } else {
                &["foreground_color", "text_color"]
            },
        )
        .or_else(|| {
            (!label_is_placeholder)
                .then(|| declared_style_color(node.button_style.element.foreground_color.as_ref()))
                .flatten()
        })
        .unwrap_or(if label_is_placeholder {
            palette.placeholder
        } else {
            palette.text
        })
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_field_stepper(
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let palette = workbench_text_field_palette();
    if is_unavailable_text_field_state(state) {
        palette.disabled_text
    } else {
        palette.placeholder
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_field_stepper_divider(
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let palette = workbench_text_field_palette();
    if is_unavailable_text_field_state(state) {
        palette.disabled_border
    } else {
        palette.stepper_divider
    }
}
