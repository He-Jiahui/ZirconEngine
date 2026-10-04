use std::fmt::Write;

use zircon_runtime_interface::ui::template::UiTemplateNode;

/// 为构树遍历生成父路径下的可诊断段；字符替换后仍以兄弟序号区分同名节点。
/// 路径随兄弟顺序改变，不能替代跨重建使用的控件身份。
pub(super) fn child_segment(node: &UiTemplateNode, index: usize) -> String {
    let raw = node
        .control_id
        .as_deref()
        .or(node.component.as_deref())
        .unwrap_or("node");
    let mut sanitized = String::with_capacity(raw.len() + 1 + decimal_digit_count(index));
    sanitized.extend(raw.chars().map(|ch| match ch {
        '/' | '\\' | ' ' | ':' | '#' => '_',
        _ => ch,
    }));
    sanitized.push('_');
    write!(&mut sanitized, "{index}").expect("writing a usize to String cannot fail");
    sanitized
}

fn decimal_digit_count(mut value: usize) -> usize {
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}

#[cfg(test)]
#[path = "child_segment/tests/direct_write_tests.rs"]
mod direct_write_tests;
