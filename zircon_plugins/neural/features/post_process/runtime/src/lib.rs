use zircon_plugin_neural_runtime::NnModelAsset;

mod capability;
mod plugin;

pub use capability::{FEATURE_ID, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    NeuralPostProcessRuntimeFeature,
};

#[cfg(test)]
#[path = "tests/lib_registration_tests.rs"]
mod registration_tests;

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
