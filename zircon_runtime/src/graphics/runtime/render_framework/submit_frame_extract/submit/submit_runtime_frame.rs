use crate::core::framework::render::{RenderFrameworkError, RenderViewportHandle};

use crate::graphics::types::ViewportRenderFrame;

use super::super::super::graphics_debugger_capture::{
    begin_graphics_debugger_capture, fail_pending_graphics_debugger_capture,
    finish_active_capture_and_relock,
};
use super::super::super::render_framework_backend_error::render_framework_backend_error;
use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::super::build_frame_submission_context::build_frame_submission_context;
use super::super::prepare_runtime_submission::prepare_runtime_submission;
use super::super::record_submission::record_submission;
use super::super::update_stats::update_stats;
use super::super::viewport_generation_guard::validate_viewport_generation;
use super::collect_runtime_feedback::collect_runtime_feedback;
use super::release_previous_history::release_previous_history;
use super::resolve_history_handle::resolve_history_handle;
use super::update_particle_previous_state::update_particle_previous_state_after_success;
use super::update_temporal_camera_history::update_temporal_camera_history_after_success;

pub(in crate::graphics::runtime::render_framework) fn submit_runtime_frame(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    mut frame: ViewportRenderFrame,
) -> Result<(), RenderFrameworkError> {
    crate::profile_scope!("runtime", "render_framework", "submit_runtime_frame");
    let _operation_guard = server.lock_operation();
    let context =
        match build_frame_submission_context(server, viewport, &frame.extract, frame.ui.as_ref()) {
            Ok(context) => context,
            Err(error) => {
                fail_pending_capture_after_preflight_error(server, viewport, &error);
                return Err(error);
            }
        };
    apply_submission_target_size_to_runtime_frame(&mut frame, &context);
    apply_submission_output_target_to_runtime_frame(&mut frame, &context);
    apply_submission_visibility_to_runtime_frame(&mut frame, &context);
    apply_effective_advanced_extracts_to_runtime_frame(&mut frame, &context);
    apply_effective_post_process_graph_to_runtime_frame(&mut frame, &context);
    apply_effective_particle_previous_state_to_runtime_frame(&mut frame, &context);
    let mut state = server.lock_state();
    let active_capture = begin_graphics_debugger_capture(&mut state, viewport);
    let prepared = match prepare_runtime_submission(&mut state, viewport, &context) {
        Ok(prepared) => prepared,
        Err(error) => {
            drop(finish_active_capture_and_relock(
                server,
                state,
                active_capture,
                None,
                Some(error.to_string()),
            ));
            return Err(error);
        }
    };
    let resolved_history = resolve_history_handle(&mut state, viewport, &context);
    state.last_virtual_geometry_debug_snapshot = frame.virtual_geometry_debug_snapshot.clone();
    let runtime_frame = attach_prepared_sidebands_to_runtime_frame(frame, &prepared);
    let rendered_frame = {
        crate::profile_scope!("runtime", "render_framework", "render_frame_with_pipeline");
        match state.renderer.render_frame_with_pipeline(
            &runtime_frame,
            context.compiled_pipeline(),
            context.capabilities(),
            resolved_history.current_history_handle(),
            resolved_history.previous_history_available(),
        ) {
            Ok(frame) => frame,
            Err(error) => {
                let error = render_framework_backend_error(error);
                drop(finish_active_capture_and_relock(
                    server,
                    state,
                    active_capture,
                    None,
                    Some(error.to_string()),
                ));
                return Err(error);
            }
        }
    };
    let frame_generation = rendered_frame.generation;
    state = finish_active_capture_and_relock(
        server,
        state,
        active_capture,
        Some(frame_generation),
        None,
    );
    let runtime_feedback = collect_runtime_feedback(&mut state.renderer, &context, &prepared);
    validate_viewport_generation(&state, viewport, &context)?;
    let record = state
        .viewports
        .get_mut(&viewport)
        .expect("viewport generation checked above");
    let record_update = record_submission(
        record,
        &context,
        prepared,
        resolved_history.allocated_history(),
        rendered_frame,
        runtime_feedback,
    );
    update_temporal_camera_history_after_success(record, &runtime_frame);
    update_particle_previous_state_after_success(record, &runtime_frame);
    release_previous_history(&mut state.renderer, &record_update);
    update_stats(&mut state, &context, &record_update, frame_generation);
    Ok(())
}

