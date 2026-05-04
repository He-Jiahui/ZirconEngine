//! Runtime absorption layer for the built-in high-level engine subsystems.

pub mod core;
pub mod diagnostic_log;
pub mod dynamic_api;
pub mod engine_module;

// `ui` must be declared before `asset` (asset types reference UI template loaders).
pub mod asset;
pub mod scene;
pub mod ui;

pub use crate::core::resource;

pub mod graphics;
pub mod render_graph;
pub mod rhi;
pub mod rhi_wgpu;

mod builtin;
pub mod foundation;
pub mod input;
pub mod platform;
pub mod plugin;
pub mod script;

pub use builtin::{
    builtin_runtime_modules, default_manifest_for_target, manifest_with_mode_baseline,
    runtime_core_modules, runtime_modules_for_target,
    runtime_modules_for_target_with_linked_plugins,
    runtime_modules_for_target_with_plugin_and_feature_registration_reports,
    runtime_modules_for_target_with_plugin_registration_reports, RuntimeModuleLoadReport,
    RuntimePluginId, RuntimeRequiredPluginMissing, RuntimeTargetMode,
};
#[allow(unused_imports)]
pub(crate) use graphics::scene::{
    cluster_buffer_bytes_for_size, cluster_dimensions_for_size, create_depth_texture,
    GBUFFER_ALBEDO_FORMAT, NORMAL_FORMAT, OFFSCREEN_FORMAT,
};
#[allow(unused_imports)]
pub(crate) use graphics::{
    backend, extract, feature, material, pipeline, runtime, types, visibility,
    BuiltinRenderFeature, CompiledRenderPipeline, CompiledRenderPipelinePassStage,
    FrameHistoryAccess, FrameHistoryBinding, FrameHistoryHandle, FrameHistorySlot, GraphicsError,
    HybridGiGpuCompletion, HybridGiRuntimeFeedback, HybridGiRuntimePrepareInput,
    HybridGiRuntimePrepareOutput, HybridGiRuntimeProvider, HybridGiRuntimeProviderRegistration,
    HybridGiRuntimeState, HybridGiRuntimeStats, HybridGiRuntimeUpdate, MaterialDomain,
    OfflineBakeOutput, OfflineBakeSettings, ParticleGpuFeedback, ParticleRuntimeFeedback,
    RenderFeature, RenderFeatureCapabilityRequirement, RenderFeatureDescriptor,
    RenderFeaturePassDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceKind,
    RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions, RendererAsset,
    RendererFeatureAsset, SceneRenderer, ViewportFrame, ViewportFrameTextureHandle,
    VirtualGeometryGpuCompletion, VirtualGeometryRuntimeExtractOutput,
    VirtualGeometryRuntimeFeedback, VirtualGeometryRuntimePrepareInput,
    VirtualGeometryRuntimePrepareOutput, VirtualGeometryRuntimeProvider,
    VirtualGeometryRuntimeProviderRegistration, VirtualGeometryRuntimeState,
    VirtualGeometryRuntimeStats, VirtualGeometryRuntimeUpdate, VisibilityContext,
    VisibilityHistorySnapshot, VisibilityHybridGiFeedback, VisibilityHybridGiUpdatePlan,
    VisibilityVirtualGeometryCluster, VisibilityVirtualGeometryDrawSegment,
    VisibilityVirtualGeometryFeedback, VisibilityVirtualGeometryPageUploadPlan,
    WgpuRenderFramework,
};
#[cfg(test)]
mod tests;
