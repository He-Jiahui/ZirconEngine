mod assemble;
mod deferred_gbuffer;
mod material_surface;
mod module_registry;
mod pass_specialization;
mod reflection;
mod taa_reactive_mask;
mod validation;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub(crate) use assemble::{
    assemble_material_shader_template, MaterialShaderTemplateAssembly,
    MaterialShaderTemplateRequest, ShaderAssemblySegment, ShaderAssemblySegmentKind,
    ShaderTemplateAssemblyError,
};
pub(crate) use deferred_gbuffer::{
    assemble_deferred_gbuffer_shader_template, DeferredGBufferShaderTemplateRequest,
};
pub(crate) use material_surface::{
    standard_material_surface_source, standard_material_surface_source_for_features,
    StandardMaterialSurfaceSource,
};
pub(crate) use module_registry::{
    builtin_shader_ide_module_includes, environment_standard_pbr_include,
    pbr_extras_include_for_features, ShaderModuleRegistry, ShaderModuleResolutionError,
    ShaderTemplateInclude,
};
pub(crate) use reflection::{
    reflect_declared_shader_resources, reflect_validated_shader_module, ShaderEntryPointReflection,
    ShaderFragmentOutputNumericType, ShaderFragmentOutputScalarKind, ShaderResourceBindingIdentity,
    ShaderResourceBindingReflection, ShaderStageIoReflection, ShaderStageVisibility,
    ShaderTemplateReflection, ShaderVertexInputScalarKind,
};
pub(crate) use taa_reactive_mask::{
    assemble_taa_reactive_mask_shader_template, TaaReactiveMaskShaderTemplateRequest,
};
pub(crate) use validation::{
    validate_material_shader_template_wgsl, validate_material_shader_template_wgsl_with_segments,
    validate_shader_variant_prewarm_wgsl, MaterialShaderTemplateValidation,
    ShaderTemplateValidationError,
};
