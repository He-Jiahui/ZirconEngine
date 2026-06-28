use std::sync::MutexGuard;

use crate::core::framework::render::{RenderFrameworkError, RenderViewportHandle};

use crate::graphics::{
    types::{ViewportCameraStackAttachmentPolicy, ViewportRenderFrame},
    ViewportCameraStackOutputPolicy, ViewportRenderRegion,
};

use super::super::super::graphics_debugger_capture::{
    begin_graphics_debugger_capture, fail_pending_graphics_debugger_capture,
    finish_active_capture_and_relock,
};
use super::super::super::render_framework_backend_error::render_framework_backend_error;
use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::super::build_frame_submission_context::{
    build_frame_submission_context_from_runtime_frame_extract, FrameSubmissionSourcePayloads,
};
use super::super::prepare_runtime_submission::prepare_runtime_submission;
use super::super::record_submission::record_submission;
use super::super::update_stats::{update_stats, SharedViewportProductReports};
use super::super::viewport_generation_guard::{
    validate_viewport_generation, viewport_record_mut_after_generation_check,
};
use super::camera_loop::{submit_camera_loop_frame, CameraLoopOutputPolicy};
use super::collect_runtime_feedback::collect_runtime_feedback;
use super::record_camera_history::record_non_viewport_camera_state_after_success;
use super::release_previous_history::release_previous_history;
use super::resolve_history_handle::resolve_history_handle;
use super::update_particle_previous_state::update_particle_previous_state_after_success;
use super::update_temporal_camera_history::update_temporal_camera_history_after_success;

pub(in crate::graphics::runtime::render_framework) fn submit_runtime_frame(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    frame: ViewportRenderFrame,
) -> Result<(), RenderFrameworkError> {
    crate::profile_scope!("runtime", "render_framework", "submit_runtime_frame");
    let _operation_guard = framework.lock_operation();
    submit_camera_loop_frame(
        framework,
        viewport,
        frame,
        fail_pending_capture_after_preflight_error,
        submit_selected_runtime_frame,
    )
}

fn submit_selected_runtime_frame(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    frame: &mut ViewportRenderFrame,
    source_payloads: Option<FrameSubmissionSourcePayloads<'_>>,
    output_policy: CameraLoopOutputPolicy,
) -> Result<(), RenderFrameworkError> {
    let output_policy = ViewportCameraStackOutputPolicy::from(output_policy);
    let owns_viewport_submission = output_policy.owns_viewport_submission();
    let owns_shared_viewport_products = output_policy.owns_shared_viewport_products();
    frame.camera_stack_output_policy = output_policy;
    let context = {
        crate::profile_scope!("runtime", "render_framework", "build_submission_context");
        match build_frame_submission_context_from_runtime_frame_extract(
            framework,
            viewport,
            &mut frame.extract,
            frame.ui.as_ref(),
            source_payloads,
        ) {
            Ok(context) => context,
            Err(error) => {
                fail_pending_capture_after_preflight_error(framework, viewport, &error);
                return Err(error);
            }
        }
    };
    apply_submission_extract_to_runtime_frame(frame, &context);
    apply_submission_output_target_to_runtime_frame(frame, &context);
    apply_submission_visibility_to_runtime_frame(frame, &context);
    let mut state = framework.lock_state();
    let active_capture =
        owns_shared_viewport_products && begin_graphics_debugger_capture(&mut state, viewport);
    let prepared = {
        crate::profile_scope!("runtime", "render_framework", "prepare_runtime_submission");
        match prepare_runtime_submission(&mut state, viewport, &context) {
            Ok(prepared) => prepared,
            Err(error) => {
                finish_or_fail_capture_after_submission_error(
                    framework,
                    state,
                    viewport,
                    active_capture,
                    &error,
                );
                return Err(error);
            }
        }
    };
    let resolved_history = resolve_history_handle(&mut state, viewport, &context);
    if owns_shared_viewport_products {
        state.last_virtual_geometry_debug_snapshot = frame.virtual_geometry_debug_snapshot.clone();
    }
    attach_prepared_sidebands_to_runtime_frame(frame, prepared);
    let rendered_frame = {
        crate::profile_scope!("runtime", "render_framework", "render_frame_with_pipeline");
        match state.renderer.render_frame_with_pipeline(
            &*frame,
            context.compiled_pipeline(),
            context.capabilities(),
            resolved_history.current_history_handle(),
            resolved_history.previous_history_available(),
        ) {
            Ok(frame) => frame,
            Err(error) => {
                let error = render_framework_backend_error(error);
                finish_or_fail_capture_after_submission_error(
                    framework,
                    state,
                    viewport,
                    active_capture,
                    &error,
                );
                return Err(error);
            }
        }
    };
    let frame_generation = rendered_frame.generation;
    state = finish_active_capture_and_relock(
        framework,
        state,
        active_capture,
        Some(frame_generation),
        None,
    );
    let runtime_feedback = {
        crate::profile_scope!("runtime", "render_framework", "collect_runtime_feedback");
        collect_runtime_feedback(
            &mut state.renderer,
            &context,
            frame.prepared_runtime_sidebands_mut(),
        )
    };
    let camera_light_grid_report = state.renderer.last_light_grid_report();
    if let Err(error) = validate_viewport_generation(&state, viewport, &context) {
        if !owns_shared_viewport_products {
            fail_pending_graphics_debugger_capture(&mut state, viewport, error.to_string());
        }
        return Err(error);
    }
    if !owns_viewport_submission {
        record_non_viewport_camera_state_after_success(
            &mut state,
            viewport,
            &context,
            &*frame,
            camera_light_grid_report,
            runtime_feedback,
            frame_generation,
            resolved_history.allocated_history(),
        )?;
        return Ok(());
    }
    let record = viewport_record_mut_after_generation_check(&mut state, viewport, &context)?;
    record.record_camera_product_reports(
        context.camera_history_key(),
        camera_light_grid_report,
        frame.virtual_geometry_debug_snapshot.as_ref(),
    );
    let record_update = record_submission(
        record,
        &context,
        resolved_history.allocated_history(),
        rendered_frame,
        runtime_feedback,
    );
    update_temporal_camera_history_after_success(
        record,
        &*frame,
        context.camera_history_key(),
        true,
    );
    update_particle_previous_state_after_success(record, &*frame, context.camera_history_key());
    release_previous_history(&mut state.renderer, &record_update);
    if owns_shared_viewport_products {
        let shared_product_reports = SharedViewportProductReports::new(camera_light_grid_report);
        update_stats(
            &mut state,
            &context,
            &record_update,
            frame_generation,
            shared_product_reports,
        );
    }
    crate::profile_counter!(
        "runtime",
        "render_framework.last_frame_generation",
        frame_generation
    );
    Ok(())
}

