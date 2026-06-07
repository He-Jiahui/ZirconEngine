pub(super) fn module_kind_from_plugin_toml(
    value: &str,
) -> zircon_runtime::plugin::PluginModuleKind {
    match value {
        "runtime" => zircon_runtime::plugin::PluginModuleKind::Runtime,
        "editor" => zircon_runtime::plugin::PluginModuleKind::Editor,
        "native" => zircon_runtime::plugin::PluginModuleKind::Native,
        "vm" => zircon_runtime::plugin::PluginModuleKind::Vm,
        _ => panic!("unknown sound module kind {value}"),
    }
}
