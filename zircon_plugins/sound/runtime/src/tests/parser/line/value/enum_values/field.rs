pub(super) fn option_enum_values_value(line: &str) -> Option<&str> {
    line.strip_prefix("enum_values = [")
        .and_then(|value| value.strip_suffix(']'))
}
