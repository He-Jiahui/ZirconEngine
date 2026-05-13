use super::hierarchy_pointer_bridge::HierarchyPointerBridge;
use super::hierarchy_pointer_layout::HierarchyPointerLayout;
use super::hierarchy_pointer_state::HierarchyPointerState;

impl HierarchyPointerBridge {
    pub(crate) fn sync(
        &mut self,
        layout: HierarchyPointerLayout,
        state: HierarchyPointerState,
    ) -> bool {
        if self.layout == layout && self.state == state {
            return false;
        }

        self.layout = layout;
        self.state = state;
        self.clamp_scroll_offset();
        self.rebuild_surface();
        true
    }
}
