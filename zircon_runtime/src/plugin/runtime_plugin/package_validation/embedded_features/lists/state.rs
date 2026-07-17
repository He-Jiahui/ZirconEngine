pub(super) type RuntimePluginPackageEmbeddedFeatureProviderState<'a> = Vec<(&'a str, &'a str)>;

pub(super) fn new_runtime_plugin_package_embedded_feature_provider_state<'a>(
) -> RuntimePluginPackageEmbeddedFeatureProviderState<'a> {
    Vec::new()
}
