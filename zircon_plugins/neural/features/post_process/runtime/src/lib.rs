use zircon_plugin_neural_runtime::NnModelAsset;

mod capability;
mod plugin;

pub use capability::{FEATURE_ID, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    NeuralPostProcessRuntimeFeature,
};

#[cfg(test)]
mod registration_tests {
    use super::*;

    #[test]
    fn neural_post_process_registration_delegates_to_the_runtime_manifest_owner() {
        let registration = plugin_feature_registration();

        assert!(registration.is_success(), "{:?}", registration.diagnostics);
        assert_eq!(
            registration.manifest,
            zircon_plugin_neural_runtime::neural_post_process_feature_manifest()
        );
        assert_eq!(registration.manifest.id, FEATURE_ID);
        assert_eq!(registration.manifest.capabilities, [RUNTIME_CAPABILITY]);
        assert!(!registration.manifest.enabled_by_default);
        assert_eq!(
            zircon_plugin_neural_runtime::RENDERING_POST_PROCESS_RUNTIME_CAPABILITY,
            zircon_plugin_rendering_post_process_runtime::RUNTIME_CAPABILITY
        );
        assert!(registration.extensions.modules().is_empty());
        assert!(registration.extensions.managers().is_empty());
        assert!(registration.extensions.shader_module_sources().is_empty());
        assert!(registration.extensions.asset_importers().is_empty());
        assert!(registration.extensions.components().is_empty());
        assert!(registration.extensions.plugin_options().is_empty());
        assert!(registration.extensions.plugin_event_catalogs().is_empty());
        assert!(registration.extensions.scene_hooks().is_empty());
        assert!(registration.extensions.render_features().is_empty());
        assert!(registration.extensions.render_pass_executors().is_empty());
        assert!(registration
            .extensions
            .runtime_prepare_collectors()
            .is_empty());
        assert!(registration
            .extensions
            .hybrid_gi_runtime_providers()
            .is_empty());
        assert!(registration
            .extensions
            .solari_runtime_providers()
            .is_empty());
        assert!(registration
            .extensions
            .virtual_geometry_runtime_providers()
            .is_empty());
        assert!(registration.extensions.geometry_sources().is_empty());
        assert!(registration.extensions.shading_models().is_empty());
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NnInferenceScale {
    Full,
    ThreeQuarters,
    Half,
}

impl NnInferenceScale {
    pub const fn factor(self) -> f32 {
        match self {
            Self::Full => 1.0,
            Self::ThreeQuarters => 0.75,
            Self::Half => 0.5,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NnPostProcessSettings {
    pub model: Option<NnModelAsset>,
    pub intensity: f32,
    pub inference_scale: NnInferenceScale,
    pub enabled: bool,
}

impl Default for NnPostProcessSettings {
    fn default() -> Self {
        Self {
            model: None,
            intensity: 1.0,
            inference_scale: NnInferenceScale::Full,
            enabled: false,
        }
    }
}

impl NnPostProcessSettings {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !(0.0..=1.0).contains(&self.intensity) {
            return Err("neural post-process intensity must be within [0, 1]");
        }
        if self.enabled && self.model.is_none() {
            return Err("an enabled neural post-process effect requires a model asset");
        }
        Ok(())
    }
}
