pub(super) fn option_key_value(line: &str) -> Option<&str> {
    line.strip_prefix("key = \"")
        .and_then(|value| value.strip_suffix('"'))
}
