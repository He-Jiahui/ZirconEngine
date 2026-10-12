use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use crate::graphics::RenderFeatureCapabilityRequirement;

// Neural compute 没有内建 pass，仅通过能力要求把插件提供的计算执行纳入编译筛选。
pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new("neural_compute", Vec::new(), Vec::new(), Vec::new())
        .with_capability_requirement(RenderFeatureCapabilityRequirement::NeuralCompute)
}
