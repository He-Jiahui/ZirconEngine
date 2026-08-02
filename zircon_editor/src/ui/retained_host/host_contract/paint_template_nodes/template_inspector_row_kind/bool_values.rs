pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn bool_display_value(
    value: &str,
) -> &'static str {
    if bool_value(value) { "On" } else { "Off" }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn bool_value(
    value: &str,
) -> bool {
    let value = value.trim();
    value == "1"
        || ["true", "on", "yes", "check", "checked"]
            .iter()
            .any(|candidate| value.eq_ignore_ascii_case(candidate))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool_parser_keeps_case_and_whitespace_semantics_without_lowercase_allocation() {
        for value in [" true ", "TRUE", "On", "yEs", "CHECK", "Checked", "1"] {
            assert!(bool_value(value), "{value}");
        }
        for value in ["", "0", "off", "unchecked", "truthy"] {
            assert!(!bool_value(value), "{value}");
        }

        let production = include_str!("bool_values.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        assert!(!production.contains("to_ascii_lowercase"));
    }
}
