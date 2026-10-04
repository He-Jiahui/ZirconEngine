use crate::graphics::backend::{
    GpuPassTimer, GpuPipelineStatisticsFrameResult, GpuPipelineStatisticsTimer,
    GpuTimerFrameResult, RenderBackend,
};
use crate::graphics::types::GraphicsError;
use zr_rhi::SubmissionPollReceipt;

use super::scene_renderer::SceneRenderer;
use super::scene_renderer_core::SceneRendererCore;
use super::scene_submission_completion_journal::SceneSubmissionCompletionJournal;
use crate::graphics::scene::resources::ResourceStreamer;

impl SceneRenderer {
    /// Advances the sole backend completion timeline, then drains feature-owned CPU deliveries.
    pub(super) fn poll_frame_submission_completions(
        &mut self,
    ) -> Result<SubmissionPollReceipt, GraphicsError> {
        let poll_receipt = self.backend.poll_submission_completions()?;
        route_frame_submission_completion_consumers(
            &self.backend,
            &mut self.core,
            &mut self.streamer,
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
    streamer: &mut ResourceStreamer,
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
    streamer.maintain_render_asset_gpu_residency_after_rhi_poll(backend, poll_receipt);
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
#[path = "tests/scene_renderer_completion.rs"]
mod tests;
