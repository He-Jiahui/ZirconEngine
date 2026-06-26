mod assemble;
mod deferred_gbuffer;
mod include_registry;
mod material_surface;
mod pass_specialization;
mod taa_reactive_mask;
mod validation;

#[cfg(test)]
mod tests;

pub(crate) use assemble::{
    assemble_material_shader_template, MaterialShaderTemplateAssembly,
    MaterialShaderTemplateRequest, ShaderTemplateAssemblyError,
};
pub(crate) use deferred_gbuffer::{
    assemble_deferred_gbuffer_shader_template, DeferredGBufferShaderTemplateRequest,
};
pub(crate) use material_surface::{
    standard_material_surface_source, standard_material_surface_source_for_features,
    StandardMaterialSurfaceSource,
};
pub(crate) use taa_reactive_mask::{
    assemble_taa_reactive_mask_shader_template, TaaReactiveMaskShaderTemplateRequest,
};
pub(crate) use validation::{
    validate_material_shader_template_wgsl, validate_shader_variant_prewarm_wgsl,
    MaterialShaderTemplateValidation, ShaderTemplateValidationError,
};
