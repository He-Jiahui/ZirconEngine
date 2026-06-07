use super::super::super::super::super::values::module_kind_value_from_plugin_toml;

pub(super) fn module_kind_from_plugin_toml(
    value: &str,
) -> zircon_runtime::plugin::PluginModuleKind {
    module_kind_value_from_plugin_toml(value)
}
