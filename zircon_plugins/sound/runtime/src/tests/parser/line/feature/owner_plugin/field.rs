pub(super) fn feature_owner_plugin_value(line: &str) -> Option<&str> {
    super::super::super::super::field::quoted_value(line, "owner_plugin_id = \"")
}
