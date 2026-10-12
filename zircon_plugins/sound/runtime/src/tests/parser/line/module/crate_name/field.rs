pub(super) fn module_crate_name_value(line: &str) -> Option<&str> {
    super::super::super::super::field::quoted_value(line, "crate_name = \"")
}
