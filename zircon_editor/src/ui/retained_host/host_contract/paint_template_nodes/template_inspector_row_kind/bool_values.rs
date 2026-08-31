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
    let value = value.trim();
    match value.len() {
        1 => value == "1",
        2 => value.eq_ignore_ascii_case("on"),
        3 => value.eq_ignore_ascii_case("yes"),
        4 => value.eq_ignore_ascii_case("true"),
        5 => value.eq_ignore_ascii_case("check"),
        7 => value.eq_ignore_ascii_case("checked"),
        _ => false,
    }
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
        assert!(!production.contains(".iter().any"));
    }
}
