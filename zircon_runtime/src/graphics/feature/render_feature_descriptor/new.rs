use crate::FrameHistoryBinding;

use super::render_feature_descriptor::RenderFeatureDescriptor;
use crate::graphics::feature::{RenderFeatureCapabilityRequirement, RenderFeaturePassDescriptor};

impl RenderFeatureDescriptor {
    pub fn new(
        name: impl Into<String>,
        required_extract_sections: Vec<String>,
        history_bindings: Vec<FrameHistoryBinding>,
        stage_passes: Vec<RenderFeaturePassDescriptor>,
    ) -> Self {
        Self {
            name: name.into(),
            required_extract_sections,
            capability_requirements: Vec::new(),
            history_bindings,
            stage_passes,
        }
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
}
