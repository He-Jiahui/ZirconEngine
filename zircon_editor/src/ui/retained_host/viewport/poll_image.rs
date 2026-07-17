use crate::ui::retained_host::primitives::Image;

use super::import_frame_image::import_frame_image;
use super::retained_viewport_controller::RetainedViewportController;

impl RetainedViewportController {
    pub(crate) fn poll_image(&self) -> Option<Image> {
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
        match render_framework.capture_frame(viewport) {
            Ok(Some(frame)) => {
                if shared.latest_generation == Some(frame.generation) {
                    return None;
                }
                match import_frame_image(&frame) {
                    Ok((generation, image)) => {
                        shared.latest_generation = Some(generation);
                        shared.latest_image = Some(image.clone());
                        shared.last_error = None;
                        Some(image)
                    }
                    Err(error) => {
                        shared.last_error = Some(error);
                        None
                    }
                }
            }
            Ok(None) => None,
            Err(error) => {
                shared.last_error = Some(error.to_string());
                None
            }
        }
    }
}
