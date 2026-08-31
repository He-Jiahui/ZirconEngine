use crate::core::framework::render::RenderPluginRendererOutputs;
use crate::graphics::scene::gpu_scene::GpuScenePreparedUpload;
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionRecord;
use crate::graphics::scene::scene_renderer::hzb::HzbOcclusionParamsCommit;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshIndirectWorkspacePreparedUpload;
use crate::graphics::scene::scene_renderer::mesh::{
    MeshDrawReplayStatsAccumulator, PreparedMeshQueueStats,
};
use crate::graphics::scene::scene_renderer::shadow::atlas::ShadowAtlasPreparedUpload;
use crate::graphics::scene::scene_renderer::sprite::PreparedSpriteQueueStats;
use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiPreparedUpload;
use crate::rhi::SubmissionTicket;

use super::super::super::scene_renderer_core::{
    SceneRendererAdvancedPluginReadbacks, SceneRendererCore, merge_plugin_renderer_outputs,
};
use super::super::SceneRendererCompiledSceneOutputs;
use super::build_compiled_scene_draws::CompiledSceneDraws;
use super::final_target_output::FinalTargetOutputSelection;

pub(super) struct CompiledSceneFrameSuccessContext {
    pub(super) hzb_occlusion_params_commits: Vec<HzbOcclusionParamsCommit>,
    pub(super) screen_space_ui_upload_commits: Vec<ScreenSpaceUiPreparedUpload>,
    pub(super) mesh_indirect_prepared_upload: MeshIndirectWorkspacePreparedUpload,
    pub(super) shadow_atlas_prepared_upload: ShadowAtlasPreparedUpload,
    pub(super) gpu_scene_prepared_upload: GpuScenePreparedUpload,
    pub(super) advanced_plugin_readbacks: SceneRendererAdvancedPluginReadbacks,
    pub(super) graph_plugin_outputs: RenderPluginRendererOutputs,
    pub(super) graph_execution_record: RenderGraphExecutionRecord,
    pub(super) prepared_mesh_queue_stats: PreparedMeshQueueStats,
    pub(super) prepared_sprite_queue_stats: PreparedSpriteQueueStats,
    pub(super) mesh_draw_replay_stats: MeshDrawReplayStatsAccumulator,
    pub(super) compiled_scene_draws: CompiledSceneDraws,
    pub(super) final_target_output: FinalTargetOutputSelection,
    pub(super) scene_submission: SubmissionTicket,
}

impl SceneRendererCore {
    pub(super) fn commit_compiled_scene_frame_success(
        &mut self,
        ctx: CompiledSceneFrameSuccessContext,
    ) -> SceneRendererCompiledSceneOutputs {
        let CompiledSceneFrameSuccessContext {
            hzb_occlusion_params_commits,
            screen_space_ui_upload_commits,
            mesh_indirect_prepared_upload,
            shadow_atlas_prepared_upload,
            gpu_scene_prepared_upload,
            mut advanced_plugin_readbacks,
            graph_plugin_outputs,
            graph_execution_record,
            prepared_mesh_queue_stats,
            prepared_sprite_queue_stats,
            mesh_draw_replay_stats,
            compiled_scene_draws,
            final_target_output,
            scene_submission,
        } = ctx;

        self.overlay_renderer.commit_pending_icon_uploads();
        if let Some(culler) = self.hzb_occlusion_culler.as_ref() {
            let expected_commit_count = hzb_occlusion_params_commits.len();
            let committed_count = culler.commit_params_uploads(hzb_occlusion_params_commits);
            debug_assert_eq!(committed_count as usize, expected_commit_count);
        } else {
            debug_assert!(hzb_occlusion_params_commits.is_empty());
        }
        if let Some(renderer) = self.screen_space_ui_renderer.as_mut() {
            for prepared in screen_space_ui_upload_commits {
                let committed = renderer.commit_prepared_upload(prepared);
                debug_assert!(committed);
            }
        } else {
            debug_assert!(screen_space_ui_upload_commits.is_empty());
        }
        let expected_mesh_indirect_commit_count = mesh_indirect_prepared_upload.commit_count();
        let mesh_indirect_commit_count =
            mesh_indirect_prepared_upload.commit(&mut self.mesh_indirect_draw_workspace);
        debug_assert_eq!(
            mesh_indirect_commit_count as usize,
            expected_mesh_indirect_commit_count
        );
        let _shadow_atlas_upload_report =
            shadow_atlas_prepared_upload.commit(&mut self.shadow_atlas_resources);
        gpu_scene_prepared_upload.commit(&mut self.gpu_scene);
        advanced_plugin_readbacks.commit_runtime_prepare_frame_transactions();
        let mut renderer_outputs = advanced_plugin_readbacks.into_outputs();
        merge_plugin_renderer_outputs(&mut renderer_outputs, graph_plugin_outputs);
        let prepared_mesh_queue_stats =
            prepared_mesh_queue_stats.with_mesh_draw_replay_stats(mesh_draw_replay_stats.stats());
        let prepared_mesh_queue_stats = prepared_mesh_queue_stats.with_gpu_scene_stats(
            self.gpu_scene.stats(),
            compiled_scene_draws.gpu_scene_upload_report(),
        );
        let prepared_mesh_queue_stats = prepared_mesh_queue_stats
            .with_virtual_geometry_execution_stats(
                compiled_scene_draws.virtual_geometry_execution_stats(),
            )
            .with_virtual_geometry_indirect_stats(
                compiled_scene_draws.virtual_geometry_indirect_stats(),
            );
        let outputs = SceneRendererCompiledSceneOutputs::new(
            SceneRendererAdvancedPluginReadbacks::from_outputs(renderer_outputs),
            graph_execution_record,
            prepared_mesh_queue_stats,
            prepared_sprite_queue_stats,
            scene_submission,
        );
        let _prev_transform_roll_report = self.gpu_scene.roll_prev_transforms_after_success();
        let _prev_skinned_palette_roll_report =
            self.gpu_scene.roll_prev_skinned_palettes_after_success();
        let _prev_skinned_source_roll_report =
            self.gpu_scene.roll_prev_skinned_gpu_sources_after_success();
        let _prev_morph_weights_roll_report =
            self.gpu_scene.roll_prev_morph_weights_after_success();
        self.mesh_pipelines
            .emit_forward_receiver_binding_profile_frame();
        self.mesh_pipelines.light_cookies.emit_profile_frame();
        match final_target_output.graph_import_report {
            Some(report) => outputs.with_output_target_graph_import_report(report),
            None => outputs,
        }
    }
}
