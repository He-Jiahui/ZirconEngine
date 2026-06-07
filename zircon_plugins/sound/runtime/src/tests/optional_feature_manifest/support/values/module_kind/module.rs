pub(super) fn module_kind_value_from_plugin_toml(
    value: &str,
) -> zircon_runtime::plugin::PluginModuleKind {
    super::module_kind_from_plugin_toml(value)
}
