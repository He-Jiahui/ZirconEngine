//! Runtime absorption layer for the built-in high-level engine subsystems.

pub mod core;
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

/// Animation driver/manager registration (first-class runtime subsystem).
pub mod animation;
mod builtin;
pub mod extensions;
pub mod foundation;
pub mod input;
/// Physics backend/driver/manager registration (first-class runtime subsystem).
pub mod physics;
pub mod platform;
pub mod script;

pub use builtin::builtin_runtime_modules;
#[allow(unused_imports)]
pub(crate) use graphics::scene::{
    cluster_buffer_bytes_for_size, cluster_dimensions_for_size, create_depth_texture,
    GBUFFER_ALBEDO_FORMAT, NORMAL_FORMAT, OFFSCREEN_FORMAT,
};
#[allow(unused_imports)]
pub(crate) use graphics::{
    backend, extract, feature, material, pipeline, runtime, types, visibility,
    BuiltinRenderFeature, CompiledRenderPipeline, FrameHistoryAccess, FrameHistoryBinding,
    FrameHistoryHandle, FrameHistorySlot, GraphicsError, MaterialDomain, OfflineBakeOutput,
    OfflineBakeSettings, RenderFeature, RenderFeatureCapabilityRequirement,
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderFeatureResourceAccess,
    RenderFeatureResourceKind, RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions,
    RendererAsset, RendererFeatureAsset, SceneRenderer, ViewportFrame, ViewportFrameTextureHandle,
    VisibilityContext, VisibilityHistorySnapshot, VisibilityHybridGiFeedback,
    VisibilityHybridGiUpdatePlan, VisibilityVirtualGeometryCluster,
    VisibilityVirtualGeometryDrawSegment, VisibilityVirtualGeometryFeedback,
    VisibilityVirtualGeometryPageUploadPlan, WgpuRenderFramework,
};

#[cfg(test)]
mod tests;
