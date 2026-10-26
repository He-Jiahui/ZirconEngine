use super::viewport::scene_viewport_surface_damage_frame;
use super::*;

impl RetainedEditorHost {
    pub(super) fn poll_viewport_image_for_native_host(&mut self) {
        let main_viewport_generation = self
            .ui
            .get_host_presentation_generation()
            .viewport_generation();
        let mut image_updated = false;
        if self.ui.direct_viewport_products_active() {
            for (surface_key, product) in self.viewport.poll_viewport_products() {
                let target = self.viewport_product_target(&surface_key);
                if publish_viewport_product(&target, &surface_key, product) {
                    image_updated = true;
                }
            }
        } else {
            for (surface_key, viewport, frame) in self.viewport.poll_captured_frames() {
                let target = self.viewport_product_target(&surface_key);
                if target
                    .global::<crate::ui::retained_host::PaneSurfaceHostContext>()
                    .set_scene_viewport_capture_for_surface(&surface_key, viewport, frame)
                {
                    target.request_redraw_region(scene_viewport_surface_damage_frame(
                        &target,
                        &surface_key,
                    ));
                    image_updated = true;
                }
            }
        }
        self.complete_viewport_image_update(main_viewport_generation, image_updated);
    }

    fn complete_viewport_image_update(
        &mut self,
        main_viewport_generation: u64,
        image_updated: bool,
    ) {
        if !image_updated {
            return;
        }
        zircon_runtime::profile_scope!("editor", "retained_host", "poll_viewport_image");
        self.record_paint_only_invalidation(HostInvalidationMask::VIEWPORT_IMAGE);
        // Image delivery can advance the main cursor after its source commit.
        // Child-only delivery must not republish unchanged main-window metadata.
        if self
            .ui
            .get_host_presentation_generation()
            .viewport_generation()
            != main_viewport_generation
            && !self
                .workbench_window_bridge
                .has_pending_surface_state_change()
            && !self
                .workbench_window_bridge
                .has_pending_host_projection_commit()
        {
            self.publish_current_normal_source_capture_metadata();
        }
    }

    fn viewport_product_target(&self, surface_key: &str) -> UiHostWindow {
        self.native_window_presenters
            .window(&MainPageId::new(surface_key))
            .unwrap_or_else(|| self.ui.clone_strong())
    }
}

fn publish_viewport_product(
    target: &UiHostWindow,
    surface_key: &str,
    product: crate::scene::viewport::RenderViewportProduct,
) -> bool {
    if !target
        .global::<crate::ui::retained_host::PaneSurfaceHostContext>()
        .set_scene_viewport_product_for_surface(surface_key, product)
    {
        return false;
    }
    target.request_redraw_region(scene_viewport_surface_damage_frame(target, surface_key));
    true
}

#[cfg(test)]
#[path = "tests/viewport_image_redraw.rs"]
mod tests;
