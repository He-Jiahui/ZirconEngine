use super::super::render_feature_descriptor::RenderFeatureDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
    name: &'static str,
    extract_section: &'static str,
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        name,
        vec![extract_section.to_string()],
        Vec::new(),
        Vec::new(),
    )
}
