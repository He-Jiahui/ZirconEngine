use crate::core::framework::render::{
    RenderFrameExtract, RenderFrameworkError, RenderViewportHandle,
};
use crate::ui::surface::{UiRenderCommandKind, UiRenderExtract};

use crate::{runtime::HybridGiSceneInputs, VisibilityContext};

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
    let synthesized_virtual_geometry =
        if virtual_geometry_enabled && extract.geometry.virtual_geometry.is_none() {
            let state = server.state.lock().unwrap();
            state.renderer.synthesize_virtual_geometry_extract(
                &extract.geometry.meshes,
                extract.geometry.virtual_geometry_debug.clone(),
            )
        } else {
            None
        };
    let synthesized_virtual_geometry_extract = synthesized_virtual_geometry
        .as_ref()
        .map(|output| output.extract().clone());
    let synthesized_virtual_geometry_cpu_reference_instances = synthesized_virtual_geometry
        .as_ref()
        .map(|output| output.cpu_reference_instances().to_vec())
        .unwrap_or_default();
    let synthesized_virtual_geometry_bvh_visualization_instances = synthesized_virtual_geometry
        .as_ref()
        .map(|output| output.bvh_visualization_instances().to_vec())
        .unwrap_or_default();
    let effective_virtual_geometry_extract = apply_virtual_geometry_debug_override(
        extract
            .geometry
            .virtual_geometry
            .clone()
            .or(synthesized_virtual_geometry_extract),
        extract.geometry.virtual_geometry_debug,
    );
    let supplemented_extract = effective_virtual_geometry_extract
        .as_ref()
        .filter(|_| extract.geometry.virtual_geometry.is_none())
        .map(|virtual_geometry| {
            let mut supplemented = extract.clone();
            supplemented.geometry.virtual_geometry = Some(virtual_geometry.clone());
            supplemented
        });
    let visibility_extract = supplemented_extract.as_ref().unwrap_or(extract);
    let sanitized_visibility_extract;
    let visibility_extract = if hybrid_gi_enabled {
        visibility_extract
    } else {
        sanitized_visibility_extract = visibility_extract_without_hybrid_gi(visibility_extract);
        &sanitized_visibility_extract
    };
    let visibility_context = VisibilityContext::from_extract_with_history(
        visibility_extract,
        viewport_state.previous_visibility(),
    );
    let hybrid_gi_update_plan =
        hybrid_gi_enabled.then(|| visibility_context.hybrid_gi_update_plan.clone());
    let hybrid_gi_feedback =
        hybrid_gi_enabled.then(|| visibility_context.hybrid_gi_feedback.clone());
    let hybrid_gi_scene_inputs = hybrid_gi_enabled
        .then(|| {
            HybridGiSceneInputs::new(
                extract.geometry.meshes.clone(),
                extract.lighting.directional_lights.clone(),
                extract.lighting.point_lights.clone(),
                extract.lighting.spot_lights.clone(),
            )
        })
        .unwrap_or_default();
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
        viewport_state.take_previous_hybrid_gi_runtime(),
        viewport_state.take_previous_virtual_geometry_runtime(),
        hybrid_gi_enabled,
        virtual_geometry_enabled,
        hybrid_gi_enabled
            .then(|| extract.lighting.hybrid_global_illumination.clone())
            .flatten(),
        hybrid_gi_scene_inputs,
        hybrid_gi_update_plan,
        hybrid_gi_feedback,
        virtual_geometry_enabled
            .then(|| effective_virtual_geometry_extract.clone())
            .flatten(),
        synthesized_virtual_geometry_cpu_reference_instances,
        synthesized_virtual_geometry_bvh_visualization_instances,
        virtual_geometry_page_upload_plan,
        virtual_geometry_feedback,
        viewport_state.predicted_generation(),
    ))
}

fn apply_virtual_geometry_debug_override(
    extract: Option<crate::core::framework::render::RenderVirtualGeometryExtract>,
    debug_override: Option<crate::core::framework::render::RenderVirtualGeometryDebugState>,
) -> Option<crate::core::framework::render::RenderVirtualGeometryExtract> {
    let mut extract = extract?;
    if let Some(debug_override) = debug_override {
        extract.debug = debug_override;
    }
    Some(extract)
}

fn visibility_extract_without_hybrid_gi(extract: &RenderFrameExtract) -> RenderFrameExtract {
    let mut extract = extract.clone();
    extract.lighting.hybrid_global_illumination = None;
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
