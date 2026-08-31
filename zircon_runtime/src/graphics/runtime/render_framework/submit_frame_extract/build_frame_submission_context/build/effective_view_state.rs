use crate::core::framework::render::{
    AntiAliasFallbackReport, AoSourceSettings, FrameHistoryInvalidationReason, PostProcessExtract,
    PostProcessPassGraph, PostProcessStackDescriptor, RenderBloomSettings,
    RenderColorGradingSettings, RenderExposureSettings, RenderFrameExtract, RenderPipelineHandle,
    RenderPostProcessEffectStackSettings, RenderResolutionPolicy, RenderUpscalerKind,
    RenderViewExtract, RenderViewFamilyPipeline, RenderViewportHandle, RenderViewportRect,
    TemporalJitterSample,
};
use crate::core::math::UVec2;
use crate::graphics::runtime::render_framework::submit_frame_extract::build_frame_submission_context::viewport_record_state::ViewportRecordState;
use crate::graphics::runtime::render_framework::viewport_record::ViewportCameraHistoryKey;
use crate::graphics::runtime::render_framework::wgpu_render_framework::WgpuRenderFrameworkAccess;
use crate::graphics::runtime::FrameHistoryValidationKey;
use crate::graphics::CompiledRenderPipeline;

pub(super) fn resolve_view_family_pipeline_for_submission(
    view: &RenderViewExtract,
    submission_size: UVec2,
    upscaler: RenderUpscalerKind,
) -> RenderViewFamilyPipeline {
    let view_camera = view.selected_effective_camera();
    let display_viewport = view
        .selected_camera_descriptor()
        .and_then(|descriptor| descriptor.viewport_rect)
        .map(|viewport| viewport.clamped_to_size(submission_size))
        .unwrap_or_else(|| RenderViewportRect::new(UVec2::ZERO, submission_size));
    RenderViewFamilyPipeline::resolve_for_viewport(
        submission_size,
        display_viewport,
        RenderResolutionPolicy::with_spatial_primary_fraction(
            view_camera.dynamic_resolution.clamped_scale(),
        ),
        upscaler,
    )
}

pub(super) fn apply_renderer_owned_particle_previous_state(
    extract: &RenderFrameExtract,
    viewport_state: &mut ViewportRecordState,
) -> Option<Vec<crate::core::framework::render::RenderParticlePreviousSpriteSnapshot>> {
    if !extract.particles.previous_sprites.is_empty() {
        return None;
    }
    Some(viewport_state.take_previous_particle_sprites())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_renderer_owned_post_process_snapshot(
    source: &PostProcessExtract,
    ambient_occlusion: AoSourceSettings,
    bloom: RenderBloomSettings,
    exposure: RenderExposureSettings,
    color_grading: RenderColorGradingSettings,
    effect_stack: RenderPostProcessEffectStackSettings,
    stack: PostProcessStackDescriptor,
    graph: PostProcessPassGraph,
) -> PostProcessExtract {
    PostProcessExtract {
        preview: source.preview.clone(),
        display_mode: source.display_mode,
        ambient_occlusion,
        bloom,
        exposure,
        color_grading,
        effect_stack,
        volumes: Vec::new(),
        stack,
        graph,
    }
}

pub(super) fn frame_history_invalidation_reason(
    framework: &dyn WgpuRenderFrameworkAccess,
    viewport: RenderViewportHandle,
    target_size: UVec2,
    render_size: UVec2,
    pipeline_handle: RenderPipelineHandle,
    compiled_pipeline: &CompiledRenderPipeline,
    camera_history_key: &ViewportCameraHistoryKey,
    history_validation_key: &FrameHistoryValidationKey,
    temporal_reprojection_compatible: bool,
) -> Option<FrameHistoryInvalidationReason> {
    let state = framework.lock_state();
    let Some(history) = state
        .viewports
        .get(&viewport)
        .and_then(|record| record.history(camera_history_key))
    else {
        return Some(FrameHistoryInvalidationReason::NoPreviousFrame);
    };
    history
        .incompatibility_reason(
            target_size,
            render_size,
            pipeline_handle,
            &compiled_pipeline.history_bindings,
            history_validation_key,
        )
        .or_else(|| {
            (!temporal_reprojection_compatible).then_some(FrameHistoryInvalidationReason::CameraCut)
        })
}

pub(super) fn apply_effective_view_settings(
    extract: &mut RenderFrameExtract,
    anti_alias_report: AntiAliasFallbackReport,
    temporal_jitter: TemporalJitterSample,
) {
    extract.view.anti_alias = anti_alias_report.effective_settings();
    extract.view.camera.temporal_jitter = temporal_jitter;
    extract.view.sync_selected_descriptor_camera_payload();
}
