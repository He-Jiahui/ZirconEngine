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
mod tests {
    #[test]
    fn direct_surface_errors_retain_the_scene_submission_receipt() {
        let source = include_str!("scene_renderer_viewport_surface.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("direct surface test boundary");

        assert!(source.contains("surface.acquire_frame_target()"));
        assert!(source.contains("let poll_receipt = self.poll_frame_submission_completions()?"));
        assert!(source.contains("self.render_frame_to_offscreen_target_after_poll("));
        assert!(source.contains("surface.present_frame_target("));
        assert!(source.contains("surface.discard_frame_target(surface_target, source)"));
        assert!(source.contains("finalize_surface_presentation("));
        assert!(!source.contains("surface.present_texture("));
    }
}
