use super::super::super::super::types::PendingOptionalFeatureManifest;

pub(in super::super::super) fn push_optional_feature_module(
    feature: &mut Option<PendingOptionalFeatureManifest>,
    name: &mut Option<String>,
    kind: &mut Option<zircon_runtime::plugin::PluginModuleKind>,
    crate_name: &mut Option<String>,
    target_modes: &mut Vec<zircon_runtime::core::framework::platform::RuntimeTargetMode>,
    capabilities: &mut Vec<String>,
) {
    let Some(module) = super::signature::take_optional_feature_module(
        name,
        kind,
        crate_name,
        target_modes,
        capabilities,
    ) else {
        return;
    };
    super::append::append_optional_feature_module(feature, module);
}
