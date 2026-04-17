use super::super::render_feature_descriptor::RenderFeatureDescriptor;

pub(in crate::feature::builtin_render_feature_descriptor) fn descriptor() -> RenderFeatureDescriptor
{
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
}
