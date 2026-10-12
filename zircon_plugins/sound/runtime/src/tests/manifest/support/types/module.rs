// runtime 模块对照保留种类、crate、目标模式和能力，避免只核对模块名称。
pub(in crate::tests::manifest::support::contributions) type StaticModule = (
    String,
    zircon_runtime::plugin::PluginModuleKind,
    String,
    Vec<zircon_runtime::core::framework::platform::RuntimeTargetMode>,
    Vec<String>,
);
