use crate::graphics::backend::{
    GpuPassTimer, GpuPipelineStatisticsFrameResult, GpuPipelineStatisticsTimer,
    GpuTimerFrameResult, RenderBackend,
};
use crate::graphics::types::GraphicsError;
use zr_rhi::SubmissionPollReceipt;

use super::scene_renderer::SceneRenderer;
use super::scene_renderer_core::SceneRendererCore;
use super::scene_submission_completion_journal::SceneSubmissionCompletionJournal;

impl SceneRenderer {
    /// Advances the sole backend completion timeline, then drains feature-owned CPU deliveries.
    pub(super) fn poll_frame_submission_completions(
        &mut self,
    ) -> Result<SubmissionPollReceipt, GraphicsError> {
        let poll_receipt = self.backend.poll_submission_completions()?;
        route_frame_submission_completion_consumers(
            &self.backend,
            &mut self.core,
            &mut self.scene_submission_completion_journal,
            &mut self.gpu_pass_timer,
            &mut self.gpu_pipeline_statistics_timer,
            &mut self.last_gpu_timer_frame_result,
            &mut self.last_gpu_pipeline_statistics_frame_result,
            poll_receipt,
        )?;
        Ok(poll_receipt)
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::graphics::scene::scene_renderer::core) fn route_frame_submission_completion_consumers(
    backend: &RenderBackend,
    core: &mut SceneRendererCore,
    journal: &mut SceneSubmissionCompletionJournal,
    gpu_pass_timer: &mut Option<GpuPassTimer>,
    gpu_pipeline_statistics_timer: &mut Option<GpuPipelineStatisticsTimer>,
    last_gpu_timer_frame_result: &mut Option<GpuTimerFrameResult>,
    last_gpu_pipeline_statistics_frame_result: &mut Option<GpuPipelineStatisticsFrameResult>,
    poll_receipt: SubmissionPollReceipt,
) -> Result<(), GraphicsError> {
    journal.observe(poll_receipt, |tickets, statuses| {
        backend.append_submission_statuses(tickets, statuses);
    })?;
    core.ibl_bake_runtime_writebacks
        .poll_completed()
        .map_err(|error| GraphicsError::Asset(error.to_string()))?;
    for result in backend.drain_product_diagnostic_query_results() {
        if let Some(timer) = gpu_pass_timer.as_mut() {
            timer.accept_product_query_delivery(
                result.renderer_frame_generation,
                &result.plan,
                &result.pass_names,
                &result.delivery,
            );
        }
        if let Some(timer) = gpu_pipeline_statistics_timer.as_mut() {
            timer.accept_product_query_delivery(
                result.renderer_frame_generation,
                &result.plan,
                &result.pass_names,
                &result.delivery,
            );
        }
    }
    *last_gpu_timer_frame_result = gpu_pass_timer.as_mut().and_then(GpuPassTimer::try_collect);
    *last_gpu_pipeline_statistics_frame_result = gpu_pipeline_statistics_timer
        .as_mut()
        .and_then(GpuPipelineStatisticsTimer::try_collect);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn completion_owner_polls_once_before_routing_all_cpu_deliveries() {
        let source = include_str!("scene_renderer_completion.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("completion owner must retain a test boundary");
        let poll = production
            .find("self.backend.poll_submission_completions()")
            .expect("completion owner must poll the backend");
        let submission_journal = production
            .find("journal.observe(poll_receipt")
            .expect("scene submission journal must consume the poll receipt");
        let ibl = production
            .find(".ibl_bake_runtime_writebacks")
            .expect("IBL artifact callbacks must drain after polling");
        let typed_queries = production
            .find("backend.drain_product_diagnostic_query_results()")
            .expect("typed queries must drain after IBL artifact callbacks");
        let timer = production
            .find("GpuPassTimer::try_collect")
            .expect("timer results must collect after typed query routing");
        let statistics = production
            .find("GpuPipelineStatisticsTimer::try_collect")
            .expect("statistics must collect after typed query routing");

        assert_eq!(
            production.matches("poll_submission_completions()").count(),
            1
        );
        assert_eq!(
            production
                .matches("append_submission_statuses(tickets, statuses)")
                .count(),
            1
        );
        assert!(poll < ibl);
        assert!(poll < submission_journal);
        assert!(submission_journal < ibl);
        assert!(ibl < typed_queries);
        assert!(typed_queries < timer);
        assert!(typed_queries < statistics);
        assert!(!production.contains("GpuReadbackQueue"));
        assert!(!production.contains("device.poll("));
    }
}
