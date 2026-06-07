mod collections;
mod required;

use super::super::super::super::types::OptionalFeatureModuleSignature;

pub(super) fn take_optional_feature_module(
    name: &mut Option<String>,
    kind: &mut Option<zircon_runtime::plugin::PluginModuleKind>,
    crate_name: &mut Option<String>,
    target_modes: &mut Vec<zircon_runtime::RuntimeTargetMode>,
    capabilities: &mut Vec<String>,
) -> Option<OptionalFeatureModuleSignature> {
    let name = name.take()?;
    Some((
        name,
        required::take_required_module_kind(kind),
        required::take_required_crate_name(crate_name),
        collections::take_target_modes(target_modes),
        collections::take_capabilities(capabilities),
    ))
}
