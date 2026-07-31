use super::super::super::frame_profiler::FrameProfileWrite;
use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::submission_record_update::SubmissionRecordUpdate;
use super::base_stats::update_base_stats;
use super::hybrid_gi_stats::{reset_hybrid_gi_stats, update_hybrid_gi_stats};
use super::particle_stats::update_particle_stats;
use super::quality_profile::update_quality_profile;
use super::shared_product_reports::SharedViewportProductReports;
use super::virtual_geometry_stats::{reset_virtual_geometry_stats, update_virtual_geometry_stats};
pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn update_stats(
    state: &mut RenderFrameworkState,
    context: &FrameSubmissionContext,
    record_update: &SubmissionRecordUpdate,
    frame_generation: u64,
    cpu_submit_time_us: u64,
    shared_product_reports: SharedViewportProductReports,
) -> FrameProfileWrite {
    update_base_stats(
        state,
        context,
        record_update,
        frame_generation,
        shared_product_reports,
    );
    update_particle_stats(state, record_update);

    if context.hybrid_gi_enabled() {
        update_hybrid_gi_stats(state, context, record_update);
    } else {
        reset_hybrid_gi_stats(state);
    }

    if context.virtual_geometry_enabled() {
        update_virtual_geometry_stats(state, context, record_update);
    } else {
        reset_virtual_geometry_stats(state);
    }

    update_quality_profile(state, context);
    let gpu_timer_frame_result = state.renderer.last_gpu_timer_frame_result().cloned();
    state.frame_profiler.write_frame_profile(
        &mut state.stats,
        frame_generation,
        cpu_submit_time_us,
        gpu_timer_frame_result.as_ref(),
    )
}
