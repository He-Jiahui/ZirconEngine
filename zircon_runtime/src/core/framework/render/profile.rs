use serde::{Deserialize, Serialize};

use super::{
    AdvancedRenderFeature, RenderCapabilityKind, RenderCapabilityMismatchDetail,
    RenderCapabilitySummary, RenderSubmissionConfig, SolariCapabilityRequirement,
};

pub const RENDER_PROFILE_CONFIG_KEY: &str = "zircon.render.profile_bundle";

#[cfg(test)]
#[path = "profile/tests/capacity_tests.rs"]
mod capacity_tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RenderProductProfile {
    Headless,
    CommonRenderApi,
    Render2d,
    Render3d,
    Ui,
    DefaultRender,
    AdvancedRender,
    SolariExperimental,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RenderProductFeature {
    Camera,
    Image,
    Mesh,
    Material,
    Shader,
    Light,
    CorePipeline,
    Pbr,
    Sprite,
    UiRender,
    RenderTarget,
    PostProcess,
    AntiAlias,
    VirtualGeometry,
    HybridGlobalIllumination,
    Solari,
}

/// profile bundle 是可序列化的能力声明，组合产品档位、包含项、特性和提交配置；运行时据此校验请求边界，不宣称某项 GPU 功能已经执行。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderProfileBundle {
    profile: RenderProductProfile,
    includes: Vec<RenderProductProfile>,
    features: Vec<RenderProductFeature>,
    #[serde(default)]
    submission_config: RenderSubmissionConfig,
}

impl RenderProfileBundle {
    pub fn new(profile: RenderProductProfile) -> Self {
        Self {
            profile,
            includes: Vec::new(),
            features: Vec::new(),
            submission_config: RenderSubmissionConfig::default(),
        }
    }

    pub fn headless() -> Self {
        Self::new(RenderProductProfile::Headless)
    }

    pub fn common_render_api() -> Self {
        Self::new(RenderProductProfile::CommonRenderApi).with_features(common_api_features())
    }

    pub fn render_2d() -> Self {
        Self::new(RenderProductProfile::Render2d).with_features(render_2d_features())
    }

    pub fn render_3d() -> Self {
        Self::new(RenderProductProfile::Render3d).with_features(render_3d_features())
    }

    pub fn ui() -> Self {
        Self::new(RenderProductProfile::Ui).with_features(ui_features())
    }

    pub fn default_render() -> Self {
        Self::new(RenderProductProfile::DefaultRender)
            .with_includes([
                RenderProductProfile::CommonRenderApi,
                RenderProductProfile::Render2d,
                RenderProductProfile::Render3d,
                RenderProductProfile::Ui,
            ])
            .with_features(default_render_features())
    }

    pub fn advanced_render() -> Self {
        Self::new(RenderProductProfile::AdvancedRender)
            .with_includes([RenderProductProfile::DefaultRender])
            .with_features(advanced_render_features())
    }

    pub fn solari_experimental() -> Self {
        Self::new(RenderProductProfile::SolariExperimental)
            .with_includes([RenderProductProfile::AdvancedRender])
            .with_features(solari_features())
    }

    pub const fn profile(&self) -> RenderProductProfile {
        self.profile
    }

    pub fn includes(&self) -> &[RenderProductProfile] {
        &self.includes
    }

    pub fn features(&self) -> &[RenderProductFeature] {
        &self.features
    }

    /// Keeps scheduling independent from the selected render feature set.
    pub const fn submission_config(&self) -> RenderSubmissionConfig {
        self.submission_config
    }

    pub fn with_includes(
        mut self,
        includes: impl IntoIterator<Item = RenderProductProfile>,
    ) -> Self {
        for include in includes {
            push_unique(&mut self.includes, include);
        }
        self
    }

    pub fn with_features(
        mut self,
        features: impl IntoIterator<Item = RenderProductFeature>,
    ) -> Self {
        for feature in features {
            push_unique(&mut self.features, feature);
        }
        self
    }

