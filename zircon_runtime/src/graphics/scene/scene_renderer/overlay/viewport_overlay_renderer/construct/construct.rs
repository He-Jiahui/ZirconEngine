use std::sync::Arc;

use super::super::super::{
    BaseScenePass, GridPass, HandlePass, PreviewSkyPass, SceneGizmoPass, SelectionOutlinePass,
    ViewportIconSource, WireframePass,
};
use super::super::viewport_overlay_renderer::{
    ViewportInteractionOverlays, ViewportOverlayRenderer,
};
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
        interaction_overlays_enabled: bool,
    ) -> Self {
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
        let interaction_overlays = interaction_overlays_enabled.then(|| {
            let line_pipeline = create_line_pipeline(device, final_color_format, scene_layout);
            let (grid_vertex_buffer, grid_vertex_count) = create_grid_buffer(device);

            ViewportInteractionOverlays {
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
                grid_vertex_buffer,
                grid_vertex_count,
            }
        });

        Self {
            preview_sky: PreviewSkyPass,
            base_scene: BaseScenePass,
            interaction_overlays,
            sky_pipeline,
            sky_volumetric_layout,
            sky_volumetric_apply,
        }
    }
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("construct.rs");

    fn production_source() -> &'static str {
        SOURCE
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("overlay construction should retain a test-module boundary")
    }

    #[test]
    fn interaction_only_pipelines_are_constructed_lazily() {
        let source = production_source();
        let interaction_gate = source
            .find("interaction_overlays_enabled.then(|| {")
            .expect("interaction overlay resources should be conditionally constructed");

        for constructor in [
            "create_line_pipeline(device, final_color_format, scene_layout)",
            "create_grid_buffer(device)",
            "SceneGizmoPass::new(",
        ] {
            let position = source
                .find(constructor)
                .unwrap_or_else(|| panic!("missing interaction constructor `{constructor}`"));
            assert!(
                interaction_gate < position,
                "{constructor} must remain behind the interaction-overlay gate"
            );
        }
    }
}
