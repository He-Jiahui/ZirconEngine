use std::sync::MutexGuard;
use std::time::Instant;

use crate::core::framework::render::{RenderFrameworkError, RenderViewportHandle};

use crate::graphics::{
    types::{ViewportCameraStackAttachmentPolicy, ViewportRenderFrame},
    ViewportCameraStackOutputPolicy, ViewportRenderRegion,
};

use super::super::super::environment_capture_submission::pump_environment_capture_source_locked;
use super::super::super::frame_profiler::FrameProfiler;
use super::super::super::graphics_debugger_capture::{
    begin_graphics_debugger_capture, fail_pending_graphics_debugger_capture,
    finish_active_capture_and_relock,
};
use super::super::super::render_framework_backend_error::render_framework_backend_error;
use super::super::super::render_framework_state::{
    RenderFrameworkState, ViewportPickFrameSnapshot,
};
use super::super::super::wgpu_render_framework::{
    WgpuRenderFramework, WgpuRenderFrameworkAccess, WgpuRenderFrameworkCore,
};
use super::super::build_frame_submission_context::build_frame_submission_context_from_runtime_frame_extract;
use super::super::prepare_runtime_submission::prepare_runtime_submission;
use super::super::record_submission::record_submission;
use super::super::update_stats::{update_stats, SharedViewportProductReports};
use super::super::viewport_generation_guard::{
    validate_viewport_generation, viewport_record_mut_after_generation_check,
};
use super::camera_loop::{submit_camera_loop_frame, CameraLoopOutputPolicy};
use super::collect_runtime_feedback::collect_runtime_feedback;
use super::completion_error_stats::publish_scene_submission_completion_stats;
use super::publish_viewport_product::publish_viewport_product;
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
    framework.dispatch_runtime_frame_submission(submit_runtime_frame_on_core, viewport, frame)
}

fn submit_runtime_frame_on_core(
    framework: &WgpuRenderFrameworkCore,
    viewport: RenderViewportHandle,
    frame: ViewportRenderFrame,
) -> Result<(), RenderFrameworkError> {
    submit_runtime_frame_locked(framework, viewport, frame)
}

fn submit_runtime_frame_locked(
    framework: &dyn WgpuRenderFrameworkAccess,
    viewport: RenderViewportHandle,
    frame: ViewportRenderFrame,
) -> Result<(), RenderFrameworkError> {
    let submit_started = Instant::now();
    let result = submit_camera_loop_frame(
        framework,
        viewport,
        frame,
        &submit_started,
        fail_pending_capture_after_preflight_error,
        submit_selected_runtime_frame,
    );
    if result.is_ok() {
        pump_environment_capture_source_locked(framework);
    }
    result
}

