use crate::graphics::backend::OffscreenTarget;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::post_process::SceneRuntimeFeatureFlags;
use crate::graphics::scene::scene_renderer::{HybridGiGpuPendingReadback, VirtualGeometryGpuPendingReadback};
use crate::graphics::types::{EditorOrRuntimeFrame, GraphicsError};

use super::super::super::scene_renderer_core::SceneRendererCore;
use super::build_compiled_scene_draws::build_compiled_scene_draws;
use super::partition_mesh_draws::partition_mesh_draws;
use super::prepare_overlay_buffers::prepare_overlay_buffers;
use super::virtual_geometry_indirect_stats::virtual_geometry_indirect_stats;

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
            u32,
            u32,
            u32,
            Vec<(u64, u32)>,
            Vec<(u64, u32, u64, usize)>,
            Vec<(u64, u32, u32, u32, usize)>,
            Option<std::sync::Arc<wgpu::Buffer>>,
            u32,
            Option<std::sync::Arc<wgpu::Buffer>>,
            Option<std::sync::Arc<wgpu::Buffer>>,
            Option<std::sync::Arc<wgpu::Buffer>>,
            Option<std::sync::Arc<wgpu::Buffer>>,
            Option<std::sync::Arc<wgpu::Buffer>>,
        ),
        GraphicsError,
    > {
        self.write_scene_uniform(queue, frame);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-compiled-scene-encoder"),
        });
        let compiled_scene_draws = build_compiled_scene_draws(
            self,
            device,
            &mut encoder,
            streamer,
            frame,
            runtime_features.virtual_geometry_enabled,
        );
        let (opaque_mesh_draws, transparent_mesh_draws) =
            partition_mesh_draws(&compiled_scene_draws.draws);
        let execution_draws = if runtime_features.deferred_lighting_enabled {
            opaque_mesh_draws
                .iter()
                .copied()
                .chain(transparent_mesh_draws.iter().copied())
                .collect::<Vec<_>>()
        } else {
            compiled_scene_draws.draws.iter().collect::<Vec<_>>()
        };
        let indirect_stats = virtual_geometry_indirect_stats(
            device,
            &execution_draws,
            compiled_scene_draws.indirect_args_count,
            compiled_scene_draws.indirect_segment_count,
            compiled_scene_draws.indirect_submission_buffer.clone(),
            compiled_scene_draws.indirect_draw_ref_buffer.clone(),
            compiled_scene_draws.indirect_segment_buffer.clone(),
        );
        let prepared_overlays = prepare_overlay_buffers(self, device, queue, streamer, frame)?;

        let (hybrid_gi_gpu_readback, virtual_geometry_gpu_readback) =
            self.execute_runtime_prepare_passes(device, queue, &mut encoder, frame)?;
        self.render_scene_passes(
            device,
            &mut encoder,
            streamer,
            frame,
            target,
            runtime_features,
            &compiled_scene_draws.draws,
            &opaque_mesh_draws,
            &transparent_mesh_draws,
        );
        self.execute_post_process_stack(
            device,
            queue,
            &mut encoder,
            target,
            frame,
            runtime_features,
            history_textures.as_deref(),
            history_available,
        );
        self.copy_history_textures(&mut encoder, target, history_textures, runtime_features);
        self.overlay_renderer.record_overlays(
            &mut encoder,
            &target.final_color_view,
            &target.depth_view,
            &self.scene_bind_group,
            frame,
            &prepared_overlays,
        );
        self.screen_space_ui_renderer
            .record(device, &mut encoder, &target.final_color_view, frame);

        queue.submit([encoder.finish()]);
        Ok((
            hybrid_gi_gpu_readback,
            virtual_geometry_gpu_readback,
            indirect_stats.draw_count,
            indirect_stats.buffer_count,
            indirect_stats.segment_count,
            indirect_stats.draw_submission_order,
            indirect_stats.draw_submission_records,
            indirect_stats.draw_submission_token_records,
            indirect_stats.args_buffer,
            indirect_stats.args_count,
            indirect_stats.submission_buffer,
            indirect_stats.draw_ref_buffer,
            indirect_stats.segment_buffer,
            indirect_stats.execution_buffer,
            indirect_stats.execution_records_buffer,
        ))
    }
}
