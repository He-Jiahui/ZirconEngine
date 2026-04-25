use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use crate::RenderFeatureCapabilityRequirement;

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
