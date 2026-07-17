use std::collections::BTreeSet;

use super::viewport_toolbar_pointer_bridge::ViewportToolbarPointerBridge;
use super::viewport_toolbar_pointer_layout::ViewportToolbarPointerLayout;

impl ViewportToolbarPointerBridge {
    pub(crate) fn sync(&mut self, layout: ViewportToolbarPointerLayout) -> bool {
        if self.layout == layout {
            return false;
        }

        self.layout = layout;
        let valid_surface_keys = self
            .layout
            .surfaces
            .iter()
            .map(|surface| surface.key.as_str())
            .collect::<BTreeSet<_>>();
        self.controls_by_surface
            .retain(|surface_key, _| valid_surface_keys.contains(surface_key.as_str()));
        self.rebuild_surface();
        true
    }
}
