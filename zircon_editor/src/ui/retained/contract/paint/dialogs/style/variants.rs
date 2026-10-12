//! 读取上游投影的完整空白分隔token，避免把描述或token子串误判成状态。
//! 五组语义字段均可提供同一token；匹配仅ASCII大小写无关，不承担本地化文本解析。

use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn variant_contains_any(
    node: &TemplatePaneNodeData,
    expected: &[&str],
) -> bool {
    [
        node.component_variant.as_str(),
        node.surface_variant.as_str(),
        node.validation_level.as_str(),
        node.text_tone.as_str(),
        node.button_variant.as_str(),
    ]
    .iter()
    .flat_map(|value| value.split_whitespace())
    .any(|part| {
        expected
            .iter()
            .any(|expected| part.len() == expected.len() && part.eq_ignore_ascii_case(expected))
    })
}

#[cfg(test)]
#[path = "tests/variants.rs"]
mod tests;
