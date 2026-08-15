#[derive(Clone, Copy, Debug, Default)]
pub struct NeuralPostProcessRuntimeFeature;

impl zircon_runtime::plugin::RuntimePluginFeature for NeuralPostProcessRuntimeFeature {
    fn manifest(&self) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
        feature_manifest()
    }
}

pub fn runtime_plugin_feature() -> NeuralPostProcessRuntimeFeature {
    NeuralPostProcessRuntimeFeature
}

pub fn plugin_feature_registration(
) -> zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport {
    zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport::from_feature(
        &runtime_plugin_feature(),
    )
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_neural_runtime::neural_post_process_feature_manifest()
}
