use super::*;

#[test]
fn bool_parser_keeps_case_and_whitespace_semantics_without_lowercase_allocation() {
    for value in [" true ", "TRUE", "On", "yEs", "CHECK", "Checked", "1"] {
        assert!(bool_value(value), "{value}");
    }
    for value in ["", "0", "off", "unchecked", "truthy"] {
        assert!(!bool_value(value), "{value}");
    }

    let production = include_str!("../bool_values.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source");
    assert!(!production.contains("to_ascii_lowercase"));
    assert!(!production.contains(".iter().any"));
}
