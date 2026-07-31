use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use crate::graphics::RenderFeatureCapabilityRequirement;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor()
-> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new("neural_compute", Vec::new(), Vec::new(), Vec::new())
        .with_capability_requirement(RenderFeatureCapabilityRequirement::NeuralCompute)
}
