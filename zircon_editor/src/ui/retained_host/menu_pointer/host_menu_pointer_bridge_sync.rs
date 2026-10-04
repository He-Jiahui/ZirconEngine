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
        let popup_semantics_changed = !self.layout.popup_semantics_equal(&layout);
        let state_changed = &self.state != state;
        let surface_state_changed = self.state.open_menu_index != state.open_menu_index
            || self.state.open_submenu_path != state.open_submenu_path
            || self.state.popup_scroll_offset != state.popup_scroll_offset
            || self.state.menu_bar_scroll_offset != state.menu_bar_scroll_offset;
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
        if popup_semantics_changed || self.popup_menu_index != self.state.open_menu_index {
            self.refresh_popup_items();
        }
        self.clamp_popup_scroll_offset();
        if layout_changed || surface_state_changed {
            self.apply_surface_delta();
        }
        true
    }
}