fn apply_effective_particle_previous_state_to_runtime_frame(
    frame: &mut ViewportRenderFrame,
    context: &super::super::frame_submission_context::FrameSubmissionContext,
) {
    frame.extract.particles.previous_sprites = context.particle_previous_sprites().to_vec();
}

fn fail_pending_capture_after_preflight_error(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    error: &RenderFrameworkError,
) {
    let mut state = server.lock_state();
    fail_pending_graphics_debugger_capture(&mut state, viewport, error.to_string());
}

fn apply_submission_output_target_to_runtime_frame(
    frame: &mut ViewportRenderFrame,
    context: &super::super::frame_submission_context::FrameSubmissionContext,
) {
    frame.output_target = context.output_target();
}

fn apply_submission_target_size_to_runtime_frame(
    frame: &mut ViewportRenderFrame,
    context: &super::super::frame_submission_context::FrameSubmissionContext,
) {
    frame.viewport_size = context.size();
    frame.extract.apply_viewport_size(context.size());
}

fn apply_submission_visibility_to_runtime_frame(
    frame: &mut ViewportRenderFrame,
    context: &super::super::frame_submission_context::FrameSubmissionContext,
) {
    frame.frame_visibility = Some(context.visibility_context().frame_visibility.clone());
}

fn apply_effective_advanced_extracts_to_runtime_frame(
    frame: &mut ViewportRenderFrame,
    context: &super::super::frame_submission_context::FrameSubmissionContext,
) {
    frame.extract.geometry.virtual_geometry = context.virtual_geometry_extract().cloned();
    if !context.hybrid_gi_enabled() {
        frame.extract.lighting.hybrid_global_illumination = None;
    }
}

fn apply_effective_post_process_graph_to_runtime_frame(
    frame: &mut ViewportRenderFrame,
    context: &super::super::frame_submission_context::FrameSubmissionContext,
) {
    frame.extract.post_process.bloom = context.post_process_bloom();
    frame.extract.post_process.exposure = context.post_process_exposure();
    frame.extract.post_process.color_grading = context.post_process_color_grading();
    frame.extract.post_process.effect_stack = context.post_process_effect_stack();
    frame.extract.post_process.volumes.clear();
    frame.extract.view.anti_alias = context.anti_alias_fallback().effective_settings();
    frame.extract.view.camera.temporal_jitter = context.temporal_jitter();
    frame.extract.post_process.stack = context.post_process_stack().clone();
    frame.extract.post_process.graph = context.post_process_graph().clone();
}

