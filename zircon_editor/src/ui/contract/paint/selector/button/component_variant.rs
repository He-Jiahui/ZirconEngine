//! 从组件变体的独立词元识别紧凑图文按钮，供绘制几何选择布局；它不决定按钮命令。

use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_compact_icon_text_workbench_button(
    node: &TemplatePaneNodeData,
) -> bool {
    node.component_variant
        .split_ascii_whitespace()
        .any(|token| {
            token.len() == "compact_icon_text".len()
                && token.eq_ignore_ascii_case("compact_icon_text")
        })
}

#[cfg(test)]
#[path = "tests/component_variant.rs"]
mod tests;