fn apply_submission_extract_to_runtime_frame(
    frame: &mut ViewportRenderFrame,
    context: &super::super::frame_submission_context::FrameSubmissionContext,
) {
    frame.viewport_size = context.size();
    frame.extract = context.source_extract();
    refresh_camera_policy_to_runtime_frame(frame);
}

fn fail_pending_capture_after_preflight_error(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    error: &RenderFrameworkError,
) {
    let mut state = framework.lock_state();
    fail_pending_graphics_debugger_capture(&mut state, viewport, error.to_string());
}

fn apply_submission_output_target_to_runtime_frame(
    frame: &mut ViewportRenderFrame,
    context: &super::super::frame_submission_context::FrameSubmissionContext,
) {
    frame.output_target = context.output_target();
}

fn apply_submission_visibility_to_runtime_frame(
    frame: &mut ViewportRenderFrame,
    context: &super::super::frame_submission_context::FrameSubmissionContext,
) {
    frame.frame_visibility = Some(context.visibility_context().frame_visibility.clone());
}

fn refresh_camera_policy_to_runtime_frame(frame: &mut ViewportRenderFrame) {
    frame.camera_stack_attachment_policy = frame
        .extract
        .view
        .selected_camera_descriptor()
        .map(ViewportCameraStackAttachmentPolicy::from_camera)
        .unwrap_or_default();
    frame.render_region = ViewportRenderRegion::from_camera(
        frame.extract.view.selected_camera_descriptor(),
        frame.viewport_size,
    );
}

fn attach_prepared_sidebands_to_runtime_frame(
    frame: &mut ViewportRenderFrame,
    prepared: super::super::prepared_runtime_submission::PreparedRuntimeSubmission,
) {
    frame.prepared_runtime_sidebands = prepared.into_prepared_runtime_sidebands();
}

fn finish_or_fail_capture_after_submission_error(
    framework: &WgpuRenderFramework,
    mut state: MutexGuard<'_, RenderFrameworkState>,
    viewport: RenderViewportHandle,
    active_capture: bool,
    error: &RenderFrameworkError,
) {
    if active_capture {
        drop(finish_active_capture_and_relock(
            framework,
            state,
            active_capture,
            None,
            Some(error.to_string()),
        ));
    } else {
        fail_pending_graphics_debugger_capture(&mut state, viewport, error.to_string());
    }
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
        let mut frame = ViewportRenderFrame::from_extract(extract, UVec2::new(1280, 720));
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

        attach_prepared_sidebands_to_runtime_frame(&mut frame, prepared);

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

        apply_submission_extract_to_runtime_frame(&mut frame, &context);
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
            Default::default(),
            std::sync::Arc::new(empty_pipeline()),
            RenderCapabilitySummary::default(),
            crate::graphics::VisibilityContext::from_extract(&extract),
            None,
            super::super::super::super::viewport_record::ViewportCameraHistoryKey::from_camera(
                extract
                    .view
                    .selected_camera_descriptor()
                    .expect("test extract has selected camera descriptor"),
            ),
            Default::default(),
            None,
            output_target,
            Default::default(),
            None,
            Default::default(),
            Default::default(),
            Default::default(),
            default_render_advanced_runtime_plan(),
            Default::default(),
            Default::default(),
            false,
            false,
            None,
            Default::default(),
            None,
            None,
            std::sync::Arc::new(extract.clone()),
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

    fn empty_pipeline() -> crate::graphics::CompiledRenderPipeline {
        let graph = crate::render_graph::RenderGraphBuilder::new("direct-runtime-frame-test")
            .compile()
            .unwrap();
        crate::graphics::CompiledRenderPipeline {
            handle: crate::core::framework::render::RenderPipelineHandle::new(1),
            name: "empty".to_string(),
            renderer_name: "empty".to_string(),
            stages: vec![crate::graphics::RenderPassStage::Opaque3d],
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
