use super::viewport_toolbar_pointer_bridge::ViewportToolbarPointerBridge;
use super::viewport_toolbar_pointer_surface::ViewportToolbarPointerSurface;

impl ViewportToolbarPointerBridge {
    pub(super) fn surface_layout(
        &self,
        surface_key: &str,
    ) -> Option<&ViewportToolbarPointerSurface> {
        self.layout
            .surfaces
            .iter()
            .find(|surface| surface.key == surface_key)
    }
}
