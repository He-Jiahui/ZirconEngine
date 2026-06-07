mod display_name;
mod id;
mod owner_plugin;

pub(super) fn feature_id(feature: &zircon_runtime::plugin::PluginFeatureBundleManifest) -> String {
    id::feature_id(feature)
}

pub(super) fn feature_display_name(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> String {
    display_name::feature_display_name(feature)
}

pub(super) fn feature_owner_plugin_id(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> String {
    owner_plugin::feature_owner_plugin_id(feature)
}
