use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};

use super::detail_pointer::{ScrollSurfacePointerBridge, ScrollSurfacePointerLayout};

const SCROLL_END_EPSILON_PX: f32 = 0.5;

pub(crate) struct ScrollSurfaceHostState {
    bridge: ScrollSurfacePointerBridge,
    size: UiSize,
    max_scroll_offset: f32,
}

impl ScrollSurfaceHostState {
    pub(crate) fn new() -> Self {
        Self {
            bridge: ScrollSurfacePointerBridge::new(),
            size: UiSize::new(0.0, 0.0),
            max_scroll_offset: 0.0,
        }
    }

    pub(crate) fn size(&self) -> UiSize {
        self.size
    }

    pub(crate) fn set_size(&mut self, size: UiSize) -> bool {
        let size = UiSize::new(size.width.max(0.0), size.height.max(0.0));
        if self.size == size {
            return false;
        }
        self.size = size;
        true
    }

    pub(crate) fn has_size(&self) -> bool {
        self.size.width > 0.0 && self.size.height > 0.0
    }

    pub(crate) fn sync(&mut self, layout: ScrollSurfacePointerLayout) -> bool {
        self.sync_with_tail_policy(layout, false)
    }

    pub(crate) fn sync_following_tail(&mut self, layout: ScrollSurfacePointerLayout) -> bool {
        self.sync_with_tail_policy(layout, true)
    }

    fn sync_with_tail_policy(
        &mut self,
        layout: ScrollSurfacePointerLayout,
        follow_tail: bool,
    ) -> bool {
        let next_max_scroll_offset = layout.max_scroll_offset();
        let mut state = self.bridge.state();
        let previous_offset = state.scroll_offset;
        if follow_tail
            && (state.scroll_offset - self.max_scroll_offset).abs() <= SCROLL_END_EPSILON_PX
        {
            state.scroll_offset = next_max_scroll_offset;
        }

        self.bridge.sync(layout, state);
        self.max_scroll_offset = next_max_scroll_offset;
        self.scroll_offset() != previous_offset
    }

    pub(crate) fn handle_scroll(&mut self, point: UiPoint, delta: f32) -> bool {
        let dispatch = self.bridge.handle_scroll(point, delta);
        dispatch.changed
    }

    pub(crate) fn scroll_offset(&self) -> f32 {
        self.bridge.state().scroll_offset
    }
}

#[cfg(test)]
#[path = "tests/scroll_surface_host_performance_tests.rs"]
mod performance_tests;
