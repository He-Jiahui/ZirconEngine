use crate::graphics::backend::ViewportSurface;

use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn bind_surface(
        &mut self,
        surface: ViewportSurface,
    ) {
        self.surface = Some(surface);
    }

    pub(in crate::graphics::runtime::render_framework) fn unbind_surface(&mut self) {
        self.surface = None;
    }

    pub(in crate::graphics::runtime::render_framework) fn take_surface(
        &mut self,
    ) -> Option<ViewportSurface> {
        self.surface.take()
    }
}
