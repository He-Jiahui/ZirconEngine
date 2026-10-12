pub(super) fn dependency_capability_value(line: &str) -> Option<&str> {
    super::super::super::super::field::quoted_value(line, "capability = \"")
}
