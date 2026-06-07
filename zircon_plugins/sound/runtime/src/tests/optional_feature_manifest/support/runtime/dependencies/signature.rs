use super::super::super::types::OptionalFeatureDependencySignature;

pub(super) fn dependency_signature(
    dependency: &zircon_runtime::plugin::PluginFeatureDependency,
) -> OptionalFeatureDependencySignature {
    (
        dependency.plugin_id.clone(),
        dependency.capability.clone(),
        dependency.primary,
    )
}
