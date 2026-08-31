use crate::core::TaskPool;
use crate::core::framework::render::{
    FrameHistoryHandle, RenderCapabilitySummary, RenderFrameHistoryInput,
};
use crate::graphics::backend::{ViewportSurface, ViewportSurfaceFrameAcquire};
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::graphics::{CompiledRenderPipeline, EnvironmentIblBakeReservation};

use super::super::super::scene_renderer::SceneRenderer;
use super::super::super::scene_renderer_submission_failure::finalize_surface_presentation;

impl SceneRenderer {
    pub(crate) fn present_frame_with_pipeline(
        &mut self,
        frame: &ViewportRenderFrame,
        pipeline: &CompiledRenderPipeline,
        capabilities: &RenderCapabilitySummary,
        history_handle: Option<FrameHistoryHandle>,
        previous_history_available: bool,
        surface: &mut ViewportSurface,
    ) -> Result<u64, GraphicsError> {
        self.present_frame_with_pipeline_optional_task_pool(
            frame,
            pipeline,
            capabilities,
            RenderFrameHistoryInput::new(history_handle, previous_history_available, None),
            surface,
            None,
            None,
        )
    }

    pub(crate) fn present_frame_with_pipeline_task_pool(
        &mut self,
        frame: &ViewportRenderFrame,
        pipeline: &CompiledRenderPipeline,
        capabilities: &RenderCapabilitySummary,
        history_handle: Option<FrameHistoryHandle>,
        previous_history_available: bool,
        surface: &mut ViewportSurface,
        task_pool: &TaskPool,
    ) -> Result<u64, GraphicsError> {
        self.present_frame_with_pipeline_task_pool_with_environment_ibl_bake_reservation(
            frame,
            pipeline,
            capabilities,
            RenderFrameHistoryInput::new(history_handle, previous_history_available, None),
            surface,
            task_pool,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn present_frame_with_pipeline_task_pool_with_environment_ibl_bake_reservation(
        &mut self,
        frame: &ViewportRenderFrame,
        pipeline: &CompiledRenderPipeline,
        capabilities: &RenderCapabilitySummary,
        history_input: RenderFrameHistoryInput,
        surface: &mut ViewportSurface,
        task_pool: &TaskPool,
        environment_ibl_bake_reservation: Option<EnvironmentIblBakeReservation>,
    ) -> Result<u64, GraphicsError> {
        self.present_frame_with_pipeline_optional_task_pool(
            frame,
            pipeline,
            capabilities,
            history_input,
            surface,
            Some(task_pool),
            environment_ibl_bake_reservation,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn present_frame_with_pipeline_optional_task_pool(
        &mut self,
        frame: &ViewportRenderFrame,
        pipeline: &CompiledRenderPipeline,
        capabilities: &RenderCapabilitySummary,
        history_input: RenderFrameHistoryInput,
        surface: &mut ViewportSurface,
        task_pool: Option<&TaskPool>,
        environment_ibl_bake_reservation: Option<EnvironmentIblBakeReservation>,
    ) -> Result<u64, GraphicsError> {
        let poll_receipt = self.poll_frame_submission_completions()?;
        let acquired = surface
            .acquire_frame_target()
            .map_err(|failure| failure.into_parts().0)?;
        let (mut submission_receipt, present_result) = match acquired {
            ViewportSurfaceFrameAcquire::Acquired(surface_target) => {
                let submission_receipt = match self.render_frame_with_pipeline_to_target(
                    frame,
                    pipeline,
                    capabilities,
                    history_input,
                    task_pool,
                    None,
                    environment_ibl_bake_reservation,
                    false,
                    Some((surface, &surface_target)),
                    Some(poll_receipt),
                ) {
                    Ok((receipt, _, _)) => receipt,
                    Err(source) => {
                        return Err(surface.discard_frame_target(surface_target, source));
                    }
                };
                let present_result = surface
                    .present_frame_target(surface_target, submission_receipt.scene_submission());
                (submission_receipt, present_result)
            }
            ViewportSurfaceFrameAcquire::NoSubmit(outcome) => {
                let (submission_receipt, _, _) = self.render_frame_with_pipeline_to_target(
                    frame,
                    pipeline,
                    capabilities,
                    history_input,
                    task_pool,
                    None,
                    environment_ibl_bake_reservation,
                    false,
                    None,
                    Some(poll_receipt),
                )?;
                (submission_receipt, Ok(outcome))
            }
        };
        submission_receipt = finalize_surface_presentation(submission_receipt, present_result)?;
        self.last_frame_submission_receipt = Some(submission_receipt.clone());
        Ok(submission_receipt.frame_generation())
    }
}
