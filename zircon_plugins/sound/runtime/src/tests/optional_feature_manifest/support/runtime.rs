mod capabilities;
mod defaults;
mod dependencies;
mod identity;
mod modules;

use super::types::StaticOptionalFeatureManifest;

pub(super) fn optional_feature_signature(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> StaticOptionalFeatureManifest {
    StaticOptionalFeatureManifest {
        id: identity::feature_id(feature),
        display_name: identity::feature_display_name(feature),
        owner_plugin_id: identity::feature_owner_plugin_id(feature),
        capabilities: capabilities::capability_signatures(feature),
        default_packaging: defaults::default_packaging(feature),
        enabled_by_default: defaults::enabled_by_default(feature),
        dependencies: dependencies::dependency_signatures(feature),
        modules: modules::module_signatures(feature),
    }
}
