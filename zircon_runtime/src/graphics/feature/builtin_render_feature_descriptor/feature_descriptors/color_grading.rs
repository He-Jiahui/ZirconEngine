use super::super::render_feature_descriptor::RenderFeatureDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "color_grading",
        vec!["view".to_string(), "post_process".to_string()],
        Vec::new(),
        Vec::new(),
    )
}
