use std::sync::Arc;

use super::super::super::{
    BaseScenePass, GridPass, HandlePass, PreviewSkyPass, SceneGizmoPass, SelectionOutlinePass,
    ViewportIconSource, WireframePass,
};
use super::super::viewport_overlay_renderer::ViewportOverlayRenderer;
use super::create_grid_buffer::create_grid_buffer;
use super::create_line_pipeline::create_line_pipeline;
use super::create_sky_pipeline::create_sky_pipeline;

impl ViewportOverlayRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        scene_layout: &wgpu::BindGroupLayout,
        texture_layout: &wgpu::BindGroupLayout,
        icon_source: Arc<dyn ViewportIconSource>,
    ) -> Self {
        let line_pipeline = create_line_pipeline(device, target_format, scene_layout);
        let sky_pipeline = create_sky_pipeline(device, target_format, scene_layout);
        let (grid_vertex_buffer, grid_vertex_count) = create_grid_buffer(device);

        Self {
            preview_sky: PreviewSkyPass,
            base_scene: BaseScenePass,
            selection_outline: SelectionOutlinePass,
            wireframe: WireframePass,
            grid: GridPass,
            scene_gizmo: SceneGizmoPass::new(
                device,
                target_format,
                scene_layout,
                texture_layout,
                icon_source,
            ),
            handle: HandlePass,
            line_pipeline,
            sky_pipeline,
            grid_vertex_buffer,
            grid_vertex_count,
        }
    }
}
