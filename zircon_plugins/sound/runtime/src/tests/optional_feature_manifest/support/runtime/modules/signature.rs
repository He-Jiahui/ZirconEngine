use super::super::super::types::OptionalFeatureModuleSignature;

pub(super) fn module_signature(
    module: &zircon_runtime::plugin::PluginModuleManifest,
) -> OptionalFeatureModuleSignature {
    (
        module.name.clone(),
        module.kind,
        module.crate_name.clone(),
        module.target_modes.clone(),
        module.capabilities.clone(),
    )
}
