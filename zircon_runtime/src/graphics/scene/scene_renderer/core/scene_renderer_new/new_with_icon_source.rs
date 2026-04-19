use std::collections::HashMap;
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;

use crate::graphics::types::GraphicsError;

use super::super::super::super::resources::ResourceStreamer;
use super::super::super::overlay::ViewportIconSource;
use super::super::constants::OFFSCREEN_FORMAT;
use super::super::scene_renderer::SceneRenderer;
use super::super::scene_renderer_core::SceneRendererCore;

impl SceneRenderer {
    pub fn new_with_icon_source(
        asset_manager: Arc<ProjectAssetManager>,
        icon_source: Arc<dyn ViewportIconSource>,
    ) -> Result<Self, GraphicsError> {
        let backend = crate::graphics::backend::RenderBackend::new_offscreen()?;
        let core = SceneRendererCore::new_with_icon_source(
            &backend.device,
            &backend.queue,
            OFFSCREEN_FORMAT,
            icon_source,
        );
        let streamer = ResourceStreamer::new(
            asset_manager,
            &backend.device,
            &backend.queue,
            &core.texture_bind_group_layout,
        );

        Ok(Self {
            backend,
            core,
            streamer,
            target: None,
            history_targets: HashMap::new(),
            generation: 0,
            last_hybrid_gi_gpu_readback: None,
            last_virtual_geometry_gpu_readback: None,
            last_virtual_geometry_indirect_draw_count: 0,
            last_virtual_geometry_indirect_buffer_count: 0,
            last_virtual_geometry_indirect_segment_count: 0,
            last_virtual_geometry_mesh_draw_submission_order: Vec::new(),
            last_virtual_geometry_mesh_draw_submission_records: Vec::new(),
            last_virtual_geometry_mesh_draw_submission_token_records: Vec::new(),
            last_virtual_geometry_indirect_args_buffer: None,
            last_virtual_geometry_indirect_args_count: 0,
            last_virtual_geometry_indirect_submission_buffer: None,
            last_virtual_geometry_indirect_draw_refs_buffer: None,
            last_virtual_geometry_indirect_segments_buffer: None,
            last_virtual_geometry_indirect_execution_buffer: None,
            last_virtual_geometry_indirect_execution_records_buffer: None,
        })
    }
}
