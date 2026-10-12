pub(super) fn event_catalog_namespace_value(line: &str) -> Option<&str> {
    line.strip_prefix("namespace = \"")
        .and_then(|value| value.strip_suffix('"'))
}
