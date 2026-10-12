pub(super) fn event_catalog_version_value(line: &str) -> Option<&str> {
    line.strip_prefix("version = ")
}
