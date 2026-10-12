pub(super) fn module_name_value(line: &str) -> Option<&str> {
    line.strip_prefix("name = \"")
        .and_then(|value| value.strip_suffix('"'))
}
