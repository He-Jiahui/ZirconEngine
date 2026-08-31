use crate::core::framework::render::{CubemapFace, ShaderQualityTier};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    build_environment_capture_command_buffers, MeshDrawCommandStream,
    MeshDrawReplayStatsAccumulator, MeshSceneDataBindHandle,
};
use crate::graphics::scene::scene_renderer::mesh::{MeshDraw, MeshPipelineCache};
use crate::graphics::scene::scene_renderer::overlay::{BaseScenePass, ViewportOverlayRenderer};
use crate::render_graph::RenderGraphAttachmentOps;

use super::{
    EnvironmentCaptureGpuTarget, EnvironmentCaptureLightGridWorkspace,
    EnvironmentCaptureRenderPlan, EnvironmentCaptureSceneBatch,
    EnvironmentCaptureSceneUniformWorkspace,
};

enum EnvironmentCaptureForwardReceiverBindGroups {
    Shared(wgpu::BindGroup),
    PerFace([wgpu::BindGroup; 6]),
}

impl EnvironmentCaptureForwardReceiverBindGroups {
    fn bind_group(&self, face: CubemapFace) -> &wgpu::BindGroup {
        match self {
            Self::Shared(bind_group) => bind_group,
            Self::PerFace(bind_groups) => &bind_groups[face.index()],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics) struct EnvironmentCaptureWgpuRecordReport {
    pub face_pass_count: usize,
    pub command_build_count: usize,
    pub commands_per_face: usize,
    pub opaque_command_count: usize,
    pub alpha_mask_command_count: usize,
    pub advanced_pbr_opaque_command_count: usize,
    pub draw_call_count: u32,
    pub state_change_count: u32,
    pub bind_skip_count: u32,
}

impl EnvironmentCaptureWgpuRecordReport {
    fn emit_profile_counters(&self) {
        crate::profile_counter!(
            "render",
            "environment_capture_face_pass_count",
            self.face_pass_count
        );
        crate::profile_counter!(
            "render",
            "environment_capture_command_build_count",
            self.command_build_count
        );
        crate::profile_counter!(
            "render",
            "environment_capture_commands_per_face",
            self.commands_per_face
        );
        crate::profile_counter!(
            "render",
            "environment_capture_opaque_command_count",
            self.opaque_command_count
        );
        crate::profile_counter!(
            "render",
            "environment_capture_alpha_mask_command_count",
            self.alpha_mask_command_count
        );
        crate::profile_counter!(
            "render",
            "environment_capture_advanced_pbr_opaque_command_count",
            self.advanced_pbr_opaque_command_count
        );
        crate::profile_counter!(
            "render",
            "environment_capture_draw_call_count",
            self.draw_call_count
        );
        crate::profile_counter!(
            "render",
            "environment_capture_state_change_count",
            self.state_change_count
        );
        crate::profile_counter!(
            "render",
            "environment_capture_bind_skip_count",
            self.bind_skip_count
        );
    }
}

pub(in crate::graphics) struct EnvironmentCaptureWgpuRecorder;

impl EnvironmentCaptureWgpuRecorder {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics) fn record(
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        target: &EnvironmentCaptureGpuTarget,
        render_plan: &EnvironmentCaptureRenderPlan,
        scene_batch: &mut EnvironmentCaptureSceneBatch,
        uniform_workspace: &EnvironmentCaptureSceneUniformWorkspace,
        light_grid_workspace: Option<&EnvironmentCaptureLightGridWorkspace>,
        mesh_draws: &[MeshDraw],
        gpu_scene_bind_group: Option<MeshSceneDataBindHandle<'_>>,
        mesh_pipelines: &mut MeshPipelineCache,
        streamer: &ResourceStreamer,
        overlay_renderer: &mut ViewportOverlayRenderer,
        shader_quality: ShaderQualityTier,
    ) -> Result<EnvironmentCaptureWgpuRecordReport, String> {
        debug_assert_eq!(target.plan(), render_plan.target());
        let command_buffers =
            build_environment_capture_command_buffers(mesh_draws, mesh_pipelines, shader_quality);
        let command_stats = command_buffers.stats();
        let commands_per_face = command_stats
            .opaque_command_count
            .saturating_add(command_stats.alpha_mask_command_count)
            .saturating_add(command_stats.advanced_pbr_opaque_command_count);
        let capture_requires_forward_receiver = !mesh_pipelines
            .environment_only_pbr_base_profile_enabled()
            || [
                command_buffers.opaque().commands(),
                command_buffers.alpha_mask().commands(),
                command_buffers.advanced_pbr_opaque().commands(),
            ]
            .into_iter()
            .flatten()
            .any(|command| {
                mesh_pipelines.base_pipeline_requires_forward_receiver(command.pipeline_variant_id)
            });
        let capture_forward_receiver_bind_groups = create_capture_forward_receiver_bind_groups(
            capture_requires_forward_receiver,
            device,
            light_grid_workspace,
            mesh_pipelines,
        );
        let replay_stats = MeshDrawReplayStatsAccumulator::default();

        for capture_pass in render_plan.passes() {
            debug_assert!(capture_pass.opaque_only());
            let view = scene_batch.select_face(capture_pass.face());
            debug_assert_eq!(
                view.reverse_raster_winding(),
                capture_pass.reverse_raster_winding()
            );
            let scene_bind_group = uniform_workspace.bind_group(capture_pass.face());
            let color_view = target.color_face(capture_pass.face());

            overlay_renderer.record_preview_sky(
                encoder,
                device,
                color_view,
                target.depth_view(),
                scene_bind_group,
                view.frame(),
            );
            let streams = [
                MeshDrawCommandStream::new(command_buffers.opaque().commands(), None),
                MeshDrawCommandStream::new(command_buffers.alpha_mask().commands(), None),
                MeshDrawCommandStream::new(command_buffers.advanced_pbr_opaque().commands(), None),
            ];
            replay_stats.record(
                BaseScenePass.record_environment_capture_commands_with_attachment_ops(
                    encoder,
                    device,
                    color_view,
                    target.depth_view(),
                    scene_bind_group,
                    gpu_scene_bind_group,
                    streams,
                    mesh_pipelines,
                    streamer,
                    view.frame(),
                    view.frame().render_region(),
                    capture_forward_receiver_bind_groups
                        .as_ref()
                        .map(|bind_groups| bind_groups.bind_group(capture_pass.face())),
                    RenderGraphAttachmentOps::load_store(),
                    RenderGraphAttachmentOps::load_store(),
                ),
            );
        }

        let replay_stats = replay_stats.stats();
        let skipped_command_count = skipped_environment_capture_command_count(
            commands_per_face,
            render_plan.total_pass_count(),
            replay_stats.draw_call_count as usize,
        );
        if skipped_command_count != 0 {
            return Err(format!(
                "environment capture replay skipped {skipped_command_count} of {} opaque commands",
                commands_per_face.saturating_mul(render_plan.total_pass_count())
            ));
        }
        let report = EnvironmentCaptureWgpuRecordReport {
            face_pass_count: render_plan.total_pass_count(),
            command_build_count: 1,
            commands_per_face,
            opaque_command_count: command_stats.opaque_command_count,
            alpha_mask_command_count: command_stats.alpha_mask_command_count,
            advanced_pbr_opaque_command_count: command_stats.advanced_pbr_opaque_command_count,
            draw_call_count: replay_stats.draw_call_count,
            state_change_count: replay_stats.state_change_count,
            bind_skip_count: replay_stats.bind_skip_count,
        };
        report.emit_profile_counters();
        Ok(report)
    }
}

fn create_capture_forward_receiver_bind_groups(
    required: bool,
    device: &wgpu::Device,
    light_grid_workspace: Option<&EnvironmentCaptureLightGridWorkspace>,
    mesh_pipelines: &mut MeshPipelineCache,
) -> Option<EnvironmentCaptureForwardReceiverBindGroups> {
    if !required {
        return None;
    }
    let mut create_receiver = |face: Option<CubemapFace>| {
        let params = face
            .and_then(|face| light_grid_workspace.map(|workspace| workspace.params_binding(face)));
        let zbins = face
            .and_then(|face| light_grid_workspace.map(|workspace| workspace.zbins_binding(face)));
        let tile_masks = face.and_then(|face| {
            light_grid_workspace.map(|workspace| workspace.tile_masks_binding(face))
        });
        mesh_pipelines.create_environment_capture_forward_receiver_bind_group(
            device, params, zbins, tile_masks,
        )
    };

    Some(match light_grid_workspace {
        Some(_) => EnvironmentCaptureForwardReceiverBindGroups::PerFace(
            CubemapFace::ALL.map(|face| create_receiver(Some(face))),
        ),
        None => EnvironmentCaptureForwardReceiverBindGroups::Shared(create_receiver(None)),
    })
}

const fn skipped_environment_capture_command_count(
    commands_per_face: usize,
    face_pass_count: usize,
    replayed_draw_call_count: usize,
) -> usize {
    commands_per_face
        .saturating_mul(face_pass_count)
        .saturating_sub(replayed_draw_call_count)
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("environment_capture_wgpu_recorder.rs");

    fn production_source() -> &'static str {
        SOURCE
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("environment capture recorder must retain a test boundary")
    }

