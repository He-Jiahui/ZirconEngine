use crate::ui::retained_host::primitives::Image;

use super::import_frame_image::import_frame_image;
use super::retained_viewport_controller::RetainedViewportController;

impl RetainedViewportController {
    pub(crate) fn poll_image(&self) -> Option<Image> {
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
            Ok(Some(frame)) => match import_frame_image(&frame) {
                Ok((generation, image)) => {
                    let mut shared = self.lock_shared();
                    if shared.viewport.map(|stored| stored.handle) != Some(viewport)
                        || shared
                            .latest_generation
                            .is_some_and(|latest| latest >= generation)
                    {
                        return None;
                    }
                    shared.latest_generation = Some(generation);
                    shared.latest_image = Some(image.clone());
                    shared.last_error = None;
                    Some(image)
                }
                Err(error) => {
                    self.lock_shared().last_error = Some(error);
                    None
                }
            },
            Ok(None) => None,
            Err(error) => {
                self.lock_shared().last_error = Some(error.to_string());
                None
            }
        }
    }
}
