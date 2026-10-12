use crate::scene::viewport::RenderViewportProduct;

use super::retained_viewport_controller::RetainedViewportController;

impl RetainedViewportController {
    pub(crate) fn poll_viewport_products(&self) -> Vec<(String, RenderViewportProduct)> {
        let surfaces = self
            .lock_shared()
            .viewports
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        surfaces
            .into_iter()
            .filter_map(|surface| {
                self.poll_viewport_product_for_surface(&surface)
                    .map(|product| (surface, product))
            })
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn poll_viewport_product(&self) -> Option<RenderViewportProduct> {
        self.poll_viewport_product_for_surface("editor.viewport")
    }

    fn poll_viewport_product_for_surface(
        &self,
        surface_key: &str,
    ) -> Option<RenderViewportProduct> {
        let poll_request = {
            let mut shared = self.lock_shared();
            let Some(viewport) = shared.viewports.get(surface_key).copied() else {
                return None;
            };
            let render_framework = match shared.render_framework() {
                Ok(render_framework) => render_framework,
                Err(error) => {
                    shared.last_error = Some(error.to_string());
                    return None;
                }
            };
            (
                viewport.handle,
                render_framework,
                viewport.latest_generation,
            )
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
                let Some(stored) = shared.viewports.get_mut(surface_key) else {
                    return None;
                };
                if stored.handle != viewport
                    || stored
                        .latest_generation
                        .is_some_and(|latest| latest >= product.generation())
                {
                    return None;
                }
                stored.latest_generation = Some(product.generation());
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
