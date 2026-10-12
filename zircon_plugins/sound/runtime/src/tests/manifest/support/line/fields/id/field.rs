pub(super) fn dependency_id_value(line: &str) -> Option<&str> {
    line.strip_prefix("id = \"")
        .and_then(|value| value.strip_suffix('"'))
}
