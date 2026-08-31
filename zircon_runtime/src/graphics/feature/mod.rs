mod builtin_render_feature;
mod builtin_render_feature_descriptor;
mod compute_pass_descriptor;
mod render_feature;
mod render_feature_capability_requirement;
mod render_feature_descriptor;
mod render_feature_pass_descriptor;

pub use builtin_render_feature::BuiltinRenderFeature;
pub(crate) use builtin_render_feature::descriptor_only_advanced_slot_requires_capability_opt_in;
#[cfg(test)]
pub(crate) use builtin_render_feature::descriptor_only_advanced_slots;
pub(crate) use builtin_render_feature_descriptor::SsaoParams;
pub(crate) use builtin_render_feature_descriptor::configure_screen_space_ambient_occlusion_for_profile;
pub use builtin_render_feature_descriptor::screen_space_ambient_occlusion_render_feature_descriptor;
pub use compute_pass_descriptor::{
    COMPUTE_GENERIC_EXECUTOR_ID, ComputePassDescriptor, ComputeShaderSource,
};
pub use render_feature::RenderFeature;
pub use render_feature_capability_requirement::RenderFeatureCapabilityRequirement;
pub use render_feature_descriptor::RenderFeatureDescriptor;
pub use render_feature_pass_descriptor::{
    RenderBufferSchema, RenderFeaturePassDescriptor, RenderFeatureResourceAccess,
    RenderFeatureResourceDescriptor, RenderFeatureResourceKind, RenderFeatureResourceVersion,
    RenderFeatureResourceWriteMode, RenderFeatureTextureViewAlias, RenderResourceFallback,
    RenderResourceSchema, RenderTextureExtentPolicy, RenderTextureExtentReference,
    RenderTextureExtentRounding, RenderTextureSchema,
};
