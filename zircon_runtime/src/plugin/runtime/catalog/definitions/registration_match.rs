use crate::plugin::PluginFeatureBundleManifest;

pub(super) fn feature_definition_registration_matches(
    declared: &PluginFeatureBundleManifest,
    registered: &PluginFeatureBundleManifest,
) -> bool {
    declared.id == registered.id
        && declared.owner_plugin_id == registered.owner_plugin_id
        && declared.dependencies == registered.dependencies
        && declared.modules == registered.modules
        && declared.capabilities == registered.capabilities
        && declared.default_packaging == registered.default_packaging
        && declared.enabled_by_default == registered.enabled_by_default
}
