pub(super) fn option_default_value(line: &str) -> Option<&str> {
    line.strip_prefix("default_value = \"")
        .and_then(|value| value.strip_suffix('"'))
}