    pub fn with_submission_config(mut self, submission_config: RenderSubmissionConfig) -> Self {
        self.submission_config = submission_config;
        self
    }

    pub fn enables(&self, profile: RenderProductProfile) -> bool {
        self.required_profiles().contains(&profile)
    }

    pub fn has_feature(&self, feature: RenderProductFeature) -> bool {
        self.features.contains(&feature)
    }

    pub fn features_without(&self, feature: RenderProductFeature) -> Vec<RenderProductFeature> {
        self.features
            .iter()
            .copied()
            .filter(|candidate| *candidate != feature)
            .collect()
    }

    pub fn validate(&self) -> Result<(), RenderProfileValidationError> {
        for profile in self.required_profiles() {
            for feature in required_features_for_profile(profile) {
                if !self.has_feature(*feature) {
                    return Err(RenderProfileValidationError::MissingRequiredFeature {
                        profile,
                        feature: *feature,
                    });
                }
            }
        }
        Ok(())
    }

    pub fn validate_capabilities(
        &self,
        capabilities: &RenderCapabilitySummary,
    ) -> Result<(), RenderProfileValidationError> {
        self.validate()?;
        for capability in self.required_capabilities() {
            if !capability.is_satisfied_by(capabilities) {
                return Err(RenderProfileValidationError::MissingBackendCapability {
                    profile: self.profile,
                    detail: RenderCapabilityMismatchDetail::new(capability),
                });
            }
        }
        Ok(())
    }

    fn required_profiles(&self) -> Vec<RenderProductProfile> {
        let profile_capacity = self.includes.len().saturating_add(1);
        let mut profiles = Vec::with_capacity(profile_capacity);
        push_unique(&mut profiles, self.profile);
        for include in &self.includes {
            push_unique(&mut profiles, *include);
            for nested in implied_profiles(*include) {
                push_unique(&mut profiles, *nested);
            }
        }
        for nested in implied_profiles(self.profile) {
            push_unique(&mut profiles, *nested);
        }
        profiles
    }

