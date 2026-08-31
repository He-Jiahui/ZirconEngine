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
mod performance_tests {
    use super::*;
    use crate::ui::retained_host::detail_pointer::console_scroll_layout;

    #[test]
    fn repeated_surface_size_is_a_no_op() {
        let mut surface = ScrollSurfaceHostState::new();
        let size = UiSize::new(320.0, 180.0);

        assert!(surface.set_size(size));
        assert!(!surface.set_size(size));
    }

    #[test]
    fn follow_tail_tracks_growth_until_the_user_scrolls_away() {
        let mut surface = ScrollSurfaceHostState::new();
        let size = UiSize::new(320.0, 100.0);

        surface.sync_following_tail(console_scroll_layout(size, 180.0));
        assert_eq!(surface.scroll_offset(), 80.0);

        assert!(surface.handle_scroll(UiPoint::new(24.0, 40.0), -20.0));
        assert_eq!(surface.scroll_offset(), 60.0);

        surface.sync_following_tail(console_scroll_layout(size, 220.0));
        assert_eq!(surface.scroll_offset(), 60.0);

        assert!(surface.handle_scroll(UiPoint::new(24.0, 40.0), 4096.0));
        assert_eq!(surface.scroll_offset(), 120.0);

        surface.sync_following_tail(console_scroll_layout(size, 240.0));
        assert_eq!(surface.scroll_offset(), 140.0);
    }

    #[test]
    fn sync_reads_back_the_bridge_clamp_after_content_shrinks() {
        let mut surface = ScrollSurfaceHostState::new();
        let size = UiSize::new(320.0, 100.0);
        surface.sync(console_scroll_layout(size, 180.0));
        assert!(surface.handle_scroll(UiPoint::new(24.0, 40.0), 4096.0));
        assert_eq!(surface.scroll_offset(), 80.0);

        surface.sync(console_scroll_layout(size, 40.0));
        assert_eq!(surface.scroll_offset(), 0.0);
    }
}
