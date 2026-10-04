mod binding_contract;
mod builtin_global_shader_contracts;
mod fullscreen_pass_parameters;
mod global_pipeline_layout;
mod ide_env_generation;
mod ide_preview;
mod ide_validation;
pub mod invocation;
mod shader_assets;
pub(crate) mod template;
pub(crate) mod variant_cache;

pub use crate::core::framework::render::ShaderIdePreviewVariant;
use crate::core::framework::render::{ShaderAssetKind, ShaderIdeModuleSource};
pub use ide_env_generation::{write_shader_ide_env_for_project, ShaderIdeEnvReport};
pub use ide_preview::{
    assemble_shader_ide_surface_preview, ShaderIdePreviewError, ShaderIdeSurfacePreview,
};
pub use ide_validation::{
    parse_shader_ide_wgsl_module, validate_shader_ide_wgsl_module, ShaderIdeWgslCheckError,
    ShaderIdeWgslModuleValidation,
};
pub use shader_assets::{
    MaterialGraphAsset, ShaderGraphAsset, ShaderProgramAsset, ShaderVariantKey,
};

pub use invocation::{
    ComputeDispatchBuilder, ComputeDispatchPlan, ComputeKernelRef, ComputePipelineCacheKey,
    FullscreenPassBuilder, FullscreenPassPlan, FullscreenPipelineCacheKey, FullscreenShaderRef,
    ShaderAbiBinding, ShaderDispatchBuildDiagnostic, ShaderDispatchExtent,
    ShaderNamedResourceBinding, ShaderParameterValue, COMPUTE_SHADER_FIRST_RESOURCE_BINDING,
    COMPUTE_SHADER_PARAMS_BINDING, COMPUTE_SHADER_RESOURCE_GROUP,
    FULLSCREEN_FIRST_PASS_INPUT_BINDING, FULLSCREEN_FRAME_GROUP, FULLSCREEN_PARAMS_BINDING,
    FULLSCREEN_PASS_INPUT_GROUP, FULLSCREEN_TRIANGLE_VERTEX_ENTRY,
};

pub fn builtin_shader_ide_module_sources() -> Vec<ShaderIdeModuleSource> {
    template::builtin_shader_ide_module_includes()
        .into_iter()
        .map(|include| {
            ShaderIdeModuleSource::new(include.token, ShaderAssetKind::Include, include.source)
        })
        .collect()
}

pub(crate) use binding_contract::{
    ShaderBindingResourceType, ShaderBindingStage, ShaderBindingVisibility,
    ShaderTextureSampleType, ShaderTextureViewDimension,
};
pub(crate) use template::{
    assemble_deferred_gbuffer_shader_template, assemble_material_shader_template,
    assemble_taa_reactive_mask_shader_template, reflect_declared_shader_resources,
    reflect_validated_shader_module, standard_material_surface_source_for_features,
    validate_material_shader_template_wgsl_with_segments, DeferredGBufferShaderTemplateRequest,
    MaterialShaderTemplateAssembly, MaterialShaderTemplateRequest, ShaderAssemblySegment,
    ShaderAssemblySegmentKind, ShaderStageVisibility, ShaderTemplateAssemblyError,
    ShaderTemplateInclude, ShaderTemplateReflection, ShaderTemplateValidationError,
    TaaReactiveMaskShaderTemplateRequest,
};

pub(crate) use builtin_global_shader_contracts::{
    hzb_build_dispatch_plan, hzb_build_msaa_dispatch_plan, motion_vector_tile_max_pass_plan,
    HZB_BUILD_PIPELINE_LABEL, HZB_SCENE_DEPTH_RESOURCE, HZB_SOURCE_RESOURCE, HZB_TARGET_RESOURCE,
    MOTION_VECTOR_SOURCE_RESOURCE, MOTION_VECTOR_TILE_SPAN_PARAMETER,
};
pub(crate) use fullscreen_pass_parameters::{
    create_fullscreen_pass_parameter_bind_group_layout, FullscreenPassParameterBindings,
};
pub(crate) use global_pipeline_layout::{
    create_compute_shader_bind_group_layout, create_fullscreen_pass_input_bind_group_layout,
    ShaderWgpuResourceDescriptor,
};
pub(crate) use variant_cache::{
    prewarm_shader_variants_to_disk, prewarm_shader_variants_to_disk_with_budget,
    prewarm_shader_variants_to_disk_with_module_and_pipeline_validation,
    prewarm_shader_variants_to_disk_with_module_and_pipeline_validation_and_budget,
    prewarm_shader_variants_to_disk_with_module_validation,
    prewarm_shader_variants_to_disk_with_module_validation_and_budget,
    prewarm_shader_variants_to_disk_with_pipeline_validation,
    prewarm_shader_variants_to_disk_with_pipeline_validation_and_budget, ShaderVariantCacheDisk,
    ShaderVariantCacheDiskKey, ShaderVariantCacheDiskLookup,
};
