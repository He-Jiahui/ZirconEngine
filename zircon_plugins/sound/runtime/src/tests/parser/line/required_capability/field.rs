pub(super) fn option_required_capability_value(line: &str) -> Option<&str> {
    line.strip_prefix("required_capability = \"")
        .and_then(|value| value.strip_suffix('"'))
}
