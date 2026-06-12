pub(in super::super::super) fn module_capability_list_from_plugin_toml(value: &str) -> Vec<String> {
    super::list::string_list_from_plugin_toml(value)
}
