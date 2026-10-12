pub(super) fn module_kind_value(line: &str) -> Option<&str> {
    line.strip_prefix("kind = \"")
        .and_then(|value| value.strip_suffix('"'))
}
