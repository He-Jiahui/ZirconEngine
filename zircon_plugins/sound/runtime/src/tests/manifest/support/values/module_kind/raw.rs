// 将静态模块种类转换成运行时包清单类型，以便过滤并比较 runtime 模块。
pub(super) fn plugin_module_kind_from_plugin_toml(
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
