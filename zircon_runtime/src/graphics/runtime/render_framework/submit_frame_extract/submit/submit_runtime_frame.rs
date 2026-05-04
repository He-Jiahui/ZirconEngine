use crate::core::framework::render::{RenderFrameworkError, RenderViewportHandle};

use crate::graphics::types::ViewportRenderFrame;

use super::super::super::render_framework_backend_error::render_framework_backend_error;
use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::super::build_frame_submission_context::build_frame_submission_context;
use super::super::prepare_runtime_submission::prepare_runtime_submission;
use super::super::record_submission::record_submission;
use super::super::update_stats::update_stats;
use super::collect_runtime_feedback::collect_runtime_feedback;
use super::release_previous_history::release_previous_history;
use super::resolve_history_handle::resolve_history_handle;

pub(in crate::graphics::runtime::render_framework) fn submit_runtime_frame(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    mut frame: ViewportRenderFrame,
) -> Result<(), RenderFrameworkError> {
    let context =
        build_frame_submission_context(server, viewport, &frame.extract, frame.ui.as_ref())?;
    apply_effective_advanced_extracts_to_runtime_frame(&mut frame, &context);
    let mut state = server.state.lock().unwrap();
    let prepared = prepare_runtime_submission(&mut state, viewport, &context);
    let resolved_history = resolve_history_handle(&mut state, viewport, &context);
    state.last_virtual_geometry_debug_snapshot = frame.virtual_geometry_debug_snapshot.clone();
    let frame = attach_prepared_sidebands_to_runtime_frame(frame, &prepared);
    let frame = state
        .renderer
        .render_frame_with_pipeline(
            &frame,
            context.compiled_pipeline(),
            resolved_history.current_history_handle(),
        )
        .map_err(render_framework_backend_error)?;
    let frame_generation = frame.generation;
    let runtime_feedback = collect_runtime_feedback(&mut state.renderer, &context, &prepared);
    let record = state
        .viewports
        .get_mut(&viewport)
        .expect("viewport checked above");
    let record_update = record_submission(
        record,
        &context,
        prepared,
        resolved_history.allocated_history(),
        frame,
        runtime_feedback,
    );
    release_previous_history(&mut state.renderer, &record_update);
    update_stats(&mut state, &context, &record_update, frame_generation);
    Ok(())
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
        FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderOverlayExtract,
        RenderPluginRendererOutputs, RenderSceneGeometryExtract, RenderSceneSnapshot,
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

    fn empty_scene_snapshot() -> RenderSceneSnapshot {
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
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
}