fn submit_selected_runtime_frame(
    framework: &dyn WgpuRenderFrameworkAccess,
    viewport: RenderViewportHandle,
    frame: &mut ViewportRenderFrame,
    submit_started: &Instant,
    output_policy: CameraLoopOutputPolicy,
) -> Result<(), RenderFrameworkError> {
    let output_policy = ViewportCameraStackOutputPolicy::from(output_policy);
    let owns_viewport_submission = output_policy.owns_viewport_submission();
    let owns_shared_viewport_products = output_policy.owns_shared_viewport_products();
    frame.camera_stack_output_policy = output_policy;
    let mut context = {
        crate::profile_scope!("runtime", "render_framework", "build_submission_context");
        match build_frame_submission_context_from_runtime_frame_extract(
            framework,
            viewport,
            &frame.extract,
            frame.ui.as_deref(),
        ) {
            Ok(context) => context,
            Err(error) => {
                fail_pending_capture_after_preflight_error(framework, viewport, &error);
                return Err(error);
            }
        }
    };
    apply_submission_extract_to_runtime_frame(frame, &mut context);
    apply_submission_output_target_to_runtime_frame(frame, &context);
    apply_submission_visibility_to_runtime_frame(frame, &context);
    let mut state = framework.lock_state();
    let mip_streaming_residency_budget = state.memory_budget.persistent_texture_bytes();
    state
        .renderer
        .set_mip_streaming_residency_budget(mip_streaming_residency_budget);
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
    attach_prepared_sidebands_to_runtime_frame(frame, prepared);
    state
        .renderer
        .set_global_material_mip_bias(context.global_material_mip_bias());
    let viewport_product_requested =
        owns_viewport_submission && state.viewport_products.has_direct_presenter();
    let viewport_capture = (owns_viewport_submission
        && state.viewport_products.requires_async_capture(viewport))
    .then(|| {
        let generation = state.renderer.next_frame_generation();
        state
            .viewports
            .get(&viewport)
            .map(|record| record.async_capture_request(generation))
    })
    .flatten();
    let environment_ibl_bake_reservation = context.take_environment_ibl_bake_reservation();
    let mut rendered_frame = {
        crate::profile_scope!("runtime", "render_framework", "render_frame_with_pipeline");
        let result = if state.renderer.supports_compiled_scene_graph() {
            state
                .renderer
                .render_frame_with_pipeline_async_capture_task_pool_with_environment_ibl_bake_reservation(
                    &*frame,
                    context.compiled_pipeline(),
                    context.capabilities(),
                    crate::core::framework::render::RenderFrameHistoryInput::new(
                        resolved_history.current_history_handle(),
                        resolved_history.previous_history_available(),
                        context.history_invalidation_reason(),
                    ),
                    framework.compute_task_pool(),
                    viewport_capture,
                    environment_ibl_bake_reservation,
                    viewport_product_requested,
                )
        } else {
            state
                .renderer
                .render_frame_direct_submission(frame, viewport_product_requested)
        };
        match result {
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
    let frame_generation = rendered_frame.generation();
    if let Err(error) = validate_viewport_generation(&state, viewport, &context) {
        finish_or_fail_capture_after_submission_error(
            framework,
            state,
            viewport,
            active_capture,
            &error,
        );
        return Err(error);
    }
    if viewport_product_requested {
        if let Err(error) = publish_viewport_product(&mut state, viewport, &mut rendered_frame) {
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
    if owns_shared_viewport_products {
        state.last_virtual_geometry_debug_snapshot = frame.virtual_geometry_debug_snapshot.clone();
    }
    if !owns_viewport_submission {
        record_non_viewport_camera_state_after_success(
            &mut state,
            viewport,
            &context,
            frame,
            camera_light_grid_report,
            runtime_feedback,
            frame_generation,
            resolved_history.allocated_history(),
        )?;
        return Ok(());
    }
    let record = viewport_record_mut_after_generation_check(&mut state, viewport, &context)?;
    let requires_hit_proxies = record.requires_hit_proxies();
    record.record_camera_product_reports(
        context.camera_history_key(),
        camera_light_grid_report,
        frame.virtual_geometry_debug_snapshot.as_ref(),
    );
    let record_update = record_submission(
        record,
        viewport,
        &context,
        resolved_history.allocated_history(),
        rendered_frame,
        runtime_feedback,
    )?;
    update_temporal_camera_history_after_success(
        record,
        &*frame,
        context.camera_history_key(),
        true,
    );
    update_particle_previous_state_after_success(record, frame, context.camera_history_key());
    release_previous_history(&mut state.renderer, &record_update);
    if owns_shared_viewport_products {
        let shared_product_reports = SharedViewportProductReports::new(camera_light_grid_report);
        let frame_profile_write = update_stats(
            &mut state,
            &context,
            &record_update,
            frame_generation,
            FrameProfiler::elapsed_micros(*submit_started),
            shared_product_reports,
        );
        let viewport_record =
            viewport_record_mut_after_generation_check(&mut state, viewport, &context)?;
        viewport_record.attach_capture_frame_profile(&frame_profile_write.capture_profile);
        for profile in &frame_profile_write.resolved_gpu_profiles {
            viewport_record.attach_capture_frame_profile(profile);
        }
    }
    if requires_hit_proxies {
        state
            .viewport_pick_frames
            .publish(ViewportPickFrameSnapshot::from_rendered_frame(
                viewport,
                frame_generation,
                frame,
                context.virtual_geometry_enabled(),
            ));
    }
    state.last_retained_scene_color_viewport = Some(viewport);
    crate::profile_counter!(
        "runtime",
        "render_framework.last_frame_generation",
        frame_generation
    );
    Ok(())
}

fn apply_submission_extract_to_runtime_frame(
    frame: &mut ViewportRenderFrame,
    context: &mut super::super::frame_submission_context::FrameSubmissionContext,
) {
    frame.viewport_size = context.size();
    frame.extract = context.submission_extract();
    frame.post_process_override = Some(context.post_process_shared());
    frame.environment_source_cubemap_override = context.take_environment_source_cubemap_override();
    frame.particle_previous_sprites_override = context.take_particle_previous_sprites_override();
    refresh_camera_policy_to_runtime_frame(frame);
}

fn fail_pending_capture_after_preflight_error(
    framework: &dyn WgpuRenderFrameworkAccess,
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
    )
    .with_local_size(frame.extract.view.effective_render_size());
}

fn attach_prepared_sidebands_to_runtime_frame(
    frame: &mut ViewportRenderFrame,
    prepared: super::super::prepared_runtime_submission::PreparedRuntimeSubmission,
) {
    frame.prepared_runtime_sidebands = prepared.into_prepared_runtime_sidebands();
}

fn finish_or_fail_capture_after_submission_error(
    framework: &dyn WgpuRenderFrameworkAccess,
    mut state: MutexGuard<'_, RenderFrameworkState>,
    viewport: RenderViewportHandle,
    active_capture: bool,
    error: &RenderFrameworkError,
) {
    publish_scene_submission_completion_stats(&mut state);
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
            None,
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
        let mut context = frame_submission_context_with_output_target(
            UVec2::new(96, 54),
            crate::graphics::ViewportRenderOutputTarget::Headless {
                size: UVec2::new(96, 54),
            },
        );

        apply_submission_extract_to_runtime_frame(&mut frame, &mut context);
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
            environment: crate::core::framework::render::EnvironmentExtract::default(),
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
            false,
            None,
            output_target,
            Default::default(),
            crate::core::framework::render::RenderViewFamilyPipeline::resolve(
                size,
                Default::default(),
                crate::core::framework::render::RenderUpscalerKind::Spatial,
            ),
            None,
            Default::default(),
            Default::default(),
            Default::default(),
            default_render_advanced_runtime_plan(),
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
        crate::graphics::CompiledRenderPipeline::from_parts(
            crate::graphics::pipeline::CompiledRenderPipelineParts {
                handle: crate::core::framework::render::RenderPipelineHandle::new(1),
                name: "empty".to_string(),
                renderer_name: "empty".to_string(),
                execution_pass_metadata: Vec::new(),
                enabled_features: Vec::new(),
                required_extract_sections: Vec::new(),
                capability_requirements: Vec::new(),
                history_bindings: Vec::new(),
                environment_ibl_bake_request: None,
                ambient_occlusion_profile: None,
                half_resolution_transparency_depth_sigma:
                    crate::core::framework::render::DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
                graph,
            },
        )
        .expect("empty direct runtime frame pipeline execution packet")
    }

    fn default_render_advanced_runtime_plan() -> AdvancedProfileRuntimePlan {
        AdvancedProfileRuntimePlan::from_profile_bundle(
            &RenderProfileBundle::default_render(),
            &RenderCapabilitySummary::default(),
            &AdvancedProviderAvailability::new(),
        )
    }
}
