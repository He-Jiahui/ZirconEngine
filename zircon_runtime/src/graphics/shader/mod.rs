mod shader_assets;
pub(crate) mod template;
pub(crate) mod variant_cache;

pub use shader_assets::{
    MaterialGraphAsset, ShaderGraphAsset, ShaderProgramAsset, ShaderVariantKey,
};
pub(crate) use template::{
    assemble_deferred_gbuffer_shader_template, assemble_material_shader_template,
    assemble_taa_reactive_mask_shader_template, standard_material_surface_source_for_features,
    DeferredGBufferShaderTemplateRequest, MaterialShaderTemplateAssembly,
    MaterialShaderTemplateRequest, ShaderTemplateAssemblyError,
    TaaReactiveMaskShaderTemplateRequest,
};
pub(crate) use variant_cache::{
    prewarm_shader_variants_to_disk, prewarm_shader_variants_to_disk_with_module_validation,
    ShaderVariantCacheDisk, ShaderVariantCacheDiskKey, ShaderVariantCacheDiskLookup,
};
