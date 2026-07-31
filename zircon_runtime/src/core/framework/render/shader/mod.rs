mod asset_kind;
mod compute_dispatch;
mod definition_value;
mod dependency;
mod entry_point;
mod fullscreen_pass;
mod geometry_source;
mod ide_env;
mod material_property_layout;
mod module_import;
mod pipeline_layout;
mod queue;
mod render_state;
mod resource;
mod stage;
mod variant_key;
mod variant_miss_report;
mod variant_prewarm;

pub use asset_kind::ShaderAssetKind;
pub use compute_dispatch::{
    COMPUTE_SHADER_FIRST_RESOURCE_BINDING, COMPUTE_SHADER_PARAMS_BINDING,
    COMPUTE_SHADER_RESOURCE_GROUP, ComputeDispatchBuilder, ComputeDispatchPlan, ComputeKernelRef,
    ComputePipelineCacheKey, ShaderAbiBinding, ShaderDispatchBuildDiagnostic, ShaderDispatchExtent,
    ShaderNamedResourceBinding, ShaderParameterValue,
};
pub use definition_value::RenderShaderDefinitionValue;
pub use dependency::RenderShaderDependency;
pub use entry_point::RenderShaderEntryPointDescriptor;
pub use fullscreen_pass::{
    FULLSCREEN_FIRST_PASS_INPUT_BINDING, FULLSCREEN_FRAME_GROUP, FULLSCREEN_PARAMS_BINDING,
    FULLSCREEN_PASS_INPUT_GROUP, FULLSCREEN_TRIANGLE_VERTEX_ENTRY, FullscreenPassBuilder,
    FullscreenPassPlan, FullscreenPipelineCacheKey, FullscreenShaderRef,
};
pub use geometry_source::{
    GEOMETRY_SOURCE_ID_MORPHED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MESH,
    GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
    GEOMETRY_SOURCE_PLUGIN_ID_START, GEOMETRY_SOURCE_WGSL_INCLUDE_MORPHED_MESH,
    GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH, GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MORPHED_MESH,
    GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH, GeometrySourceBindingKind,
    GeometrySourceBindingRequirement, GeometrySourceDescriptor, GeometrySourceId,
    GeometrySourceVertexAttribute, builtin_geometry_source_descriptor,
    builtin_geometry_source_descriptors,
};
pub use ide_env::{
    SHADER_IDE_ENV_CACHE_DIR, SHADER_IDE_ENV_SCHEMA_VERSION, SHADER_IDE_MODULE_MAP_FILE,
    SHADER_IDE_PREVIEW_DEFAULT_VARIANT, ShaderIdeModuleMap, ShaderIdeModuleMapEntry,
    ShaderIdeModuleSource, ShaderIdePreviewMap, ShaderIdePreviewSegment, ShaderIdePreviewVariant,
    shader_ide_generated_material_stub_relative_path, shader_ide_module_stub_relative_path,
    shader_ide_preview_relative_path, shader_ide_preview_segments_relative_path,
    shader_ide_relative_path_string,
};
pub use material_property_layout::{
    MaterialOptionKind, MaterialOptionRef, MaterialOptionTable, MaterialPropertyKind,
    MaterialPropertyLayout, MaterialPropertySlotRef, MaterialTextureBindingRef,
    PropertyScalarClass,
};
pub use module_import::{
    GENERATED_MATERIAL_MODULE_IMPORT_PATH, SHADER_IMPORT_PROJECT_NAMESPACE_SETTING,
    SHADER_SELF_MODULE_NAMESPACE, ShaderImportPathDerivation, ShaderImportPathDerivationError,
    derive_shader_import_path, is_builtin_shader_module_token, is_generated_shader_module_token,
    shader_project_namespace_from_name, strip_wgsl_include_directives, wgsl_include_paths,
};
pub use pipeline_layout::{
    RenderShaderBindGroupLayoutDescriptor, RenderShaderBindingDescriptor,
    RenderShaderBindingResourceType, RenderShaderPipelineLayoutDescriptor,
};
pub use queue::{ShaderQueueDescriptor, ShaderQueueSegment};
pub use render_state::{
    ShaderBlendMode, ShaderCullMode, ShaderDepthCompare, ShaderRenderStateDescriptor,
};
pub use resource::{ShaderResourceAccess, ShaderResourceDescriptor, ShaderResourceKind};
pub use stage::RenderShaderStage;
pub use variant_key::{
    RenderShaderVariantKey, ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
};
pub use variant_miss_report::{
    ShaderVariantMissReport, ShaderVariantRuntimeDimensionCount,
    ShaderVariantRuntimeDimensionSummary,
};
pub use variant_prewarm::{
    ShaderVariantPrewarmDimensionCount, ShaderVariantPrewarmDimensionSummary,
    ShaderVariantPrewarmFailure, ShaderVariantPrewarmManifest, ShaderVariantPrewarmReport,
    ShaderVariantPrewarmRequest, ShaderVariantPrewarmSourceProvenanceEntry,
    ShaderVariantPrewarmSourceProvenanceSummary, ShaderVariantPrewarmWgpuModuleValidationSummary,
    ShaderVariantPrewarmWrittenVariant,
};
