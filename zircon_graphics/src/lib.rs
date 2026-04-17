//! Core rendering, scene rasterization, and host-agnostic GPU services.

mod backend;
mod compat;
mod extract;
mod feature;
mod host;
mod material;
mod pipeline;
mod runtime;
mod scene;
mod service;
mod shader;
mod types;
mod visibility;

use zircon_module::{EngineModule, ModuleDescriptor};

pub use backend::RuntimePreviewRenderer;
pub use compat::{
    LegacyRenderService, LegacyRuntimePreviewRenderer, LegacySharedTextureRenderService,
};
pub use extract::{FrameHistoryAccess, FrameHistoryBinding, FrameHistoryHandle, FrameHistorySlot};
pub use feature::{BuiltinRenderFeature, RenderFeature, RenderFeatureDescriptor};
pub use host::{
    create_render_server, create_render_service, create_render_service_with_icon_source,
    create_runtime_preview_renderer, create_shared_texture_render_service,
    create_shared_texture_render_service_with_icon_source, module_descriptor, WgpuDriver,
    WgpuRenderingManager, GRAPHICS_MODULE_NAME, RENDERING_MANAGER_NAME, RENDER_SERVER_NAME,
    WGPU_DRIVER_NAME,
};
pub use material::MaterialDomain;
pub use pipeline::{
    CompiledRenderPipeline, RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions,
    RendererAsset, RendererFeatureAsset,
};
pub use runtime::{offline_bake_frame, OfflineBakeOutput, OfflineBakeSettings, WgpuRenderServer};
#[cfg(test)]
pub(crate) use scene::ViewportOverlayRenderer;
pub use scene::{SceneRenderer, ViewportIconSource};
pub use service::{RenderService, SharedTextureRenderService};
pub use shader::{MaterialGraphAsset, ShaderGraphAsset, ShaderProgramAsset, ShaderVariantKey};
pub use types::{
    EditorOrRuntimeFrame, GpuResourceHandle, GraphicsError, ViewportFrame,
    ViewportFrameTextureHandle,
};
pub use visibility::{
    VisibilityBatch, VisibilityBatchKey, VisibilityBounds, VisibilityBvhInstance,
    VisibilityBvhUpdatePlan, VisibilityBvhUpdateStrategy, VisibilityContext, VisibilityDrawCommand,
    VisibilityHistoryEntry, VisibilityHistorySnapshot, VisibilityHybridGiFeedback,
    VisibilityHybridGiProbe, VisibilityHybridGiUpdatePlan, VisibilityInstanceUploadPlan,
    VisibilityParticleUploadPlan, VisibilityVirtualGeometryCluster,
    VisibilityVirtualGeometryFeedback, VisibilityVirtualGeometryPageUploadPlan,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct GraphicsModule;

impl EngineModule for GraphicsModule {
    fn module_name(&self) -> &'static str {
        GRAPHICS_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Rendering device abstraction and scene rendering"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}

#[cfg(test)]
mod tests;