    #[test]
    fn recorder_builds_one_opaque_command_set_for_all_six_faces() {
        let source = production_source();

        assert_eq!(
            source
                .matches("build_environment_capture_command_buffers(")
                .count(),
            1
        );
        assert!(source.contains("for capture_pass in render_plan.passes()"));
        assert!(source.contains("command_buffers.opaque().commands()"));
        assert!(source.contains("command_buffers.alpha_mask().commands()"));
        assert!(source.contains("command_buffers.advanced_pbr_opaque().commands()"));
        assert_eq!(
            source
                .matches("create_environment_capture_forward_receiver_bind_group(")
                .count(),
            1
        );
        assert!(!source.contains("build_mesh_pass_command_buffers("));
        assert!(!source.contains("command_buffers.transparent().commands()"));
        assert!(!source.contains("command_buffers.transmission().commands()"));
    }

    #[test]
    fn recorder_selects_face_owned_bindings_and_attachments() {
        let source = production_source();

        assert!(source.contains("scene_batch.select_face(capture_pass.face())"));
        assert!(source.contains("uniform_workspace.bind_group(capture_pass.face())"));
        assert!(source.contains("light_grid_workspace"));
        assert!(source.contains("bind_group(capture_pass.face())"));
        assert!(source.contains("target.color_face(capture_pass.face())"));
        assert!(source.contains("target.depth_view()"));
        assert!(source.contains("record_preview_sky("));
        assert!(source.contains("record_environment_capture_commands_with_attachment_ops("));
        assert!(!source.contains("BaseScenePass.record_commands_with_attachment_ops("));
        assert!(source.contains("RenderGraphAttachmentOps::load_store()"));
    }

