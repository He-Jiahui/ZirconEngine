pub(super) fn module_crate_name_value(line: &str) -> Option<&str> {
    line.strip_prefix("crate_name = \"")
        .and_then(|value| value.strip_suffix('"'))
}
