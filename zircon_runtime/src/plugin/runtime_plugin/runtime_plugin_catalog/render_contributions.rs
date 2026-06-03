mod feature;
mod prepare;
mod providers;

use crate::plugin::RuntimeExtensionRegistry;

use feature::merge_render_feature_contributions;
use prepare::merge_runtime_prepare_contributions;
use providers::merge_runtime_provider_contributions;

pub(super) fn merge_render_extension_registry_contributions(
    extensions: &RuntimeExtensionRegistry,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    merge_render_feature_contributions(extensions, registry, diagnostics, fatal_diagnostics);
    merge_runtime_prepare_contributions(extensions, registry, diagnostics, fatal_diagnostics);
    merge_runtime_provider_contributions(extensions, registry, diagnostics, fatal_diagnostics);
}
