mod raw;

pub(in super::super) fn plugin_module_kind_from_plugin_toml(
    value: &str,
) -> zircon_runtime::plugin::PluginModuleKind {
    raw::plugin_module_kind_from_plugin_toml(value)
}
