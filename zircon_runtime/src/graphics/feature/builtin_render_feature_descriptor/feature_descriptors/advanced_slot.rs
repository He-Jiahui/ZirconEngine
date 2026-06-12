use super::super::super::builtin_render_feature::AdvancedBuiltinFeatureSlot;
use super::super::render_feature_descriptor::RenderFeatureDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
    slot: &AdvancedBuiltinFeatureSlot,
) -> RenderFeatureDescriptor {
    let mut descriptor = RenderFeatureDescriptor::new(
        slot.descriptor_name(),
        slot.extract_section()
            .into_iter()
            .map(str::to_string)
            .collect(),
        Vec::new(),
        Vec::new(),
    );
    if let Some(requirement) = slot.capability_requirement() {
        descriptor = descriptor.with_capability_requirement(requirement);
    }
    descriptor
}
