use std::collections::HashSet;
use std::sync::Arc;

use crate::backend::OffscreenTarget;
use crate::scene::resources::ResourceStreamer;
use crate::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::scene::scene_renderer::{HybridGiGpuPendingReadback, VirtualGeometryGpuPendingReadback};
use crate::types::{EditorOrRuntimeFrame, GraphicsError};

use super::super::super::mesh::build_mesh_draws;
use super::super::super::post_process::SceneRuntimeFeatureFlags;
use super::super::scene_renderer_core::SceneRendererCore;

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
            Option<Arc<wgpu::Buffer>>,
            u32,
        ),
        GraphicsError,
    > {
        self.write_scene_uniform(queue, frame);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-compiled-scene-encoder"),
        });
        let built_mesh_draws = build_mesh_draws(
            device,
            &mut encoder,
            &self.virtual_geometry_indirect_args,
            &self.model_bind_group_layout,
            streamer,
            frame,
            runtime_features.virtual_geometry_enabled,
        );
        let virtual_geometry_indirect_segment_count = built_mesh_draws.indirect_segment_count;
        let mesh_draws = built_mesh_draws.draws;
        let virtual_geometry_indirect_draw_count = mesh_draws
            .iter()
            .filter(|draw| draw.uses_indirect_draw())
            .count() as u32;
        let virtual_geometry_indirect_buffer_count = mesh_draws
            .iter()
            .filter_map(|draw| {
                draw.indirect_args_buffer
                    .as_ref()
                    .map(|buffer| Arc::as_ptr(buffer) as usize)
            })
            .collect::<HashSet<_>>()
            .len() as u32;
        let virtual_geometry_indirect_args_buffer = mesh_draws
            .iter()
            .find_map(|draw| draw.indirect_args_buffer.as_ref().map(Arc::clone));
        let (opaque_mesh_draws, transparent_mesh_draws): (Vec<_>, Vec<_>) =
            mesh_draws.iter().partition(|draw| !draw.is_transparent());
        let prepared_overlays = self.overlay_renderer.prepare_buffers(
            device,
            queue,
            &self.texture_bind_group_layout,
            streamer,
            frame,
        )?;

        let (hybrid_gi_gpu_readback, virtual_geometry_gpu_readback) =
            self.execute_runtime_prepare_passes(device, queue, &mut encoder, frame)?;
        self.render_scene_passes(
            device,
            &mut encoder,
            streamer,
            frame,
            target,
            runtime_features,
            &mesh_draws,
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

        queue.submit([encoder.finish()]);
        Ok((
            hybrid_gi_gpu_readback,
            virtual_geometry_gpu_readback,
            virtual_geometry_indirect_draw_count,
            virtual_geometry_indirect_buffer_count,
            virtual_geometry_indirect_segment_count,
            virtual_geometry_indirect_args_buffer,
            virtual_geometry_indirect_draw_count,
        ))
    }
}
