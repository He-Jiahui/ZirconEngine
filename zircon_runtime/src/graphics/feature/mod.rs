mod builtin_render_feature;
mod builtin_render_feature_descriptor;
mod render_feature;
mod render_feature_capability_requirement;
mod render_feature_descriptor;
mod render_feature_pass_descriptor;

pub use builtin_render_feature::BuiltinRenderFeature;
pub(crate) use builtin_render_feature::descriptor_only_advanced_slot_requires_capability_opt_in;
#[cfg(test)]
pub(crate) use builtin_render_feature::descriptor_only_advanced_slots;
pub use render_feature::RenderFeature;
pub use render_feature_capability_requirement::RenderFeatureCapabilityRequirement;
pub use render_feature_descriptor::RenderFeatureDescriptor;
pub use render_feature_pass_descriptor::{
    RenderFeaturePassDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceDescriptor,
    RenderFeatureResourceKind, RenderFeatureResourceWriteMode,
};
