use crate::scene::viewport::{CapturedFrame, RenderViewportHandle};

use super::retained_viewport_controller::RetainedViewportController;

impl RetainedViewportController {
    pub(crate) fn poll_captured_frame(&self) -> Option<(RenderViewportHandle, CapturedFrame)> {
        let _operation = self.lock_viewport_lifecycle();
        let poll_request = {
            let mut shared = self.lock_shared();
            let Some(viewport) = shared.viewport.map(|viewport| viewport.handle) else {
                return None;
            };
            let render_framework = match shared.render_framework() {
                Ok(render_framework) => render_framework,
                Err(error) => {
                    shared.last_error = Some(error.to_string());
                    return None;
                }
            };
            (viewport, render_framework, shared.latest_generation)
        };
        let (viewport, render_framework, last_generation) = poll_request;
        match render_framework.poll_captured_frame_if_newer(viewport, last_generation) {
            Ok(Some(frame)) => {
                if let Err(error) = validate_captured_frame(&frame) {
                    self.record_viewport_error(viewport, error);
                    return None;
                }
                let mut shared = self.lock_shared();
                if shared.viewport.map(|stored| stored.handle) != Some(viewport)
                    || shared
                        .latest_generation
                        .is_some_and(|latest| latest >= frame.generation)
                {
                    return None;
                }
                shared.latest_generation = Some(frame.generation);
                shared.last_error = None;
                Some((viewport, frame))
            }
            Ok(None) => None,
            Err(error) => {
                self.record_viewport_error(viewport, error.to_string());
                None
            }
        }
    }

    fn record_viewport_error(&self, viewport: RenderViewportHandle, error: String) {
        let mut shared = self.lock_shared();
        if shared
            .viewport
            .is_some_and(|active| active.handle == viewport)
        {
            shared.last_error = Some(error);
        }
    }
}

fn validate_captured_frame(frame: &CapturedFrame) -> Result<(), String> {
    if frame.width == 0 || frame.height == 0 {
        return Err("render framework returned a zero-sized viewport frame".to_string());
    }

    let expected_len = frame.width as usize * frame.height as usize * 4;
    if frame.rgba.len() != expected_len {
        return Err(format!(
            "render framework returned {} RGBA bytes for a {}x{} frame",
            frame.rgba.len(),
            frame.width,
            frame.height
        ));
    }
    Ok(())
}
