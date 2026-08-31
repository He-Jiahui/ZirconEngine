use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionRecord;
use crate::graphics::scene::scene_renderer::hzb::HzbOcclusionCuller;
use crate::graphics::visibility::{
    HzbOcclusionCullReadbackStats, HzbOcclusionIndirectArgsReadbackSummary,
};

pub(super) fn attach_hzb_occlusion_readback_stats(
    culler: &HzbOcclusionCuller,
    current_frame_index: Option<u64>,
    graph_execution_record: &mut RenderGraphExecutionRecord,
) {
    let Some(report) = graph_execution_record.hzb_occlusion_cull_report() else {
        return;
    };
    let mut report = if report.dispatched_phase_count == 0 {
        report
            .with_readback_stats(HzbOcclusionCullReadbackStats::default())
            .with_indirect_args_readback(HzbOcclusionIndirectArgsReadbackSummary::default())
    } else if let Some((source_frame_index, readback_stats)) = culler.collect_last_readback_stats()
    {
        report
            .with_readback_stats(readback_stats)
            .with_readback_stats_source_frame_index(source_frame_index)
    } else {
        report
    };
    if report.dispatched_phase_count > 0 {
        if let Some((source_frame_index, summary)) = culler.collect_last_indirect_args_summary() {
            report = report
                .with_indirect_args_readback(summary)
                .with_indirect_args_readback_source_frame_index(source_frame_index);
        }
    }
    let report = culler.with_readback_queue_diagnostics(report, current_frame_index);
    graph_execution_record.set_hzb_occlusion_cull_report(report);
}
