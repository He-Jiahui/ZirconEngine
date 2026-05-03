use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::prepared_runtime_submission::PreparedRuntimeSubmission;
use crate::core::framework::render::RenderPluginRendererOutputs;
use crate::{HybridGiRuntimePrepareInput, VirtualGeometryRuntimePrepareInput};

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn prepare_runtime_submission(
    state: &mut RenderFrameworkState,
    viewport: crate::core::framework::render::RenderViewportHandle,
    context: &FrameSubmissionContext,
) -> PreparedRuntimeSubmission {
    let (hybrid_gi_evictable_probe_ids, hybrid_gi_renderer_outputs) =
        prepare_hybrid_gi_runtime(state, viewport, context)
            .map(crate::HybridGiRuntimePrepareOutput::into_parts)
            .unwrap_or_default();
    let (virtual_geometry_evictable_page_ids, virtual_geometry_renderer_outputs) =
        prepare_virtual_geometry_runtime(state, viewport, context)
            .map(crate::VirtualGeometryRuntimePrepareOutput::into_parts)
            .unwrap_or_default();
    let plugin_renderer_outputs = merge_prepare_plugin_renderer_outputs(
        hybrid_gi_renderer_outputs,
        virtual_geometry_renderer_outputs,
    );

    PreparedRuntimeSubmission::new(
        hybrid_gi_evictable_probe_ids,
        virtual_geometry_evictable_page_ids,
        plugin_renderer_outputs,
    )
}

fn merge_prepare_plugin_renderer_outputs(
    hybrid_gi_outputs: RenderPluginRendererOutputs,
    virtual_geometry_outputs: RenderPluginRendererOutputs,
) -> RenderPluginRendererOutputs {
    RenderPluginRendererOutputs {
        hybrid_gi: hybrid_gi_outputs.hybrid_gi,
        virtual_geometry: virtual_geometry_outputs.virtual_geometry,
        ..RenderPluginRendererOutputs::default()
    }
}

fn prepare_hybrid_gi_runtime(
    state: &mut RenderFrameworkState,
    viewport: crate::core::framework::render::RenderViewportHandle,
    context: &FrameSubmissionContext,
) -> Option<crate::HybridGiRuntimePrepareOutput> {
    if !context.hybrid_gi_enabled() {
        if let Some(record) = state.viewports.get_mut(&viewport) {
            record.clear_hybrid_gi_runtime();
        }
        return None;
    }

    let Some(registration) = state.hybrid_gi_runtime_provider.clone() else {
        if let Some(record) = state.viewports.get_mut(&viewport) {
            record.clear_hybrid_gi_runtime();
        }
        return None;
    };
    let record = state
        .viewports
        .get_mut(&viewport)
        .expect("viewport checked while building frame submission context");
    let input = HybridGiRuntimePrepareInput::new(
        context.hybrid_gi_extract(),
        context.scene_meshes(),
        context.scene_directional_lights(),
        context.scene_point_lights(),
        context.scene_spot_lights(),
        context.hybrid_gi_update_plan(),
        context.predicted_generation(),
    );
    Some(
        record
            .ensure_hybrid_gi_runtime(registration.provider())
            .prepare_frame(input),
    )
}

fn prepare_virtual_geometry_runtime(
    state: &mut RenderFrameworkState,
    viewport: crate::core::framework::render::RenderViewportHandle,
    context: &FrameSubmissionContext,
) -> Option<crate::VirtualGeometryRuntimePrepareOutput> {
    if !context.virtual_geometry_enabled() {
        if let Some(record) = state.viewports.get_mut(&viewport) {
            record.clear_virtual_geometry_runtime();
        }
        return None;
    }

    let Some(registration) = state.virtual_geometry_runtime_provider.clone() else {
        if let Some(record) = state.viewports.get_mut(&viewport) {
            record.clear_virtual_geometry_runtime();
        }
        return None;
    };
    let record = state
        .viewports
        .get_mut(&viewport)
        .expect("viewport checked while building frame submission context");
    let visibility_context = context.visibility_context();
    let input = VirtualGeometryRuntimePrepareInput::new(
        context.virtual_geometry_extract(),
        context.virtual_geometry_page_upload_plan(),
        &visibility_context.virtual_geometry_visible_clusters,
        &visibility_context.virtual_geometry_draw_segments,
        context.predicted_generation(),
    );
    Some(
        record
            .ensure_virtual_geometry_runtime(registration.provider())
            .prepare_frame(input),
    )
}

#[cfg(test)]
mod tests {
    use super::merge_prepare_plugin_renderer_outputs;
    use crate::core::framework::render::{
        RenderHybridGiReadbackOutputs, RenderParticleGpuReadbackOutputs,
        RenderPluginRendererOutputs, RenderVirtualGeometryNodeClusterCullReadbackOutputs,
        RenderVirtualGeometryReadbackOutputs,
    };

    #[test]
    fn prepare_merge_keeps_only_hybrid_gi_and_virtual_geometry_sidebands() {
        let merged = merge_prepare_plugin_renderer_outputs(
            RenderPluginRendererOutputs {
                hybrid_gi: RenderHybridGiReadbackOutputs {
                    completed_probe_ids: vec![11],
                    ..RenderHybridGiReadbackOutputs::default()
                },
                particles: RenderParticleGpuReadbackOutputs {
                    alive_count: 5,
                    ..RenderParticleGpuReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            },
            RenderPluginRendererOutputs {
                virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                    node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                        page_request_ids: vec![300],
                        ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
                    },
                    ..RenderVirtualGeometryReadbackOutputs::default()
                },
                particles: RenderParticleGpuReadbackOutputs {
                    alive_count: 7,
                    ..RenderParticleGpuReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            },
        );

        assert_eq!(merged.hybrid_gi.completed_probe_ids, vec![11]);
        assert_eq!(
            merged.virtual_geometry.node_cluster_cull.page_request_ids,
            vec![300]
        );
        assert!(merged.particles.is_empty());
    }
}
