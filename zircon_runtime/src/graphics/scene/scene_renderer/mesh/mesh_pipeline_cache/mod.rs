mod construct;
mod ensure_depth_prepass_pipeline;
mod ensure_gbuffer_pipeline;
mod ensure_hit_proxy_pipeline;
mod ensure_oit_pipeline;
mod ensure_pipeline;
mod ensure_shadow_pipeline;
mod ensure_taa_reactive_mask_pipeline;
mod ensure_velocity_pipeline;
mod forward_shadow_receiver;
mod material_pipeline_generation_admission;
mod material_pipeline_publication;
mod mesh_pipeline_cache;
mod mesh_pipeline_submission_usage;
mod mesh_pipeline_variant_registry;
mod mesh_shader_entry_contract;
mod mesh_shader_fragment_contract;
mod mesh_shader_fragment_contract_wgpu;
mod mesh_shader_resource_contract;
mod mesh_shader_resource_contract_wgpu;
mod mesh_shader_vertex_contract;
mod mesh_shader_vertex_contract_wgpu;
#[cfg(test)]
mod pipeline_admission_contract_tests;
mod pipeline_creation_diagnostics;
mod pipeline_creation_metrics;
mod pipeline_shader_module_references;
mod prewarm_manifest;
mod prewarm_pipeline_validation;
mod shader_source;
mod shader_source_validation_admission;
mod shader_source_validation_states;

pub(crate) use ensure_pipeline::EnvironmentOnlyPbrBasePipelinePrewarmReport;
pub(in crate::graphics::scene::scene_renderer::mesh) use forward_shadow_receiver::create_forward_shadow_receiver_layout;
pub(crate) use material_pipeline_publication::{
    MaterialPipelinePublicationAdmission, MaterialPipelineRequirement,
    MaterialPipelineRequirementSet,
};
pub(crate) use mesh_pipeline_cache::MeshPipelineCache;
use mesh_pipeline_cache::PipelineAdmissionKey;
pub(in crate::graphics::scene::scene_renderer::mesh) use mesh_pipeline_cache::PipelineCreationTarget;
pub(in crate::graphics::scene::scene_renderer::mesh) use mesh_pipeline_cache::{
    AsyncBasePipelineCompileResult, AsyncBasePipelineProduct, MAX_ASYNC_BASE_PIPELINES_IN_FLIGHT,
    MAX_ASYNC_SHADER_SOURCE_VALIDATIONS_IN_FLIGHT, MAX_PENDING_PIPELINE_CREATION_DIAGNOSTICS,
};
pub(crate) use mesh_pipeline_variant_registry::{
    MeshPipelineVariantRegistry, MeshPipelineVariantResolver,
};
pub use prewarm_manifest::{
    RuntimeShaderPipelinePrewarmFailure, RuntimeShaderPipelinePrewarmReport,
};
pub(crate) use prewarm_pipeline_validation::{
    create_mesh_prewarm_validation_pipeline_layout, validate_mesh_prewarm_request_render_pipeline,
};
pub(crate) use shader_source::{
    MeshPipelineShaderSource, mesh_pipeline_standard_material_template_source,
    mesh_pipeline_standard_material_template_source_for_geometry,
    mesh_pipeline_standard_material_template_source_for_shader_pass,
    mesh_pipeline_standard_material_template_source_for_shader_pass_and_descriptor,
};
#[cfg(test)]
pub(in crate::graphics::scene::scene_renderer::mesh) use shader_source::{
    mesh_pipeline_deferred_gbuffer_template_source_for_geometry,
    mesh_pipeline_depth_prepass_template_source_for_geometry,
    mesh_pipeline_hit_proxy_template_source_for_geometry,
    mesh_pipeline_shadow_template_source_for_geometry,
    mesh_pipeline_taa_reactive_mask_template_source_for_geometry,
    mesh_pipeline_velocity_template_source_for_geometry,
};
