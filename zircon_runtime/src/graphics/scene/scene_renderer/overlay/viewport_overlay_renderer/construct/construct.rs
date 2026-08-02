use std::sync::Arc;

use super::super::super::{
    BaseScenePass, GridPass, HandlePass, PreviewSkyPass, SceneGizmoPass, SelectionOutlinePass,
    ViewportIconSource, WireframePass,
};
use super::super::viewport_overlay_renderer::ViewportOverlayRenderer;
use super::create_grid_buffer::create_grid_buffer;
use super::create_line_pipeline::create_line_pipeline;
use super::create_sky_pipeline::create_sky_pipeline;
use crate::graphics::scene::scene_renderer::advanced_lighting::froxel::{
    volumetric_apply_bind_group_layout_entries, VolumetricApplyFallbackResources,
};

impl ViewportOverlayRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        scene_color_format: wgpu::TextureFormat,
        final_color_format: wgpu::TextureFormat,
        scene_layout: &wgpu::BindGroupLayout,
        texture_layout: &wgpu::BindGroupLayout,
        icon_source: Arc<dyn ViewportIconSource>,
        volumetric_enabled: bool,
    ) -> Self {
        let line_pipeline = create_line_pipeline(device, final_color_format, scene_layout);
        let sky_volumetric_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("zircon-sky-volumetric-layout"),
                entries: &volumetric_apply_bind_group_layout_entries(wgpu::ShaderStages::FRAGMENT),
            });
        let sky_pipeline = create_sky_pipeline(
            device,
            scene_color_format,
            scene_layout,
            &sky_volumetric_layout,
            volumetric_enabled,
        );
        let sky_volumetric_apply = VolumetricApplyFallbackResources::new(device, "zircon-sky");
        let (grid_vertex_buffer, grid_vertex_count) = create_grid_buffer(device);

        Self {
            preview_sky: PreviewSkyPass,
            base_scene: BaseScenePass,
            selection_outline: SelectionOutlinePass,
            wireframe: WireframePass,
            grid: GridPass,
            scene_gizmo: SceneGizmoPass::new(
                device,
                final_color_format,
                scene_layout,
                texture_layout,
                icon_source,
            ),
            handle: HandlePass,
            line_pipeline,
            sky_pipeline,
            sky_volumetric_layout,
            sky_volumetric_apply,
            grid_vertex_buffer,
            grid_vertex_count,
        }
    }
}
