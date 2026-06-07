mod enabled;
mod packaging;

pub(super) fn default_packaging(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> Vec<zircon_runtime::plugin::ExportPackagingStrategy> {
    packaging::default_packaging(feature)
}

pub(super) fn enabled_by_default(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> bool {
    enabled::enabled_by_default(feature)
}
