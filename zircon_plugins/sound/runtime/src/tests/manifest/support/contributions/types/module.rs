pub(in crate::tests::manifest::support::contributions) type StaticModule = (
    String,
    zircon_runtime::plugin::PluginModuleKind,
    String,
    Vec<zircon_runtime::core::framework::platform::RuntimeTargetMode>,
    Vec<String>,
);
