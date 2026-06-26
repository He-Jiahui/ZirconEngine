use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{
    AntiAliasSettings, PostProcessGraphResourceNames, RenderDepthOfFieldSettings,
    RenderFrameExtract, RenderMotionBlurSettings, RenderPipelineHandle,
    RenderPluginRendererOutputs, RenderPostProcessEffectStackSettings,
    RenderScreenSpaceReflectionSettings,
};
use crate::core::math::UVec2;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer;
use crate::graphics::ViewportRenderFrame;
use crate::graphics::{CompiledRenderPipeline, RenderPipelineAsset, RenderPipelineCompileOptions};
use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphAttachmentOps, RenderGraphBuilder,
    RenderGraphPassResourceAccess, RenderGraphResourceAccessKind, RenderGraphResourceKind,
    RenderPassId,
};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

use super::super::{
    RenderGraphExecutionResources, RenderPassExecutionContext, RenderPassExecutorId,
    RenderPassGpuExecutionContext,
};
use super::RenderPassExecutorRegistry;
use support::{
    execute_gpu_executor_without_specialized_context,
    execute_gpu_executor_without_specialized_context_for_extract, import_test_buffer,
    import_test_texture, test_extract, test_ui_extract, ContextMutatingExecutor,
};

#[path = "plugin_executor_policy.rs"]
mod plugin_executor_policy;
#[path = "support.rs"]
mod support;

mod postprocess_context_guards;
mod registry_contracts;
mod renderer_context_guards;
