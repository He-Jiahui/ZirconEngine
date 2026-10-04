use crate::core::framework::render::RenderViewportSurfaceDescriptor;
use crate::graphics::backend::{ViewportSurface, ViewportSurfaceFrameAcquire};
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

use super::scene_renderer::SceneRenderer;
use super::scene_renderer_submission_failure::finalize_surface_presentation;

impl SceneRenderer {
    pub(in crate::graphics) fn create_framework_viewport_surface(
        &self,
        descriptor: RenderViewportSurfaceDescriptor,
    ) -> Result<ViewportSurface, GraphicsError> {
        self.backend.create_viewport_surface(descriptor)
    }

    /// 直达 surface 的呈现入口；scene receipt 已由渲染提交生成，呈现异常不得抹掉该提交。
    pub(crate) fn present_frame_direct(
        &mut self,
        frame: &ViewportRenderFrame,
        surface: &mut ViewportSurface,
    ) -> Result<u64, GraphicsError> {
        let poll_receipt = self.poll_frame_submission_completions()?;
        let acquired = surface
            .acquire_frame_target()
            .map_err(|failure| failure.into_parts().0)?;
        let (mut submission_receipt, present_result) = match acquired {
            ViewportSurfaceFrameAcquire::Acquired(surface_target) => {
                let submission_receipt = match self.render_frame_to_offscreen_target_after_poll(
                    frame,
                    poll_receipt,
                    Some((surface, &surface_target)),
                ) {
                    Ok(receipt) => receipt,
                    Err(source) => {
                        return Err(surface.discard_frame_target(surface_target, source));
                    }
                };
                let present_result = surface
                    .present_frame_target(surface_target, submission_receipt.scene_submission());
                (submission_receipt, present_result)
            }
            ViewportSurfaceFrameAcquire::NoSubmit(outcome) => (
                self.render_frame_to_offscreen_target_after_poll(frame, poll_receipt, None)?,
                Ok(outcome),
            ),
        };
        submission_receipt = finalize_surface_presentation(submission_receipt, present_result)?;
        self.last_frame_submission_receipt = Some(submission_receipt.clone());
        Ok(submission_receipt.frame_generation())
    }
}

#[cfg(test)]
#[path = "tests/scene_renderer_viewport_surface.rs"]
mod tests;
