pub(super) fn dependency_plugin_id_value(line: &str) -> Option<&str> {
    super::super::super::super::field::quoted_value(line, "plugin_id = \"")
}
