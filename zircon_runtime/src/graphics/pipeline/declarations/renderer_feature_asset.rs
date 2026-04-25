use std::collections::BTreeMap;

use crate::graphics::feature::{
    BuiltinRenderFeature, RenderFeatureCapabilityRequirement, RenderFeatureDescriptor,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RendererFeatureAsset {
    pub feature: BuiltinRenderFeature,
    pub enabled: bool,
    pub local_config: BTreeMap<String, String>,
    pub quality_gate: Option<BuiltinRenderFeature>,
    pub capability_requirements: Vec<RenderFeatureCapabilityRequirement>,
    pub descriptor_override: Option<RenderFeatureDescriptor>,
}

impl RendererFeatureAsset {
    pub fn builtin(feature: BuiltinRenderFeature) -> Self {
        Self {
            feature,
            enabled: true,
            local_config: BTreeMap::new(),
            quality_gate: Some(feature),
            capability_requirements: Vec::new(),
            descriptor_override: None,
        }
    }

    pub fn disabled(feature: BuiltinRenderFeature) -> Self {
        Self {
            enabled: false,
            ..Self::builtin(feature)
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_local_config(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.local_config.insert(key.into(), value.into());
        self
    }

    pub fn with_quality_gate(mut self, gate: BuiltinRenderFeature) -> Self {
        self.quality_gate = Some(gate);
        self
    }

    pub fn without_quality_gate(mut self) -> Self {
        self.quality_gate = None;
        self
    }

    pub fn with_capability_requirement(
        mut self,
        requirement: RenderFeatureCapabilityRequirement,
    ) -> Self {
        if !self.capability_requirements.contains(&requirement) {
            self.capability_requirements.push(requirement);
        }
        self
    }

    pub fn with_descriptor_override(mut self, descriptor: RenderFeatureDescriptor) -> Self {
        self.descriptor_override = Some(descriptor);
        self
    }

    pub fn without_descriptor_override(mut self) -> Self {
        self.descriptor_override = None;
        self
    }
}
