mod definition_value;
mod dependency;
mod entry_point;
mod geometry_source;
mod pipeline_layout;
mod stage;
mod variant_key;
mod variant_miss_report;
mod variant_prewarm;

pub use definition_value::RenderShaderDefinitionValue;
pub use dependency::RenderShaderDependency;
pub use entry_point::RenderShaderEntryPointDescriptor;
pub use geometry_source::{
    GeometrySourceId, GEOMETRY_SOURCE_ID_MORPHED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MESH,
    GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
    GEOMETRY_SOURCE_PLUGIN_ID_START,
};
pub use pipeline_layout::{
    RenderShaderBindGroupLayoutDescriptor, RenderShaderBindingDescriptor,
    RenderShaderBindingResourceType, RenderShaderPipelineLayoutDescriptor,
};
pub use stage::RenderShaderStage;
pub use variant_key::{
    RenderShaderVariantKey, ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
};
pub use variant_miss_report::ShaderVariantMissReport;
pub use variant_prewarm::{
    ShaderVariantPrewarmFailure, ShaderVariantPrewarmManifest, ShaderVariantPrewarmReport,
    ShaderVariantPrewarmRequest,
};
