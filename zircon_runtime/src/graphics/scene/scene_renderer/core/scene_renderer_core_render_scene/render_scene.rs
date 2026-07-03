use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

use super::super::super::super::resources::ResourceStreamer;
use super::super::scene_renderer_core::SceneRendererCore;

impl SceneRendererCore {
    pub(crate) fn render_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
    ) -> Result<(), GraphicsError> {
        self.write_scene_uniform(queue, frame);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-scene-encoder"),
        });
        let shadow_frame_plan =
            crate::graphics::scene::scene_renderer::shadow::build_shadow_frame_plan(
                &mut self.shadow_atlas_allocator,
                frame,
                self.shadow_atlas_resources.config(),
            );
        self.shadow_atlas_resources
            .upload_frame(
                queue,
                shadow_frame_plan.slots(),
                shadow_frame_plan.globals(),
            )
            .map_err(GraphicsError::Asset)?;
        let built_mesh_draws = self.advanced_plugin_resources.build_mesh_draws(
            device,
            queue,
            &mut encoder,
            &self.material_texture_bind_group_layout,
            &mut self.gpu_scene,
            streamer,
            frame,
            false,
            Some(shadow_frame_plan.light_slots()),
        );
        let mesh_draws = built_mesh_draws.into_draws();
        let gpu_scene_bind_group = self.gpu_scene.scene_bind_group().clone();
        let gpu_scene_bind_handle =
            crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshSceneDataBindHandle::new(
                &gpu_scene_bind_group,
            );
        let prepared_overlays = self.overlay_renderer.prepare_buffers(
            device,
            queue,
            &self.texture_bind_group_layout,
            streamer,
            frame,
        )?;
        self.overlay_renderer.record(
            &mut encoder,
            device,
            color_view,
            depth_view,
            &self.scene_bind_group,
            &mesh_draws,
            Some(gpu_scene_bind_handle),
            &mut self.mesh_pipelines,
            streamer,
            frame,
            &prepared_overlays,
            Some(&self.shadow_atlas_resources),
        );
        self.screen_space_ui_renderer.record(
            device,
            queue,
            &mut encoder,
            color_view,
            frame,
            crate::render_graph::RenderGraphAttachmentOps::load_store(),
        );
        queue.submit([encoder.finish()]);
        let _prev_transform_roll_report = self.gpu_scene.roll_prev_transforms_after_success();
        let _prev_skinned_palette_roll_report =
            self.gpu_scene.roll_prev_skinned_palettes_after_success();
        let _prev_skinned_source_roll_report =
            self.gpu_scene.roll_prev_skinned_gpu_sources_after_success();
        let _prev_morph_weights_roll_report =
            self.gpu_scene.roll_prev_morph_weights_after_success();
        Ok(())
    }
}
