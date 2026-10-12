use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use crate::graphics::RenderFeatureCapabilityRequirement;

// Ray tracing 只声明 view/geometry/visibility 提取依赖及加速结构、光追管线能力要求；描述符未提供内建 pass。
pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "ray_tracing",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        Vec::new(),
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::AccelerationStructures)
    .with_capability_requirement(RenderFeatureCapabilityRequirement::RayTracingPipeline)
}
