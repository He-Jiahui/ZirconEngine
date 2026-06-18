pub(in crate::tests::manifest::support::contributions) type StaticModule = (
    String,
    zircon_runtime::plugin::PluginModuleKind,
    String,
    Vec<zircon_runtime::builtin::RuntimeTargetMode>,
    Vec<String>,
);
