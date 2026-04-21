use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::prepared_runtime_submission::PreparedRuntimeSubmission;
use super::hybrid_gi::{
    build_hybrid_gi_prepare, build_hybrid_gi_runtime, build_hybrid_gi_scene_prepare,
    collect_hybrid_gi_evictable_probe_ids,
};
use super::virtual_geometry::{
    build_virtual_geometry_prepare, build_virtual_geometry_runtime,
    collect_virtual_geometry_evictable_page_ids,
};

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn prepare_runtime_submission(
    context: &FrameSubmissionContext,
) -> PreparedRuntimeSubmission {
    let hybrid_gi_runtime = build_hybrid_gi_runtime(context);
    let hybrid_gi_prepare = build_hybrid_gi_prepare(hybrid_gi_runtime.as_ref());
    let hybrid_gi_scene_prepare = build_hybrid_gi_scene_prepare(hybrid_gi_runtime.as_ref());
    let hybrid_gi_resolve_runtime = hybrid_gi_runtime
        .as_ref()
        .map(crate::graphics::runtime::HybridGiRuntimeState::build_resolve_runtime);
    let virtual_geometry_runtime = build_virtual_geometry_runtime(context);
    let virtual_geometry_prepare = build_virtual_geometry_prepare(
        virtual_geometry_runtime.as_ref(),
        &context.visibility_context.virtual_geometry_visible_clusters,
        &context.visibility_context.virtual_geometry_draw_segments,
    );

    PreparedRuntimeSubmission {
        hybrid_gi_evictable_probe_ids: collect_hybrid_gi_evictable_probe_ids(
            hybrid_gi_prepare.as_ref(),
        ),
        hybrid_gi_prepare,
        hybrid_gi_scene_prepare,
        hybrid_gi_resolve_runtime,
        hybrid_gi_runtime,
        virtual_geometry_evictable_page_ids: collect_virtual_geometry_evictable_page_ids(
            virtual_geometry_prepare.as_ref(),
        ),
        virtual_geometry_prepare,
        virtual_geometry_runtime,
    }
}