    #[test]
    fn recorder_reuses_one_disabled_receiver_or_builds_six_lit_receivers() {
        let source = production_source();

        assert!(source.contains("EnvironmentCaptureForwardReceiverBindGroups::Shared"));
        assert!(source.contains("EnvironmentCaptureForwardReceiverBindGroups::PerFace"));
        assert!(source.contains("CubemapFace::ALL.map"));
    }

    #[test]
    fn recorder_publishes_existing_structural_report_to_the_profiler() {
        let source = production_source();

        for counter in [
            "environment_capture_face_pass_count",
            "environment_capture_command_build_count",
            "environment_capture_commands_per_face",
            "environment_capture_draw_call_count",
            "environment_capture_state_change_count",
            "environment_capture_bind_skip_count",
        ] {
            assert!(
                source.contains(counter),
                "missing profile counter {counter}"
            );
        }
        assert!(source.contains("report.emit_profile_counters()"));
    }

    #[test]
    fn incomplete_face_replay_is_not_a_successful_capture() {
        assert_eq!(skipped_environment_capture_command_count(3, 6, 18), 0);
        assert_eq!(skipped_environment_capture_command_count(3, 6, 17), 1);
        assert_eq!(skipped_environment_capture_command_count(3, 6, 0), 18);
    }
}
