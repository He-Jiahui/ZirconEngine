#[path = "../../declarations/advanced_lighting_compile_inputs.rs"]
mod advanced_lighting_compile_inputs;
#[path = "advanced_pbr_pass_contract.rs"]
mod advanced_pbr_pass_contract;
#[path = "ambient_occlusion.rs"]
mod ambient_occlusion;
#[path = "compiled_render_pipeline.rs"]
mod compiled_render_pipeline;
#[path = "render_pass_stage.rs"]
mod render_pass_stage;
#[path = "render_pipeline_asset.rs"]
mod render_pipeline_asset;
#[path = "../../declarations/render_pipeline_compile_options.rs"]
mod render_pipeline_compile_options;
#[path = "../../declarations/render_pipeline_compile_report.rs"]
mod render_pipeline_compile_report;
#[path = "renderer_asset.rs"]
mod renderer_asset;
#[path = "renderer_data_document.rs"]
mod renderer_data_document;
#[path = "renderer_feature_asset.rs"]
mod renderer_feature_asset;
#[path = "../../declarations/renderer_feature_contract_diagnostic.rs"]
mod renderer_feature_contract_diagnostic;
#[path = "renderer_feature_reference.rs"]
mod renderer_feature_reference;
#[path = "renderer_feature_source.rs"]
mod renderer_feature_source;
#[path = "terminal_surface_pass.rs"]
mod terminal_surface_pass;

pub(crate) use advanced_lighting_compile_inputs::AdvancedLightingCompileInputs;
pub(crate) use advanced_pbr_pass_contract::{
    transmission_mesh_pass_name, transmission_mesh_step_index, transmission_scene_copy_pass_name,
    transmission_scene_copy_step_index, ADVANCED_PBR_OPAQUE_EXECUTOR_ID,
    ADVANCED_PBR_OPAQUE_PASS_NAME, TRANSMISSION_MESH_EXECUTOR_IDS,
    TRANSMISSION_SCENE_COPY_EXECUTOR_IDS,
};
pub use ambient_occlusion::{
    AmbientOcclusionDepthConvention, AmbientOcclusionInputQualification,
    AmbientOcclusionInputSemantic, AmbientOcclusionMethod, AmbientOcclusionOutputs,
    AmbientOcclusionProjectionClass, AmbientOcclusionRenderRectKey, AoHistoryKey,
    CompiledAoProfile, CompiledAoWorkPlan, QualifiedAmbientOcclusionInput,
    AO_PROFILE_COMPILER_VERSION, AO_SHADER_INTERFACE_VERSION, COMPILED_AO_PROFILE_VERSION,
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
    RendererDataDocument, RendererDataDocumentError, RendererFeatureDocument,
    RendererFeatureReferenceListKind, RENDERER_DATA_DOCUMENT_VERSION,
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
