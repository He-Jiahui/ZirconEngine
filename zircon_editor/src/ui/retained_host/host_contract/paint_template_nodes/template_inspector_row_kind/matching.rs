//! 多候选标签匹配保持 ASCII 大小写等价且先排除长度不合的字符串；仅用于现有英文显示标签识别。

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn matches_ignore_ascii_case(
    value: &str,
    candidates: &[&str],
) -> bool {
    candidates
        .iter()
        .any(|candidate| value.len() == candidate.len() && value.eq_ignore_ascii_case(candidate))
}

#[cfg(test)]
#[path = "tests/matching.rs"]
mod tests;
