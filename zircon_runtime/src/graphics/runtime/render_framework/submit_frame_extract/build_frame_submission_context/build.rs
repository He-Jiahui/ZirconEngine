use crate::core::framework::render::{
    RenderFrameExtract, RenderFrameworkError, RenderViewportHandle, RenderVirtualGeometryExtract,
};
use zircon_runtime_interface::ui::surface::{UiRenderCommandKind, UiRenderExtract};

use crate::VisibilityContext;

use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::super::frame_submission_context::{FrameSubmissionContext, UiSubmissionStats};
use super::compile_pipeline::compile_submission_pipeline;
use super::resolve_enabled_features::resolve_enabled_features;
use super::resolve_viewport_record_state::resolve_viewport_record_state;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn build_frame_submission_context(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: &RenderFrameExtract,
    ui_extract: Option<&UiRenderExtract>,
) -> Result<FrameSubmissionContext, RenderFrameworkError> {
    let mut viewport_state = resolve_viewport_record_state(server, viewport)?;
    let compiled_pipeline = compile_submission_pipeline(&viewport_state, extract)?;
    let (hybrid_gi_enabled, virtual_geometry_enabled) =
        resolve_enabled_features(&compiled_pipeline);
    let effective_virtual_geometry_extract = apply_virtual_geometry_debug_override(
        extract.geometry.virtual_geometry.clone(),
        extract.geometry.virtual_geometry_debug,
    );
    let sanitized_feature_extract;
    let visibility_extract = if hybrid_gi_enabled && virtual_geometry_enabled {
        extract
    } else {
        sanitized_feature_extract =
            visibility_extract_without_disabled_advanced_features(extract, hybrid_gi_enabled);
        &sanitized_feature_extract
    };
    let visibility_context = VisibilityContext::from_extract_with_history(
        visibility_extract,
        viewport_state.previous_visibility(),
    );
    let hybrid_gi_update_plan =
        hybrid_gi_enabled.then(|| visibility_context.hybrid_gi_update_plan.clone());
    let hybrid_gi_feedback =
        hybrid_gi_enabled.then(|| visibility_context.hybrid_gi_feedback.clone());
    let virtual_geometry_page_upload_plan = virtual_geometry_enabled
        .then(|| visibility_context.virtual_geometry_page_upload_plan.clone());
    let virtual_geometry_feedback =
        virtual_geometry_enabled.then(|| visibility_context.virtual_geometry_feedback.clone());

    Ok(FrameSubmissionContext::new(
        viewport_state.size(),
        viewport_state.pipeline_handle(),
        viewport_state.take_quality_profile(),
        compiled_pipeline,
        visibility_context,
        ui_extract
            .map(compute_ui_submission_stats)
            .unwrap_or_default(),
        hybrid_gi_enabled,
        virtual_geometry_enabled,
        hybrid_gi_enabled
            .then(|| extract.lighting.hybrid_global_illumination.clone())
            .flatten(),
        hybrid_gi_update_plan,
        hybrid_gi_feedback,
        virtual_geometry_enabled
            .then(|| effective_virtual_geometry_extract.clone())
            .flatten(),
        Vec::new(),
        Vec::new(),
        virtual_geometry_page_upload_plan,
        virtual_geometry_feedback,
        viewport_state.predicted_generation(),
    ))
}

fn apply_virtual_geometry_debug_override(
    extract: Option<RenderVirtualGeometryExtract>,
    debug_override: Option<crate::core::framework::render::RenderVirtualGeometryDebugState>,
) -> Option<RenderVirtualGeometryExtract> {
    let mut extract = extract?;
    if let Some(debug_override) = debug_override {
        extract.debug = debug_override;
    }
    Some(extract)
}

fn visibility_extract_without_disabled_advanced_features(
    extract: &RenderFrameExtract,
    hybrid_gi_enabled: bool,
) -> RenderFrameExtract {
    let mut extract = extract.clone();
    if !hybrid_gi_enabled {
        extract.lighting.hybrid_global_illumination = None;
    }
    extract
}

fn compute_ui_submission_stats(extract: &UiRenderExtract) -> UiSubmissionStats {
    let mut stats = UiSubmissionStats::default();
    for command in &extract.list.commands {
        stats.record_command();
        if matches!(command.kind, UiRenderCommandKind::Quad) {
            stats.record_quad();
        }
        if command.text.is_some() {
            stats.record_text_payload();
        }
        if command.image.is_some() {
            stats.record_image_payload();
        }
        if command.clip_frame.is_some() {
            stats.record_clipped_command();
        }
    }
    stats
}
