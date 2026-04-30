use crate::core::framework::render::{
    RenderVirtualGeometryCullInputSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
};
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::post_process::SceneRuntimeFeatureFlags;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

use super::super::super::scene_renderer_core::SceneRendererCore;
use super::super::SceneRendererCompiledSceneOutputs;
use super::assign_execution_owned_indirect_args::assign_execution_owned_indirect_args;
use super::build_compiled_scene_draws::build_compiled_scene_draws;
use super::partition_mesh_draws::partition_mesh_draws;
use super::prepare_overlay_buffers::prepare_overlay_buffers;
use super::virtual_geometry_indirect_stats::collect_virtual_geometry_indirect_stats;

impl SceneRendererCore {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_compiled_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        virtual_geometry_cull_input: Option<&RenderVirtualGeometryCullInputSnapshot>,
        previous_node_and_cluster_cull_global_state: Option<
            &RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
        >,
        target: &mut OffscreenTarget,
        runtime_features: SceneRuntimeFeatureFlags,
        history_textures: Option<&mut SceneFrameHistoryTextures>,
        history_available: bool,
    ) -> Result<SceneRendererCompiledSceneOutputs, GraphicsError> {
        self.write_scene_uniform(queue, frame);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-compiled-scene-encoder"),
        });
        let mut compiled_scene_draws = build_compiled_scene_draws(
            &self.advanced_plugin_resources,
            device,
            &mut encoder,
            &self.model_bind_group_layout,
            streamer,
            frame,
            runtime_features.virtual_geometry_enabled,
        );
        let _execution_args_buffer = assign_execution_owned_indirect_args(
            device,
            &mut encoder,
            compiled_scene_draws.draws_mut(),
            runtime_features.deferred_lighting_enabled,
        );
        let (opaque_mesh_draws, transparent_mesh_draws) =
            partition_mesh_draws(compiled_scene_draws.draws());
        let execution_draws = if runtime_features.deferred_lighting_enabled {
            opaque_mesh_draws
                .iter()
                .copied()
                .chain(transparent_mesh_draws.iter().copied())
                .collect::<Vec<_>>()
        } else {
            compiled_scene_draws.draws().iter().collect::<Vec<_>>()
        };
        let resolved_virtual_geometry_cluster_selections =
            frame.resolved_virtual_geometry_cluster_selections();
        let indirect_stats = collect_virtual_geometry_indirect_stats(
            &self.advanced_plugin_resources,
            device,
            &mut encoder,
            runtime_features.virtual_geometry_enabled,
            frame,
            virtual_geometry_cull_input,
            previous_node_and_cluster_cull_global_state,
            resolved_virtual_geometry_cluster_selections.as_deref(),
            &execution_draws,
            compiled_scene_draws.indirect_args_buffer(),
            compiled_scene_draws.indirect_args_count(),
            compiled_scene_draws.indirect_segment_count(),
            compiled_scene_draws.indirect_submission_buffer(),
            compiled_scene_draws.indirect_authority_buffer(),
            compiled_scene_draws.indirect_draw_ref_buffer(),
            compiled_scene_draws.indirect_segment_buffer(),
        );
        let prepared_overlays = prepare_overlay_buffers(self, device, queue, streamer, frame)?;

        let advanced_plugin_readbacks =
            self.execute_runtime_prepare_passes(device, queue, &mut encoder, streamer, frame)?;
        let hybrid_gi_scene_prepare_resources =
            advanced_plugin_readbacks.hybrid_gi_scene_prepare_resources();
        self.render_scene_passes(
            device,
            &mut encoder,
            streamer,
            frame,
            target,
            runtime_features,
            compiled_scene_draws.draws(),
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
            hybrid_gi_scene_prepare_resources,
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
        self.screen_space_ui_renderer.record(
            device,
            queue,
            &mut encoder,
            &target.final_color_view,
            frame,
        );

        queue.submit([encoder.finish()]);
        Ok(SceneRendererCompiledSceneOutputs::new(
            advanced_plugin_readbacks,
            indirect_stats,
        ))
    }
}
