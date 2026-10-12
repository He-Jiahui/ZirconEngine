//! 管理网格各 pass 的管线准入、材质代际与提交引用；着色器装配和 ABI 校验由相邻模块实现。
#[path = "construct.rs"]
mod construct;
#[path = "../../../../mesh_pipeline_cache/ensure_depth_prepass_pipeline.rs"]
mod ensure_depth_prepass_pipeline;
#[path = "../../../../mesh/mesh_pipeline_cache/ensure_gbuffer_pipeline.rs"]
mod ensure_gbuffer_pipeline;
#[path = "../../../../mesh_pipeline_cache/ensure_hit_proxy_pipeline.rs"]
mod ensure_hit_proxy_pipeline;
#[path = "../../../../mesh/mesh_pipeline_cache/ensure_oit_pipeline.rs"]
mod ensure_oit_pipeline;
#[path = "../../../mesh/mesh_pipeline_cache/ensure_pipeline.rs"]
mod ensure_pipeline;
#[path = "../../../../mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs"]
mod ensure_shadow_pipeline;
#[path = "../../../../ensure_taa_reactive_mask_pipeline.rs"]
mod ensure_taa_reactive_mask_pipeline;
#[path = "../../../../mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs"]
mod ensure_velocity_pipeline;
#[path = "../../../../mesh/mesh_pipeline_cache/forward_shadow_receiver.rs"]
mod forward_shadow_receiver;
#[path = "../../../../material_pipeline_generate_admission.rs"]
mod material_pipeline_generation_admission;
#[path = "../../../../mesh_pipeline_cache/material_pipeline_publish.rs"]
mod material_pipeline_publication;
#[path = "../../../../mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs"]
mod mesh_pipeline_cache;
#[path = "../../../mesh/mesh_pipeline_cache/submission_usage.rs"]
mod mesh_pipeline_submission_usage;
#[path = "../../../mesh/mesh_pipeline_cache/variant_registry.rs"]
mod mesh_pipeline_variant_registry;
#[path = "../../../../mesh/mesh_pipeline_cache/shader_entry_contract.rs"]
mod mesh_shader_entry_contract;
#[path = "../../../../mesh/mesh_pipeline_cache/shader_fragment_contract.rs"]
mod mesh_shader_fragment_contract;
#[path = "../../../../mesh_pipeline_cache/shader_fragment_contract_wgpu.rs"]
mod mesh_shader_fragment_contract_wgpu;
#[path = "../../../../mesh/mesh_pipeline_cache/shader_resource_contract.rs"]
mod mesh_shader_resource_contract;
#[path = "../../../../mesh_pipeline_cache/shader_resource_contract_wgpu.rs"]
mod mesh_shader_resource_contract_wgpu;
#[path = "../../../../mesh/mesh_pipeline_cache/shader_vertex_contract.rs"]
mod mesh_shader_vertex_contract;
#[path = "../../../../mesh_pipeline_cache/shader_vertex_contract_wgpu.rs"]
mod mesh_shader_vertex_contract_wgpu;
#[cfg(test)]
#[path = "../tests/admission_contract.rs"]
mod pipeline_admission_contract_tests;
#[path = "../../../../mesh/mesh_pipeline_cache/creation_diagnostics.rs"]
mod pipeline_creation_diagnostics;
#[path = "../../../mesh/mesh_pipeline_cache/creation_metrics.rs"]
mod pipeline_creation_metrics;
#[path = "../../../../mesh/mesh_pipeline_cache/shader_module_references.rs"]
mod pipeline_shader_module_references;
#[path = "../../../mesh/mesh_pipeline_cache/prewarm_manifest.rs"]
mod prewarm_manifest;
#[path = "../../../../mesh_pipeline_cache/prewarm_pipeline_validate.rs"]
mod prewarm_pipeline_validation;
#[path = "../../../mesh/mesh_pipeline_cache/shader_source.rs"]
mod shader_source;
#[path = "../../../../shader_source_validate_admission.rs"]
mod shader_source_validation_admission;
#[path = "../../../../mesh_pipeline_cache/shader_source_validate_states.rs"]
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
    MeshPipelineResolverConfigurationEpoch, MeshPipelineVariantRegistry,
    MeshPipelineVariantResolver,
};
pub use prewarm_manifest::{
    RuntimeShaderPipelinePrewarmFailure, RuntimeShaderPipelinePrewarmReport,
};
pub(crate) use prewarm_pipeline_validation::{
    create_mesh_prewarm_validation_pipeline_layout, validate_mesh_prewarm_request_render_pipeline,
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
pub(crate) use shader_source::{
    mesh_pipeline_standard_material_template_source,
    mesh_pipeline_standard_material_template_source_for_geometry,
    mesh_pipeline_standard_material_template_source_for_shader_pass,
    mesh_pipeline_standard_material_template_source_for_shader_pass_and_descriptor,
    MeshPipelineShaderSource,
};
