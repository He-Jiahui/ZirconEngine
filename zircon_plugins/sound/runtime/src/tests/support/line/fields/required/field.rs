pub(super) fn dependency_required_value(line: &str) -> Option<&str> {
    line.strip_prefix("required = ")
}
