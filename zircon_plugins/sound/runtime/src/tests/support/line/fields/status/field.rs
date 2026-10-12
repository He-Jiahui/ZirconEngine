pub(super) fn capability_status_status_value(line: &str) -> Option<&str> {
    line.strip_prefix("status = \"")
        .and_then(|value| value.strip_suffix('"'))
}
