pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn entry_name_for_source_key(
    source_key: &str,
) -> Option<&str> {
    source_key
        .strip_prefix("template-image:")
        .or_else(|| source_key.strip_prefix("template-icon:"))
        .or_else(|| source_key.strip_prefix("image:"))
        .or_else(|| source_key.strip_prefix("icon:"))
}
