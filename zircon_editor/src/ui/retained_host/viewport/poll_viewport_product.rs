use crate::scene::viewport::RenderViewportProduct;

use super::retained_viewport_controller::RetainedViewportController;

impl RetainedViewportController {
    pub(crate) fn poll_viewport_product(&self) -> Option<RenderViewportProduct> {
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
        match render_framework.poll_viewport_product_if_newer(viewport, last_generation) {
            Ok(Some(product)) => {
                if !product.is_valid() {
                    self.record_viewport_error(
                        viewport,
                        "render framework returned an invalid viewport GPU product".to_string(),
                    );
                    return None;
                }
                let mut shared = self.lock_shared();
                if shared.viewport.map(|stored| stored.handle) != Some(viewport)
                    || shared
                        .latest_generation
                        .is_some_and(|latest| latest >= product.generation())
                {
                    return None;
                }
                shared.latest_generation = Some(product.generation());
                shared.last_error = None;
                Some(product)
            }
            Ok(None) => None,
            Err(error) => {
                self.record_viewport_error(viewport, error.to_string());
                None
            }
        }
    }
}
