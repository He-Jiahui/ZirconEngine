//! 滑块标题和值显示的文字来源；值文本优先，缺省时将已归一化进度格式化为两位小数。

use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_label(
    node: &TemplatePaneNodeData,
) -> Option<String> {
    let label = node.label_text.trim();
    (!label.is_empty()).then(|| label.to_owned())
}

/// 节点显式显示文本优先；缺省显示归一化进度，调用方不应再把百分数乘百。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_value_label(
    node: &TemplatePaneNodeData,
    percent: f32,
) -> String {
    let value = node.value_text.trim();
    if value.is_empty() {
        normalized_percent_label(percent)
    } else {
        value.to_owned()
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_range_min_label(
    percent: f32,
) -> String {
    normalized_percent_label(percent)
}

// 与旧 format 两位小数语义一致，保留 NaN 和负零的特殊格式；正常有限值走短字符串构造。
fn normalized_percent_label(percent: f32) -> String {
    let percent = percent.clamp(0.0, 1.0);
    if !percent.is_finite() || (percent == 0.0 && percent.is_sign_negative()) {
        return format!("{percent:.2}");
    }

    let hundredths = (f64::from(percent) * 100.0).round_ties_even() as u8;
    let mut label = String::with_capacity(4);
    label.push(char::from(b'0' + hundredths / 100));
    label.push('.');
    label.push(char::from(b'0' + (hundredths / 10) % 10));
    label.push(char::from(b'0' + hundredths % 10));
    label
}

#[cfg(test)]
#[path = "tests/labels_optimization_batch_ez_tests.rs"]
mod optimization_batch_ez_tests;
