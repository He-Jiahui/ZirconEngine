//! Runtime absorption layer for the built-in high-level engine subsystems.

pub mod core;
pub mod engine_module;

// `ui` must be declared before `asset` (asset types reference UI template loaders).
pub mod asset;
pub mod scene;
pub mod ui;

pub use crate::core::resource;

pub mod rhi;
pub mod rhi_wgpu;
pub mod render_graph;
pub mod graphics;

mod builtin;
pub mod extensions;
/// Physics backend/driver/manager registration (first-class runtime subsystem).
pub mod physics;
/// Animation driver/manager registration (first-class runtime subsystem).
pub mod animation;
pub mod foundation;
pub mod input;
pub mod platform;
pub mod script;

pub use builtin::builtin_runtime_modules;
#[allow(unused_imports)]
pub(crate) use crate::core::framework;
#[allow(unused_imports)]
pub(crate) use crate::core::manager;
#[allow(unused_imports)]
pub(crate) use graphics::{
    backend, extract, feature, material, pipeline, runtime, types, visibility,
    BuiltinRenderFeature, CompiledRenderPipeline, EditorOrRuntimeFrame, FrameHistoryAccess,
    FrameHistoryBinding, FrameHistoryHandle, FrameHistorySlot, GraphicsError, MaterialDomain,
    OfflineBakeOutput, OfflineBakeSettings, RenderFeature, RenderFeatureDescriptor,
    RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions, RendererAsset,
    RendererFeatureAsset, SceneRenderer, ViewportFrame, ViewportFrameTextureHandle,
    ViewportIconSource, VisibilityContext, VisibilityHistorySnapshot,
    VisibilityHybridGiFeedback, VisibilityHybridGiUpdatePlan, VisibilityVirtualGeometryCluster,
    VisibilityVirtualGeometryDrawSegment, VisibilityVirtualGeometryFeedback,
    VisibilityVirtualGeometryPageUploadPlan, WgpuRenderFramework,
};
#[allow(unused_imports)]
pub(crate) use graphics::scene::{
    cluster_buffer_bytes_for_size, cluster_dimensions_for_size, create_depth_texture,
    GBUFFER_ALBEDO_FORMAT, NORMAL_FORMAT, OFFSCREEN_FORMAT,
};
pub(crate) use render_graph::{
    CompiledRenderGraph, CompiledRenderPass, ExternalResource, PassFlags, QueueLane,
    RenderGraphError, RenderPassId, TransientBuffer, TransientTexture,
};
pub(crate) use rhi::{RenderBackendCaps, RenderQueueClass};
pub(crate) use rhi_wgpu::wgpu_backend_caps;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use crate::engine_module::{
    dependency_on, driver_contract, factory, module_context, plugin_context, qualified_name,
    stub_driver_descriptor, stub_module_descriptor, stub_plugin_descriptor, EngineModule,
    EngineService,
};

#[cfg(test)]
mod tests;
