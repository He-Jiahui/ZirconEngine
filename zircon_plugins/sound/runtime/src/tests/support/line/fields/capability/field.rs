pub(super) fn dependency_capability_value(line: &str) -> Option<&str> {
    line.strip_prefix("capability = \"")
        .and_then(|value| value.strip_suffix('"'))
}
