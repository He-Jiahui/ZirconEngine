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
    CompiledGraphCache, CompiledGraphCacheKey, RenderGraphCompileCameraTargetFingerprint,
    RenderGraphCompileInputError, RenderGraphCompileTextureTargetFormat,
    extract_compile_fingerprint,
};
pub use declarations::{
    AO_PROFILE_COMPILER_VERSION, AO_SHADER_INTERFACE_VERSION, AmbientOcclusionDepthConvention,
    AmbientOcclusionInputQualification, AmbientOcclusionInputSemantic, AmbientOcclusionMethod,
    AmbientOcclusionOutputs, AmbientOcclusionProjectionClass, AmbientOcclusionRenderRectKey,
    AoHistoryKey, COMPILED_AO_PROFILE_VERSION, CompiledAoProfile, CompiledAoWorkPlan,
    CompiledRenderPipeline, QualifiedAmbientOcclusionInput, RENDERER_DATA_DOCUMENT_VERSION,
    RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions,
    RenderPipelineCompileReport, RendererAsset, RendererDataDocument, RendererDataDocumentError,
    RendererFeatureAsset, RendererFeatureAssetReferences, RendererFeatureContractDiagnostic,
    RendererFeatureContractDiagnosticSeverity, RendererFeatureDocument,
    RendererFeatureReferenceListKind, RendererFeatureSource,
};
pub(crate) use declarations::{
    AdvancedLightingCompileInputs, CompiledHistoryEpiloguePlan, CompiledHistoryTextureSource,
    CompiledRenderPipelineParts, OUTPUT_TARGET_DIRECT_IMPORT_EXECUTOR_ID,
    OUTPUT_TARGET_DIRECT_IMPORT_PASS_NAME, OUTPUT_TARGET_TEXTURE_RESOURCE_NAME,
    OUTPUT_TARGET_WRITEBACK_EXECUTOR_ID, OUTPUT_TARGET_WRITEBACK_PASS_NAME,
    RenderGraphExecutionBatch, RenderGraphExecutionCursor, RenderGraphExecutionPass,
    RenderGraphExecutionPassMetadata, SURFACE_PRESENT_EXECUTOR_ID, SURFACE_PRESENT_PASS_NAME,
    TRANSMISSION_MESH_EXECUTOR_IDS, TRANSMISSION_SCENE_COPY_EXECUTOR_IDS,
    transmission_mesh_step_index, transmission_scene_copy_step_index,
};
pub use render_pipeline_asset::RenderPipelineAssetContext;
