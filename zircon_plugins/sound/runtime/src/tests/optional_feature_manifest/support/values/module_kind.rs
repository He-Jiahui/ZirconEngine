mod module;
mod raw;

fn module_kind_from_plugin_toml(value: &str) -> zircon_runtime::plugin::PluginModuleKind {
    raw::module_kind_from_plugin_toml(value)
}

pub(in super::super) fn module_kind_value_from_plugin_toml(
    value: &str,
) -> zircon_runtime::plugin::PluginModuleKind {
    module::module_kind_value_from_plugin_toml(value)
}
