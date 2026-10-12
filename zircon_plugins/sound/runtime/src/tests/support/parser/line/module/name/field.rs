pub(super) fn module_name_value(line: &str) -> Option<&str> {
    super::super::super::super::field::quoted_value(line, "name = \"")
}
