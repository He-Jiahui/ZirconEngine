use crate::FrameHistoryBinding;

use super::render_feature_descriptor::RenderFeatureDescriptor;
use crate::feature::RenderFeaturePassDescriptor;

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
            history_bindings,
            stage_passes,
        }
    }
}
