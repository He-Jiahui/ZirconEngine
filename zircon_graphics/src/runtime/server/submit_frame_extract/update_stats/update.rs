use super::super::super::render_server_state::RenderServerState;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::submission_record_update::SubmissionRecordUpdate;
use super::base_stats::update_base_stats;
use super::hybrid_gi_stats::{reset_hybrid_gi_stats, update_hybrid_gi_stats};
use super::quality_profile::update_quality_profile;
use super::virtual_geometry_stats::{reset_virtual_geometry_stats, update_virtual_geometry_stats};

pub(in crate::runtime::server::submit_frame_extract) fn update_stats(
    state: &mut RenderServerState,
    context: &FrameSubmissionContext,
    record_update: &SubmissionRecordUpdate,
    frame_generation: u64,
) {
    update_base_stats(state, context, record_update, frame_generation);

    if context.hybrid_gi_enabled {
        update_hybrid_gi_stats(state, context, record_update);
    } else {
        reset_hybrid_gi_stats(state);
    }

    if context.virtual_geometry_enabled {
        update_virtual_geometry_stats(state, context, record_update);
    } else {
        reset_virtual_geometry_stats(state);
    }

    update_quality_profile(state, context);
}
