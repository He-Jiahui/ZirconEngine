pub(in crate::ui::retained_host::host_contract) fn first_non_empty<'a>(
    values: &[&'a str],
) -> &'a str {
    values
        .iter()
        .copied()
        .find(|value| !value.trim().is_empty())
        .unwrap_or("")
}
