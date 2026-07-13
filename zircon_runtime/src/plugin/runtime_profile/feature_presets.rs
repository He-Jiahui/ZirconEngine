use crate::core::framework::project::RuntimeProfileId;

/// Compile-time Cargo feature requirements for one logical runtime profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeProfileFeaturePreset {
    pub id: RuntimeProfileId,
    pub name: &'static str,
    pub cargo_feature: &'static str,
    pub runtime_features: &'static [&'static str],
    pub app_features: &'static [&'static str],
}

include!(concat!(
    env!("OUT_DIR"),
    "/runtime_profile_feature_presets_generated.rs"
));
