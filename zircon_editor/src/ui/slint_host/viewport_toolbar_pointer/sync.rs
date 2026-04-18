use std::collections::BTreeSet;

use super::viewport_toolbar_pointer_bridge::ViewportToolbarPointerBridge;
use super::viewport_toolbar_pointer_layout::ViewportToolbarPointerLayout;

impl ViewportToolbarPointerBridge {
    pub(crate) fn sync(&mut self, layout: ViewportToolbarPointerLayout) {
        self.layout = layout;
        let valid_surface_keys = self
            .layout
            .surfaces
            .iter()
            .map(|surface| surface.key.clone())
            .collect::<BTreeSet<_>>();
        self.active_controls
            .retain(|surface_key, _| valid_surface_keys.contains(surface_key));
        self.rebuild_surface();
    }
}
