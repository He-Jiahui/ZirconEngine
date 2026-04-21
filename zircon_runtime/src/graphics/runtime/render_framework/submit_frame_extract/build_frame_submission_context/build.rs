use crate::core::framework::render::{
    RenderFrameExtract, RenderFrameworkError, RenderViewportHandle,
};
use crate::ui::surface::{UiRenderCommandKind, UiRenderExtract};

use crate::VisibilityContext;

use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::super::frame_submission_context::{
    FrameSubmissionContext, HybridGiSceneInputs, UiSubmissionStats,
};
use super::compile_pipeline::compile_submission_pipeline;
use super::resolve_enabled_features::resolve_enabled_features;
use super::resolve_viewport_record_state::resolve_viewport_record_state;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn build_frame_submission_context(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: &RenderFrameExtract,
    ui_extract: Option<&UiRenderExtract>,
) -> Result<FrameSubmissionContext, RenderFrameworkError> {
    let viewport_state = resolve_viewport_record_state(server, viewport)?;
    let compiled_pipeline = compile_submission_pipeline(&viewport_state, extract)?;
    let (hybrid_gi_enabled, virtual_geometry_enabled) =
        resolve_enabled_features(&compiled_pipeline);
    let synthesized_virtual_geometry =
        if virtual_geometry_enabled && extract.geometry.virtual_geometry.is_none() {
            let state = server.state.lock().unwrap();
            state
                .renderer
                .synthesize_virtual_geometry_extract(&extract.geometry.meshes)
        } else {
            None
        };
    let synthesized_virtual_geometry_extract = synthesized_virtual_geometry
        .as_ref()
        .map(|output| output.extract.clone());
    let synthesized_virtual_geometry_cpu_reference_instances = synthesized_virtual_geometry
        .as_ref()
        .map(|output| output.cpu_reference_instances.clone())
        .unwrap_or_default();
    let synthesized_virtual_geometry_bvh_visualization_instances = synthesized_virtual_geometry
        .as_ref()
        .map(|output| output.bvh_visualization_instances.clone())
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
    let visibility_context = VisibilityContext::from_extract_with_history(
        supplemented_extract.as_ref().unwrap_or(extract),
        viewport_state.previous_visibility.as_ref(),
    );
    let hybrid_gi_update_plan =
        hybrid_gi_enabled.then(|| visibility_context.hybrid_gi_update_plan.clone());
    let hybrid_gi_feedback =
        hybrid_gi_enabled.then(|| visibility_context.hybrid_gi_feedback.clone());
    let hybrid_gi_scene_inputs = hybrid_gi_enabled
        .then(|| HybridGiSceneInputs {
            meshes: extract.geometry.meshes.clone(),
            directional_lights: extract.lighting.directional_lights.clone(),
            point_lights: extract.lighting.point_lights.clone(),
            spot_lights: extract.lighting.spot_lights.clone(),
        })
        .unwrap_or_default();
    let virtual_geometry_page_upload_plan = virtual_geometry_enabled
        .then(|| visibility_context.virtual_geometry_page_upload_plan.clone());
    let virtual_geometry_feedback =
        virtual_geometry_enabled.then(|| visibility_context.virtual_geometry_feedback.clone());

    Ok(FrameSubmissionContext {
        size: viewport_state.size,
        pipeline_handle: viewport_state.pipeline_handle,
        quality_profile: viewport_state.quality_profile,
        compiled_pipeline,
        visibility_context,
        ui_stats: ui_extract
            .map(compute_ui_submission_stats)
            .unwrap_or_default(),
        previous_hybrid_gi_runtime: viewport_state.previous_hybrid_gi_runtime,
        previous_virtual_geometry_runtime: viewport_state.previous_virtual_geometry_runtime,
        hybrid_gi_enabled,
        virtual_geometry_enabled,
        hybrid_gi_extract: hybrid_gi_enabled
            .then(|| extract.lighting.hybrid_global_illumination.clone())
            .flatten(),
        hybrid_gi_scene_inputs,
        hybrid_gi_update_plan,
        hybrid_gi_feedback,
        virtual_geometry_extract: virtual_geometry_enabled
            .then(|| effective_virtual_geometry_extract.clone())
            .flatten(),
        virtual_geometry_cpu_reference_instances:
            synthesized_virtual_geometry_cpu_reference_instances,
        virtual_geometry_bvh_visualization_instances:
            synthesized_virtual_geometry_bvh_visualization_instances,
        virtual_geometry_page_upload_plan,
        virtual_geometry_feedback,
        predicted_generation: viewport_state.predicted_generation,
    })
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

fn compute_ui_submission_stats(extract: &UiRenderExtract) -> UiSubmissionStats {
    let mut stats = UiSubmissionStats::default();
    for command in &extract.list.commands {
        stats.command_count += 1;
        if matches!(command.kind, UiRenderCommandKind::Quad) {
            stats.quad_count += 1;
        }
        if command.text.is_some() {
            stats.text_payload_count += 1;
        }
        if command.image.is_some() {
            stats.image_payload_count += 1;
        }
        if command.clip_frame.is_some() {
            stats.clipped_command_count += 1;
        }
    }
    stats
}
