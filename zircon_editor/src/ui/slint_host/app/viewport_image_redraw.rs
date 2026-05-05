use super::*;

impl SlintEditorHost {
    pub(super) fn poll_viewport_image_for_native_host(&mut self) {
        let Some(image) = self.viewport.poll_image() else {
            return;
        };
        let image_updated = self
            .ui
            .global::<crate::ui::slint_host::PaneSurfaceHostContext>()
            .set_viewport_image(image);
        if image_updated {
            let frame = self.ui.get_host_window_bootstrap().viewport_content_frame;
            self.record_paint_only_invalidation(HostInvalidationMask::VIEWPORT_IMAGE);
            self.ui.request_redraw_region(frame);
        }
    }
}
