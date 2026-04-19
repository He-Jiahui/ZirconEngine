use crate::FrameHistoryBinding;

use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFeatureDescriptor {
    pub name: String,
    pub required_extract_sections: Vec<String>,
    pub history_bindings: Vec<FrameHistoryBinding>,
    pub stage_passes: Vec<RenderFeaturePassDescriptor>,
}
