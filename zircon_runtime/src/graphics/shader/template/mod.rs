mod assemble;
mod deferred_gbuffer;
mod material_surface;
mod module_registry;
mod pass_specialization;
mod taa_reactive_mask;
mod validation;

#[cfg(test)]
mod tests;

pub(crate) use assemble::{
    MaterialShaderTemplateAssembly, MaterialShaderTemplateRequest, ShaderAssemblySegment,
    ShaderAssemblySegmentKind, ShaderTemplateAssemblyError, assemble_material_shader_template,
};
pub(crate) use deferred_gbuffer::{
    DeferredGBufferShaderTemplateRequest, assemble_deferred_gbuffer_shader_template,
};
pub(crate) use material_surface::{
    StandardMaterialSurfaceSource, standard_material_surface_source,
    standard_material_surface_source_for_features,
};
pub(crate) use module_registry::{
    ShaderModuleRegistry, ShaderModuleResolutionError, ShaderTemplateInclude,
    builtin_shader_ide_module_includes, environment_standard_pbr_include,
};
pub(crate) use taa_reactive_mask::{
    TaaReactiveMaskShaderTemplateRequest, assemble_taa_reactive_mask_shader_template,
};
pub(crate) use validation::{
    MaterialShaderTemplateValidation, ShaderTemplateValidationError,
    validate_material_shader_template_wgsl, validate_material_shader_template_wgsl_with_segments,
    validate_shader_variant_prewarm_wgsl,
};
