pub(super) fn dependency_primary_value(line: &str) -> Option<&str> {
    super::super::super::field::raw_value(line, "primary = ")
}
