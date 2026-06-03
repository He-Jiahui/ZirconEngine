use std::collections::HashMap;

use crate::core::framework::render::FrameHistoryHandle;

use super::super::scene_renderer_core::SceneRendererCore;
use crate::graphics::backend::{OffscreenTarget, RenderBackend};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::mesh::PreparedMeshQueueStats;
use crate::graphics::scene::scene_renderer::sprite::PreparedSpriteQueueStats;

use super::super::super::graph_execution::{
    RenderGraphExecutionRecord, RenderPassExecutorRegistry,
};
use super::advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;

pub struct SceneRenderer {
    pub(in crate::graphics::scene::scene_renderer::core) backend: RenderBackend,
    pub(in crate::graphics::scene::scene_renderer::core) core: SceneRendererCore,
    pub(in crate::graphics::scene::scene_renderer::core) streamer: ResourceStreamer,
    pub(in crate::graphics::scene::scene_renderer::core) target: Option<OffscreenTarget>,
    pub(in crate::graphics::scene::scene_renderer::core) history_targets:
        HashMap<FrameHistoryHandle, SceneFrameHistoryTextures>,
    pub(in crate::graphics::scene::scene_renderer::core) generation: u64,
    pub(in crate::graphics::scene::scene_renderer::core) render_pass_executors:
        RenderPassExecutorRegistry,
    pub(in crate::graphics::scene::scene_renderer::core) last_render_graph_execution:
        RenderGraphExecutionRecord,
    pub(in crate::graphics::scene::scene_renderer::core) last_prepared_mesh_queue_stats:
        PreparedMeshQueueStats,
    pub(in crate::graphics::scene::scene_renderer::core) last_prepared_sprite_queue_stats:
        PreparedSpriteQueueStats,
    pub(in crate::graphics::scene::scene_renderer::core) advanced_plugin_outputs:
        SceneRendererAdvancedPluginOutputs,
}
