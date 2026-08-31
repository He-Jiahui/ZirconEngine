mod advanced_lighting_compile_inputs;
mod advanced_pbr_pass_contract;
mod ambient_occlusion;
mod compiled_render_pipeline;
mod render_pass_stage;
mod render_pipeline_asset;
mod render_pipeline_compile_options;
mod render_pipeline_compile_report;
mod renderer_asset;
mod renderer_data_document;
mod renderer_feature_asset;
mod renderer_feature_contract_diagnostic;
mod renderer_feature_reference;
mod renderer_feature_source;
mod terminal_surface_pass;

pub(crate) use advanced_lighting_compile_inputs::AdvancedLightingCompileInputs;
pub(crate) use advanced_pbr_pass_contract::{
    ADVANCED_PBR_OPAQUE_EXECUTOR_ID, ADVANCED_PBR_OPAQUE_PASS_NAME, TRANSMISSION_MESH_EXECUTOR_IDS,
    TRANSMISSION_SCENE_COPY_EXECUTOR_IDS, transmission_mesh_pass_name,
    transmission_mesh_step_index, transmission_scene_copy_pass_name,
    transmission_scene_copy_step_index,
};
pub use ambient_occlusion::{
    AO_PROFILE_COMPILER_VERSION, AO_SHADER_INTERFACE_VERSION, AmbientOcclusionDepthConvention,
    AmbientOcclusionInputQualification, AmbientOcclusionInputSemantic, AmbientOcclusionMethod,
    AmbientOcclusionOutputs, AmbientOcclusionProjectionClass, AmbientOcclusionRenderRectKey,
    AoHistoryKey, COMPILED_AO_PROFILE_VERSION, CompiledAoProfile, CompiledAoWorkPlan,
    QualifiedAmbientOcclusionInput,
};
pub use compiled_render_pipeline::CompiledRenderPipeline;
pub(crate) use compiled_render_pipeline::{
    CompiledHistoryEpiloguePlan, CompiledHistoryTextureSource, CompiledRenderPipelineParts,
    RenderGraphExecutionBatch, RenderGraphExecutionCursor, RenderGraphExecutionPass,
    RenderGraphExecutionPassMetadata,
};
pub use render_pass_stage::RenderPassStage;
pub use render_pipeline_asset::RenderPipelineAsset;
pub use render_pipeline_compile_options::RenderPipelineCompileOptions;
pub use render_pipeline_compile_report::RenderPipelineCompileReport;
pub use renderer_asset::RendererAsset;
pub use renderer_data_document::{
    RENDERER_DATA_DOCUMENT_VERSION, RendererDataDocument, RendererDataDocumentError,
    RendererFeatureDocument, RendererFeatureReferenceListKind,
};
pub use renderer_feature_asset::RendererFeatureAsset;
pub use renderer_feature_contract_diagnostic::{
    RendererFeatureContractDiagnostic, RendererFeatureContractDiagnosticSeverity,
};
pub use renderer_feature_reference::RendererFeatureAssetReferences;
pub use renderer_feature_source::RendererFeatureSource;
pub(crate) use terminal_surface_pass::{
    OUTPUT_TARGET_DIRECT_IMPORT_EXECUTOR_ID, OUTPUT_TARGET_DIRECT_IMPORT_PASS_NAME,
    OUTPUT_TARGET_TEXTURE_RESOURCE_NAME, OUTPUT_TARGET_WRITEBACK_EXECUTOR_ID,
    OUTPUT_TARGET_WRITEBACK_PASS_NAME, SURFACE_PRESENT_EXECUTOR_ID, SURFACE_PRESENT_PASS_NAME,
};
