mod builtin_global_shader_contracts;
mod fullscreen_pass_parameters;
mod global_pipeline_layout;
mod ide_env_generation;
mod ide_preview;
mod ide_validation;
mod shader_assets;
pub(crate) mod template;
pub(crate) mod variant_cache;

pub use crate::core::framework::render::ShaderIdePreviewVariant;
use crate::core::framework::render::{ShaderAssetKind, ShaderIdeModuleSource};
pub use ide_env_generation::{ShaderIdeEnvReport, write_shader_ide_env_for_project};
pub use ide_preview::{
    ShaderIdePreviewError, ShaderIdeSurfacePreview, assemble_shader_ide_surface_preview,
};
pub use ide_validation::{
    ShaderIdeWgslCheckError, ShaderIdeWgslModuleValidation, parse_shader_ide_wgsl_module,
    validate_shader_ide_wgsl_module,
};
pub use shader_assets::{
    MaterialGraphAsset, ShaderGraphAsset, ShaderProgramAsset, ShaderVariantKey,
};

pub fn builtin_shader_ide_module_sources() -> Vec<ShaderIdeModuleSource> {
    template::builtin_shader_ide_module_includes()
        .into_iter()
        .map(|include| {
            ShaderIdeModuleSource::new(include.token, ShaderAssetKind::Include, include.source)
        })
        .collect()
}

pub(crate) use template::{
    DeferredGBufferShaderTemplateRequest, MaterialShaderTemplateAssembly,
    MaterialShaderTemplateRequest, ShaderAssemblySegment, ShaderAssemblySegmentKind,
    ShaderTemplateAssemblyError,
    ShaderTemplateInclude, ShaderTemplateValidationError, TaaReactiveMaskShaderTemplateRequest,
    assemble_deferred_gbuffer_shader_template, assemble_material_shader_template,
    assemble_taa_reactive_mask_shader_template, standard_material_surface_source_for_features,
    validate_material_shader_template_wgsl_with_segments,
};

pub(crate) use builtin_global_shader_contracts::{
    HZB_BUILD_PIPELINE_LABEL, HZB_SCENE_DEPTH_RESOURCE, HZB_SOURCE_RESOURCE, HZB_TARGET_RESOURCE,
    MOTION_VECTOR_SOURCE_RESOURCE, MOTION_VECTOR_TILE_SPAN_PARAMETER, hzb_build_dispatch_plan,
    hzb_build_msaa_dispatch_plan, motion_vector_tile_max_pass_plan,
};
pub(crate) use fullscreen_pass_parameters::{
    FullscreenPassParameterBindings, create_fullscreen_pass_parameter_bind_group_layout,
};
pub(crate) use global_pipeline_layout::{
    ShaderWgpuResourceDescriptor, create_compute_shader_bind_group_layout,
    create_fullscreen_pass_input_bind_group_layout,
};
pub(crate) use variant_cache::{
    ShaderVariantCacheDisk, ShaderVariantCacheDiskKey, ShaderVariantCacheDiskLookup,
    prewarm_shader_variants_to_disk, prewarm_shader_variants_to_disk_with_budget,
    prewarm_shader_variants_to_disk_with_module_and_pipeline_validation,
    prewarm_shader_variants_to_disk_with_module_and_pipeline_validation_and_budget,
    prewarm_shader_variants_to_disk_with_module_validation,
    prewarm_shader_variants_to_disk_with_module_validation_and_budget,
    prewarm_shader_variants_to_disk_with_pipeline_validation,
    prewarm_shader_variants_to_disk_with_pipeline_validation_and_budget,
};
