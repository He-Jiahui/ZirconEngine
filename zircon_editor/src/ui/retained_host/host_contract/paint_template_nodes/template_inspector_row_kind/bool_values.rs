pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn bool_display_value(
    value: &str,
) -> &'static str {
    if bool_value(value) {
        "On"
    } else {
        "Off"
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn bool_value(
    value: &str,
) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "on" | "yes" | "check" | "checked"
    )
}
