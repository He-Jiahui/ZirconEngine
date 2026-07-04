mod ide_env_generation;
mod ide_preview;
mod ide_validation;
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

pub fn builtin_shader_ide_module_sources() -> Vec<ShaderIdeModuleSource> {
    template::builtin_shader_ide_module_includes()
        .into_iter()
        .map(|include| {
            ShaderIdeModuleSource::new(include.token, ShaderAssetKind::Include, include.source)
        })
        .collect()
}

pub(crate) use template::{
    assemble_deferred_gbuffer_shader_template, assemble_material_shader_template,
    assemble_taa_reactive_mask_shader_template, standard_material_surface_source_for_features,
    DeferredGBufferShaderTemplateRequest, MaterialShaderTemplateAssembly,
    MaterialShaderTemplateRequest, ShaderTemplateAssemblyError, ShaderTemplateInclude,
    TaaReactiveMaskShaderTemplateRequest,
};
pub(crate) use variant_cache::{
    prewarm_shader_variants_to_disk,
    prewarm_shader_variants_to_disk_with_module_and_pipeline_validation,
    prewarm_shader_variants_to_disk_with_module_validation,
    prewarm_shader_variants_to_disk_with_pipeline_validation, ShaderVariantCacheDisk,
    ShaderVariantCacheDiskKey, ShaderVariantCacheDiskLookup,
};
