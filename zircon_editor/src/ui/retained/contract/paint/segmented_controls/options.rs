//! 忽略空选项并为选中值建立文本匹配规则；计数与绘制迭代器必须相同以保持每段宽度。

use super::super::super::data::TemplatePaneNodeData;

/// 计数和正文绘制必须共享此非空选项迭代器，否则段宽与标签索引不一致。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segmented_options(
    node: &TemplatePaneNodeData,
) -> impl Iterator<Item = &str> + '_ {
    node.options
        .iter()
        .map(String::as_str)
        .filter(|option| !option.trim().is_empty())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segmented_option_count(
    node: &TemplatePaneNodeData,
) -> usize {
    segmented_options(node).count()
}

/// 从值文本、选项文本、普通文本依序取首个非空候选；调用方用它按文字比较各选项。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_segment_value(
    node: &TemplatePaneNodeData,
) -> Option<&str> {
    [
        node.value_text.as_str(),
        node.options_text.as_str(),
        node.text.as_str(),
    ]
    .into_iter()
    .find(|value| !value.trim().is_empty())
    .map(str::trim)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn option_is_selected(
    option: &str,
    selected: Option<&str>,
) -> bool {
    selected.is_some_and(|value| option.trim().eq_ignore_ascii_case(value))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_label(
    option: &str,
) -> String {
    let trimmed = option.trim();
    let mut chars = trimmed.chars();
    match chars.next() {
        Some(first) => {
            let mut label = first.to_ascii_uppercase().to_string();
            label.push_str(chars.as_str());
            label
        }
        None => String::new(),
    }
}
