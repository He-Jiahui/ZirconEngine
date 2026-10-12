pub(super) fn option_value_type_value(line: &str) -> Option<&str> {
    line.strip_prefix("value_type = \"")
        .and_then(|value| value.strip_suffix('"'))
}
