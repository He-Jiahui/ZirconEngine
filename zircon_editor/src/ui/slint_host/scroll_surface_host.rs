use zircon_ui::{UiPoint, UiSize};

use super::detail_pointer::{
    ScrollSurfacePointerBridge, ScrollSurfacePointerLayout, ScrollSurfacePointerState,
};

pub(crate) struct ScrollSurfaceHostState {
    bridge: ScrollSurfacePointerBridge,
    state: ScrollSurfacePointerState,
    size: UiSize,
}

impl ScrollSurfaceHostState {
    pub(crate) fn new(tree_id: &'static str, path_prefix: &'static str) -> Self {
        Self {
            bridge: ScrollSurfacePointerBridge::new(tree_id, path_prefix),
            state: ScrollSurfacePointerState::default(),
            size: UiSize::new(0.0, 0.0),
        }
    }

    pub(crate) fn size(&self) -> UiSize {
        self.size
    }

    pub(crate) fn set_size(&mut self, size: UiSize) {
        self.size = UiSize::new(size.width.max(0.0), size.height.max(0.0));
    }

    pub(crate) fn has_size(&self) -> bool {
        self.size.width > 0.0 && self.size.height > 0.0
    }

    pub(crate) fn sync(&mut self, layout: ScrollSurfacePointerLayout) {
        self.bridge.sync(layout, self.state.clone());
    }

    pub(crate) fn handle_scroll(&mut self, point: UiPoint, delta: f32) -> Result<(), String> {
        let dispatch = self.bridge.handle_scroll(point, delta)?;
        self.state = dispatch.state;
        Ok(())
    }

    pub(crate) fn scroll_offset(&self) -> f32 {
        self.state.scroll_offset
    }
}
