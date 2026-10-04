mod admission;
mod async_compile;
mod compile_options;
mod compiled_graph_cache;
mod declarations;
mod pipeline_cache_gate;
mod render_pipeline_asset;
mod validation;

pub(crate) use admission::{PipelineAdmission, PipelineAdmissionReason, PipelineUnavailable};
pub(crate) use async_compile::{
    PipelineAsyncCompileError, PipelineAsyncCompiler, PipelineAsyncQueueResult,
};
pub(crate) use pipeline_cache_gate::RuntimePipelineCache;

pub(crate) use compiled_graph_cache::{
    extract_compile_fingerprint, CompiledGraphCache, CompiledGraphCacheKey,
    RenderGraphCompileCameraTargetFingerprint, RenderGraphCompileInputError,
    RenderGraphCompileTextureTargetFormat,
};
pub(crate) use declarations::{
    transmission_mesh_step_index, transmission_scene_copy_step_index,
    AdvancedLightingCompileInputs, CompiledHistoryEpiloguePlan, CompiledHistoryTextureSource,
    CompiledRenderPipelineParts, RenderGraphExecutionBatch, RenderGraphExecutionCursor,
    RenderGraphExecutionPass, RenderGraphExecutionPassMetadata,
    OUTPUT_TARGET_DIRECT_IMPORT_EXECUTOR_ID, OUTPUT_TARGET_DIRECT_IMPORT_PASS_NAME,
    OUTPUT_TARGET_TEXTURE_RESOURCE_NAME, OUTPUT_TARGET_WRITEBACK_EXECUTOR_ID,
    OUTPUT_TARGET_WRITEBACK_PASS_NAME, SURFACE_PRESENT_EXECUTOR_ID, SURFACE_PRESENT_PASS_NAME,
    TRANSMISSION_MESH_EXECUTOR_IDS, TRANSMISSION_SCENE_COPY_EXECUTOR_IDS,
};
pub use declarations::{
    AmbientOcclusionDepthConvention, AmbientOcclusionInputQualification,
    AmbientOcclusionInputSemantic, AmbientOcclusionMethod, AmbientOcclusionOutputs,
    AmbientOcclusionProjectionClass, AmbientOcclusionRenderRectKey, AoHistoryKey,
    CompiledAoProfile, CompiledAoWorkPlan, CompiledRenderPipeline, QualifiedAmbientOcclusionInput,
    RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions,
    RenderPipelineCompileReport, RendererAsset, RendererDataDocument, RendererDataDocumentError,
    RendererFeatureAsset, RendererFeatureAssetReferences, RendererFeatureContractDiagnostic,
    RendererFeatureContractDiagnosticSeverity, RendererFeatureDocument,
    RendererFeatureReferenceListKind, RendererFeatureSource, AO_PROFILE_COMPILER_VERSION,
    AO_SHADER_INTERFACE_VERSION, COMPILED_AO_PROFILE_VERSION, RENDERER_DATA_DOCUMENT_VERSION,
};
pub use render_pipeline_asset::RenderPipelineAssetContext;
