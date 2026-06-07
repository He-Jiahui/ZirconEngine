pub(super) fn module_kind_value(line: &str) -> Option<&str> {
    super::super::super::field::quoted_value(line, "kind = \"")
}