    fn required_capabilities(&self) -> Vec<RenderCapabilityKind> {
        let virtual_geometry = self.has_feature(RenderProductFeature::VirtualGeometry);
        let hybrid_global_illumination =
            self.has_feature(RenderProductFeature::HybridGlobalIllumination);
        let anti_alias = self.has_feature(RenderProductFeature::AntiAlias);
        let solari = self.has_feature(RenderProductFeature::Solari);
        let capability_capacity = usize::from(virtual_geometry)
            * AdvancedRenderFeature::VirtualGeometry
                .required_capabilities()
                .len()
            + usize::from(hybrid_global_illumination)
                * AdvancedRenderFeature::HybridGlobalIllumination
                    .required_capabilities()
                    .len()
            + usize::from(anti_alias)
            + usize::from(solari) * SolariCapabilityRequirement::ALL.len();
        let mut capabilities = Vec::with_capacity(capability_capacity);
        if virtual_geometry {
            for capability in AdvancedRenderFeature::VirtualGeometry.required_capabilities() {
                push_unique(&mut capabilities, *capability);
            }
        }
        if hybrid_global_illumination {
            for capability in
                AdvancedRenderFeature::HybridGlobalIllumination.required_capabilities()
            {
                push_unique(&mut capabilities, *capability);
            }
        }
        if anti_alias {
            push_unique(
                &mut capabilities,
                RenderCapabilityKind::ScreenSpaceAntiAlias,
            );
        }
        if solari {
            for requirement in SolariCapabilityRequirement::ALL {
                push_unique(&mut capabilities, requirement.capability_kind());
            }
        }
        capabilities
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderProfileValidationError {
    MissingRequiredFeature {
        profile: RenderProductProfile,
        feature: RenderProductFeature,
    },
    MissingBackendCapability {
        profile: RenderProductProfile,
        detail: RenderCapabilityMismatchDetail,
    },
}

fn common_api_features() -> Vec<RenderProductFeature> {
    vec![
        RenderProductFeature::Camera,
        RenderProductFeature::Image,
        RenderProductFeature::Mesh,
        RenderProductFeature::Material,
        RenderProductFeature::Shader,
    ]
}

fn render_2d_features() -> Vec<RenderProductFeature> {
    let mut features = common_api_features();
    for feature in [
        RenderProductFeature::Sprite,
        RenderProductFeature::CorePipeline,
        RenderProductFeature::PostProcess,
    ] {
        push_unique(&mut features, feature);
    }
    features
}

fn render_3d_features() -> Vec<RenderProductFeature> {
    let mut features = common_api_features();
    for feature in [
        RenderProductFeature::Light,
        RenderProductFeature::Pbr,
        RenderProductFeature::CorePipeline,
        RenderProductFeature::PostProcess,
        RenderProductFeature::AntiAlias,
    ] {
        push_unique(&mut features, feature);
    }
    features
}

fn ui_features() -> Vec<RenderProductFeature> {
    let mut features = common_api_features();
    for feature in [
        RenderProductFeature::UiRender,
        RenderProductFeature::CorePipeline,
        RenderProductFeature::RenderTarget,
    ] {
        push_unique(&mut features, feature);
    }
    features
}

fn default_render_features() -> Vec<RenderProductFeature> {
    let mut features = Vec::new();
    for feature in render_2d_features()
        .into_iter()
        .chain(render_3d_features())
        .chain(ui_features())
    {
        push_unique(&mut features, feature);
    }
    features
}

fn advanced_render_features() -> Vec<RenderProductFeature> {
    let mut features = default_render_features();
    for feature in [
        RenderProductFeature::VirtualGeometry,
        RenderProductFeature::HybridGlobalIllumination,
    ] {
        push_unique(&mut features, feature);
    }
    features
}

fn solari_features() -> Vec<RenderProductFeature> {
    let mut features = advanced_render_features();
    push_unique(&mut features, RenderProductFeature::Solari);
    features
}

fn required_features_for_profile(profile: RenderProductProfile) -> &'static [RenderProductFeature] {
    match profile {
        RenderProductProfile::Headless | RenderProductProfile::CommonRenderApi => &[],
        RenderProductProfile::Render2d => &[
            RenderProductFeature::Camera,
            RenderProductFeature::Image,
            RenderProductFeature::Mesh,
            RenderProductFeature::Material,
            RenderProductFeature::Shader,
            RenderProductFeature::Sprite,
            RenderProductFeature::CorePipeline,
        ],
        RenderProductProfile::Render3d => &[
            RenderProductFeature::Camera,
            RenderProductFeature::Image,
            RenderProductFeature::Mesh,
            RenderProductFeature::Material,
            RenderProductFeature::Shader,
            RenderProductFeature::Light,
            RenderProductFeature::Pbr,
            RenderProductFeature::CorePipeline,
            RenderProductFeature::PostProcess,
            RenderProductFeature::AntiAlias,
        ],
        RenderProductProfile::Ui => &[
            RenderProductFeature::UiRender,
            RenderProductFeature::CorePipeline,
            RenderProductFeature::RenderTarget,
        ],
        RenderProductProfile::DefaultRender
        | RenderProductProfile::AdvancedRender
        | RenderProductProfile::SolariExperimental => &[],
    }
}

fn implied_profiles(profile: RenderProductProfile) -> &'static [RenderProductProfile] {
    match profile {
        RenderProductProfile::DefaultRender => &[
            RenderProductProfile::CommonRenderApi,
            RenderProductProfile::Render2d,
            RenderProductProfile::Render3d,
            RenderProductProfile::Ui,
        ],
        RenderProductProfile::AdvancedRender => &[RenderProductProfile::DefaultRender],
        RenderProductProfile::SolariExperimental => &[RenderProductProfile::AdvancedRender],
        _ => &[],
    }
}

fn push_unique<T: Copy + PartialEq>(values: &mut Vec<T>, value: T) {
    if !values.contains(&value) {
        values.push(value);
    }
}

#[cfg(test)]
#[path = "tests/profile.rs"]
mod tests;
