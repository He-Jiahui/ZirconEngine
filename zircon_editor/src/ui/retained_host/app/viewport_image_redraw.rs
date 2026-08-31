use super::*;

impl RetainedEditorHost {
    pub(super) fn poll_viewport_image_for_native_host(&mut self) {
        let image_updated = if self.ui.direct_viewport_products_active() {
            let Some(product) = self.viewport.poll_viewport_product() else {
                return;
            };
            self.ui
                .global::<crate::ui::retained_host::PaneSurfaceHostContext>()
                .set_scene_viewport_product(product)
        } else {
            let Some((viewport, frame)) = self.viewport.poll_captured_frame() else {
                return;
            };
            self.ui
                .global::<crate::ui::retained_host::PaneSurfaceHostContext>()
                .set_scene_viewport_capture(viewport, frame)
        };
        zircon_runtime::profile_scope!("editor", "retained_host", "poll_viewport_image");
        if image_updated {
            let frame = self.ui.get_host_window_bootstrap().viewport_content_frame;
            self.record_paint_only_invalidation(HostInvalidationMask::VIEWPORT_IMAGE);
            self.ui.request_redraw_region(frame);
        }
    }
}
