use crate::core::framework::render::SkyboxMode;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::render_graph::RenderGraphAttachmentOps;

use super::super::super::super::resources::ResourceStreamer;
use super::super::scene_renderer_core::SceneRendererCore;

impl SceneRendererCore {
    pub(crate) fn render_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        scene_color_view: &wgpu::TextureView,
        final_color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
    ) -> Result<(), GraphicsError> {
        let realtime_ibl_prepared = matches!(
            frame.environment().skybox.mode,
            SkyboxMode::ProceduralGradient
        )
        .then_some(frame.environment().skybox.procedural)
        .filter(|sky| sky.intensity > 0.0)
        .map(|sky| self.realtime_ibl.prepare_frame(device, sky));
        self.write_scene_uniform(
            device,
            queue,
            streamer,
            frame,
            realtime_ibl_prepared.as_ref(),
            true,
        )?;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-scene-encoder"),
        });
        let realtime_ibl_submission = realtime_ibl_prepared
            .as_ref()
            .map(|prepared| {
                self.realtime_ibl.record_prepared_frame(
                    device,
                    &mut encoder,
                    false,
                    prepared,
                    &mut self.ibl_bake_pipeline_cache,
                )
            })
            .transpose()
            .map_err(GraphicsError::Asset)?
            .flatten();
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
        self.overlay_renderer.record_scene_content(
            &mut encoder,
            device,
            scene_color_view,
            depth_view,
            &self.scene_bind_group,
            &mesh_draws,
            Some(gpu_scene_bind_handle),
            &mut self.mesh_pipelines,
            streamer,
            frame,
            Some(&self.shadow_atlas_resources),
        );
        self.post_process.execute_output_transfer(
            device,
            &mut encoder,
            scene_color_view,
            final_color_view,
            RenderGraphAttachmentOps::clear_store(),
            frame.render_region(),
        );
        self.overlay_renderer.record_overlays(
            &mut encoder,
            final_color_view,
            depth_view,
            &self.scene_bind_group,
            frame,
            &prepared_overlays,
            frame.render_region(),
        );
        if let Some(screen_space_ui_renderer) = self.screen_space_ui_renderer.as_mut() {
            screen_space_ui_renderer.record(
                device,
                queue,
                &mut encoder,
                final_color_view,
                frame,
                RenderGraphAttachmentOps::load_store(),
                Some(streamer),
            )?;
        }
        self.readback_frame_index = self.readback_frame_index.wrapping_add(1);
        let readback_frame_index = self.readback_frame_index;
        let readback_ready = self
            .readback_queue
            .prepare_frame(device, readback_frame_index)
            .is_ok();
        if readback_ready {
            if let Some(submission) = realtime_ibl_submission.as_ref() {
                self.realtime_ibl.request_gpu_timestamp_readback(
                    submission,
                    queue.get_timestamp_period(),
                    &mut self.readback_queue,
                );
            }
            if let Err(error) = self
                .readback_queue
                .encode_copies(&mut encoder, readback_frame_index)
            {
                self.readback_queue.abort_frame(readback_frame_index);
                return Err(GraphicsError::BufferMap(error.to_string()));
            }
        }
        queue.submit([encoder.finish()]);
        let readback_map_error = if readback_ready {
            match self.readback_queue.begin_map(readback_frame_index) {
                Ok(()) => None,
                Err(error) => {
                    self.readback_queue.abort_frame(readback_frame_index);
                    Some(error)
                }
            }
        } else {
            None
        };
        if let Some(submission) = realtime_ibl_submission {
            self.realtime_ibl.complete_submission(submission, true);
        }
        let _prev_transform_roll_report = self.gpu_scene.roll_prev_transforms_after_success();
        let _prev_skinned_palette_roll_report =
            self.gpu_scene.roll_prev_skinned_palettes_after_success();
        let _prev_skinned_source_roll_report =
            self.gpu_scene.roll_prev_skinned_gpu_sources_after_success();
        let _prev_morph_weights_roll_report =
            self.gpu_scene.roll_prev_morph_weights_after_success();
        if let Some(error) = readback_map_error {
            return Err(GraphicsError::BufferMap(error.to_string()));
        }
        Ok(())
    }
}
