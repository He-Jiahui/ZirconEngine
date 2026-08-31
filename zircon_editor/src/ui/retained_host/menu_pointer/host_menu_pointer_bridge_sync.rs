use std::sync::Arc;

use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::host_menu_pointer_layout::HostMenuPointerLayout;
use super::host_menu_pointer_state::HostMenuPointerState;

impl HostMenuPointerBridge {
    pub(crate) fn sync(
        &mut self,
        layout: HostMenuPointerLayout,
        state: HostMenuPointerState,
    ) -> bool {
        self.sync_shared(Arc::new(layout), &state)
    }

    pub(crate) fn sync_shared(
        &mut self,
        layout: Arc<HostMenuPointerLayout>,
        state: &HostMenuPointerState,
    ) -> bool {
        let layout_changed =
            !Arc::ptr_eq(&self.layout, &layout) && self.layout.as_ref() != layout.as_ref();
        let state_changed = &self.state != state;
        if !layout_changed && !state_changed {
            return false;
        }

        if layout_changed {
            self.layout = layout;
        }
        if state_changed {
            self.state.clone_from(state);
        }
        self.clamp_menu_bar_scroll_offset();
        self.refresh_popup_items();
        self.clamp_popup_scroll_offset();
        self.rebuild_surface();
        true
    }
}