fn attach_prepared_sidebands_to_runtime_frame(
    frame: ViewportRenderFrame,
    prepared: &super::super::prepared_runtime_submission::PreparedRuntimeSubmission,
) -> ViewportRenderFrame {
    frame.with_prepared_runtime_sidebands(prepared.prepared_runtime_sidebands())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        AdvancedProfileRuntimePlan, AdvancedProviderAvailability, FallbackSkyboxKind,
        PreviewEnvironmentExtract, RenderCameraTargetKind, RenderCapabilitySummary,
        RenderFrameExtract, RenderOverlayExtract, RenderPluginRendererOutputs, RenderProfileBundle,
        RenderSceneGeometryExtract, RenderSceneSnapshot,
        RenderVirtualGeometryNodeClusterCullReadbackOutputs, RenderVirtualGeometryReadbackOutputs,
        RenderWorldSnapshotHandle, ViewportCameraSnapshot,
    };
    use crate::core::math::{UVec2, Vec4};

    use super::super::super::prepared_runtime_submission::PreparedRuntimeSubmission;

    #[test]
    fn direct_runtime_frame_submit_projects_prepared_sidebands() {
        let extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(44),
            empty_scene_snapshot(),
        );
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(1280, 720));
        let prepared = PreparedRuntimeSubmission::new(
            vec![5],
            vec![9],
            RenderPluginRendererOutputs {
                virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                    node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                        page_request_ids: vec![300],
                        ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
                    },
                    ..RenderVirtualGeometryReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            },
        );

        let frame = attach_prepared_sidebands_to_runtime_frame(frame, &prepared);

        assert_eq!(
            frame
                .prepared_runtime_sidebands()
                .hybrid_gi_evictable_probe_ids(),
            &[5]
        );
        assert_eq!(
            frame
                .prepared_runtime_sidebands()
                .virtual_geometry_evictable_page_ids(),
            &[9]
        );
        assert_eq!(
            frame
                .prepared_runtime_sidebands()
                .virtual_geometry_readback_outputs()
                .node_cluster_cull
                .page_request_ids,
            vec![300]
        );
    }

    #[test]
    fn direct_runtime_frame_submit_projects_resolved_output_target() {
        let extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(45),
            empty_scene_snapshot(),
        );
        let mut frame = ViewportRenderFrame::from_extract(extract, UVec2::new(1280, 720));
        let context = frame_submission_context_with_output_target(
            UVec2::new(96, 54),
            crate::graphics::ViewportRenderOutputTarget::Headless {
                size: UVec2::new(96, 54),
            },
        );

        apply_submission_target_size_to_runtime_frame(&mut frame, &context);
        apply_submission_output_target_to_runtime_frame(&mut frame, &context);

        assert_eq!(frame.viewport_size, UVec2::new(96, 54));
        assert_eq!(
            frame.output_target().kind(),
            RenderCameraTargetKind::Headless
        );
        assert_eq!(frame.output_target().size(), Some(UVec2::new(96, 54)));
    }

    fn empty_scene_snapshot() -> RenderSceneSnapshot {
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        }
    }

    fn frame_submission_context_with_output_target(
        size: UVec2,
        output_target: crate::graphics::ViewportRenderOutputTarget,
    ) -> super::super::super::frame_submission_context::FrameSubmissionContext {
        let extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(46),
            empty_scene_snapshot(),
        );
        super::super::super::frame_submission_context::FrameSubmissionContext::new(
            size,
            size,
            crate::core::framework::render::RenderPipelineHandle::new(1),
            0,
            None,
            empty_pipeline(),
            RenderCapabilitySummary::default(),
            crate::VisibilityContext::from_extract(&extract),
            None,
            Default::default(),
            None,
            output_target,
            Default::default(),
            None,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            1,
            default_render_advanced_runtime_plan(),
            Default::default(),
            Default::default(),
            Default::default(),
            false,
            false,
            None,
            Default::default(),
            None,
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            0,
            0,
            0,
            None,
            Default::default(),
            Vec::new(),
            Vec::new(),
            None,
            None,
            1,
        )
    }

    fn empty_pipeline() -> crate::CompiledRenderPipeline {
        let graph = crate::render_graph::RenderGraphBuilder::new("direct-runtime-frame-test")
            .compile()
            .unwrap();
        crate::CompiledRenderPipeline {
            handle: crate::core::framework::render::RenderPipelineHandle::new(1),
            name: "empty".to_string(),
            renderer_name: "empty".to_string(),
            stages: vec![crate::RenderPassStage::Opaque3d],
            pass_stages: Vec::new(),
            enabled_features: Vec::new(),
            required_extract_sections: Vec::new(),
            capability_requirements: Vec::new(),
            history_bindings: Vec::new(),
            graph,
        }
    }

    fn default_render_advanced_runtime_plan() -> AdvancedProfileRuntimePlan {
        AdvancedProfileRuntimePlan::from_profile_bundle(
            &RenderProfileBundle::default_render(),
            &RenderCapabilitySummary::default(),
            &AdvancedProviderAvailability::new(),
        )
    }
}
