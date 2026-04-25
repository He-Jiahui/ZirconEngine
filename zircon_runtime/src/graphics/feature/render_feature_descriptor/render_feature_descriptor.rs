use crate::FrameHistoryBinding;

use super::super::render_feature_capability_requirement::RenderFeatureCapabilityRequirement;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFeatureDescriptor {
    pub name: String,
    pub required_extract_sections: Vec<String>,
    pub capability_requirements: Vec<RenderFeatureCapabilityRequirement>,
    pub history_bindings: Vec<FrameHistoryBinding>,
    pub stage_passes: Vec<RenderFeaturePassDescriptor>,
}
