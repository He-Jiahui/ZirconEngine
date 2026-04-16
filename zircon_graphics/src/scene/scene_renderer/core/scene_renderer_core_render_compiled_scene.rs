use crate::backend::OffscreenTarget;
use crate::scene::scene_renderer::{HybridGiGpuPendingReadback, VirtualGeometryGpuPendingReadback};
use crate::types::{EditorOrRuntimeFrame, GraphicsError};

use super::super::super::resources::ResourceStreamer;
use super::super::history::SceneFrameHistoryTextures;
use super::super::mesh::build_mesh_draws;
use super::super::post_process::SceneRuntimeFeatureFlags;
use super::scene_renderer_core::SceneRendererCore;
use super::target_extent::texture_extent;

impl SceneRendererCore {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_compiled_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        streamer: &ResourceStreamer,
        frame: &EditorOrRuntimeFrame,
        target: &mut OffscreenTarget,
        runtime_features: SceneRuntimeFeatureFlags,
        history_textures: Option<&mut SceneFrameHistoryTextures>,
        history_available: bool,
    ) -> Result<
        (
            Option<HybridGiGpuPendingReadback>,
            Option<VirtualGeometryGpuPendingReadback>,
        ),
        GraphicsError,
    > {
        self.write_scene_uniform(queue, frame);
        let mesh_draws = build_mesh_draws(
            device,
            &self.model_bind_group_layout,
            streamer,
            frame,
            runtime_features.virtual_geometry_enabled,
        );
        let (opaque_mesh_draws, transparent_mesh_draws): (Vec<_>, Vec<_>) =
            mesh_draws.iter().partition(|draw| !draw.is_transparent());
        let prepared_overlays = self.overlay_renderer.prepare_buffers(
            device,
            queue,
            &self.texture_bind_group_layout,
            streamer,
            frame,
        )?;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-compiled-scene-encoder"),
        });
        let hybrid_gi_gpu_readback = self.hybrid_gi.execute_prepare(
            device,
            queue,
            &mut encoder,
            frame.hybrid_gi_prepare.as_ref(),
            frame
                .extract
                .lighting
                .hybrid_global_illumination
                .as_ref()
                .map(|hybrid_gi| hybrid_gi.probe_budget),
            frame
                .extract
                .lighting
                .hybrid_global_illumination
                .as_ref()
                .map(|hybrid_gi| hybrid_gi.tracing_budget),
        )?;
        let virtual_geometry_gpu_readback = self.virtual_geometry.execute_prepare(
            device,
            queue,
            &mut encoder,
            frame.virtual_geometry_prepare.as_ref(),
            frame
                .extract
                .geometry
                .virtual_geometry
                .as_ref()
                .map(|virtual_geometry| virtual_geometry.page_budget),
        )?;
        if runtime_features.deferred_lighting_enabled {
            self.overlay_renderer.record_preview_sky(
                &mut encoder,
                &target.final_color_view,
                &target.depth_view,
                &self.scene_bind_group,
                frame,
            );
            self.normal_prepass.record(
                &mut encoder,
                &target.normal_view,
                &target.depth_view,
                &self.scene_bind_group,
                opaque_mesh_draws.iter().copied(),
            );
            self.deferred.record_gbuffer_geometry(
                &mut encoder,
                &target.gbuffer_albedo_view,
                &target.depth_view,
                &self.scene_bind_group,
                opaque_mesh_draws.iter().copied(),
            );
        } else {
            self.normal_prepass.record(
                &mut encoder,
                &target.normal_view,
                &target.depth_view,
                &self.scene_bind_group,
                mesh_draws.iter(),
            );
            self.overlay_renderer.record_scene_content(
                &mut encoder,
                device,
                &target.scene_color_view,
                &target.depth_view,
                &self.scene_bind_group,
                &mesh_draws,
                &mut self.mesh_pipelines,
                streamer,
                frame,
            );
            if runtime_features.particle_rendering_enabled {
                self.particle_renderer.record(
                    device,
                    &mut encoder,
                    &target.scene_color_view,
                    &target.depth_view,
                    &self.scene_bind_group,
                    frame,
                );
            }
        }
        self.post_process.execute_ssao(
            device,
            queue,
            &mut encoder,
            target.size,
            &target.depth_view,
            &target.normal_view,
            history_textures
                .as_ref()
                .map(|history| &history.ambient_occlusion_view),
            &target.ambient_occlusion_view,
            runtime_features.ssao_enabled,
            history_available,
        );
        self.post_process.execute_clustered_lighting(
            device,
            queue,
            &mut encoder,
            target.size,
            target.cluster_dimensions,
            &target.cluster_buffer,
            target.cluster_buffer_bytes,
            &frame.extract.lighting.directional_lights,
            runtime_features.clustered_lighting_enabled,
        );
        if runtime_features.deferred_lighting_enabled {
            self.deferred.execute_lighting(
                device,
                &mut encoder,
                &self.scene_bind_group,
                &target.gbuffer_albedo_view,
                &target.normal_view,
                &target.final_color_view,
                &target.scene_color_view,
            );
            self.overlay_renderer.record_meshes(
                &mut encoder,
                device,
                &target.scene_color_view,
                &target.depth_view,
                &self.scene_bind_group,
                transparent_mesh_draws.iter().copied(),
                &mut self.mesh_pipelines,
                streamer,
                frame,
            );
            if runtime_features.particle_rendering_enabled {
                self.particle_renderer.record(
                    device,
                    &mut encoder,
                    &target.scene_color_view,
                    &target.depth_view,
                    &self.scene_bind_group,
                    frame,
                );
            }
        }
        self.post_process.execute_bloom(
            device,
            queue,
            &mut encoder,
            target.size,
            &target.scene_color_view,
            &target.bloom_view,
            frame.extract.post_process.bloom,
            runtime_features.bloom_enabled,
        );
        self.post_process.execute_post_process(
            device,
            queue,
            &mut encoder,
            target.size,
            target.cluster_dimensions,
            &target.scene_color_view,
            &target.ambient_occlusion_view,
            history_textures
                .as_ref()
                .map(|history| &history.scene_color_view),
            &target.bloom_view,
            &target.final_color_view,
            &target.cluster_buffer,
            frame,
            runtime_features,
            history_available,
        );

        if let Some(history) = history_textures {
            if runtime_features.history_resolve_enabled {
                encoder.copy_texture_to_texture(
                    target.final_color.as_image_copy(),
                    history.scene_color.as_image_copy(),
                    texture_extent(target.size),
                );
            }
            if runtime_features.ssao_enabled {
                encoder.copy_texture_to_texture(
                    target.ambient_occlusion.as_image_copy(),
                    history.ambient_occlusion.as_image_copy(),
                    texture_extent(target.size),
                );
            }
        }

        self.overlay_renderer.record_overlays(
            &mut encoder,
            &target.final_color_view,
            &target.depth_view,
            &self.scene_bind_group,
            frame,
            &prepared_overlays,
        );

        queue.submit([encoder.finish()]);
        Ok((hybrid_gi_gpu_readback, virtual_geometry_gpu_readback))
    }
}
