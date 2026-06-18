use super::super::super::super::StaticModule;

pub(super) fn module_contribution_manifest(
    name: String,
    kind: zircon_runtime::plugin::PluginModuleKind,
    crate_name: String,
    target_modes: Vec<zircon_runtime::builtin::RuntimeTargetMode>,
    capabilities: Vec<String>,
) -> StaticModule {
    (name, kind, crate_name, target_modes, capabilities)
}
