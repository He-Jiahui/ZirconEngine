use crate::ui::retained_host::primitives::Image;

use super::import_frame_image::import_frame_image;
use super::retained_viewport_controller::RetainedViewportController;

impl RetainedViewportController {
    pub(crate) fn poll_image(&self) -> Option<Image> {
        let mut shared = self.lock_shared();
        let Some(viewport) = shared.viewport.map(|viewport| viewport.handle) else {
            return shared.latest_image.clone();
        };
        let render_framework = match shared.render_framework() {
            Ok(render_framework) => render_framework,
            Err(error) => {
                shared.last_error = Some(error.to_string());
                return shared.latest_image.clone();
            }
        };
        match render_framework.capture_frame(viewport) {
            Ok(Some(frame)) => {
                if shared.latest_generation == Some(frame.generation) {
                    return None;
                }
                match import_frame_image(&frame) {
                    Ok(image) => {
                        shared.latest_generation = Some(image.0);
                        shared.latest_image = Some(image.1.clone());
                        shared.last_error = None;
                        shared.latest_image.clone()
                    }
                    Err(error) => {
                        shared.last_error = Some(error);
                        shared.latest_image.clone()
                    }
                }
            }
            Ok(None) => None,
            Err(error) => {
                shared.last_error = Some(error.to_string());
                shared.latest_image.clone()
            }
        }
    }
}
