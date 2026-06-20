pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn matches_ignore_ascii_case(
    value: &str,
    candidates: &[&str],
) -> bool {
    candidates
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}
