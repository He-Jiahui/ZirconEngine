use super::super::super::types::StaticOptionalFeatureManifest;

pub(in super::super::super) fn optional_feature_signature(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> StaticOptionalFeatureManifest {
    StaticOptionalFeatureManifest {
        id: super::super::identity::feature_id(feature),
        display_name: super::super::identity::feature_display_name(feature),
        owner_plugin_id: super::super::identity::feature_owner_plugin_id(feature),
        capabilities: super::super::capabilities::capability_signatures(feature),
        default_packaging: super::super::defaults::default_packaging(feature),
        enabled_by_default: super::super::defaults::enabled_by_default(feature),
        dependencies: super::super::dependencies::dependency_signatures(feature),
        modules: super::super::modules::module_signatures(feature),
    }
}
