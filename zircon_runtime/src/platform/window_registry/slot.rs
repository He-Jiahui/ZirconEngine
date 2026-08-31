use std::num::NonZeroU32;

use crate::core::framework::window::{NativeWindowId, WindowId};
use zircon_runtime_interface::ZrRuntimeViewportHandle;

use super::relationship::WindowParentRelation;

pub(super) struct WindowRegistrySlot {
    pub(super) generation: NonZeroU32,
    pub(super) native_window: Option<NativeWindowId>,
    pub(super) closing: bool,
    pub(super) parent: Option<WindowParentRelation>,
    pub(super) children: Vec<WindowId>,
    pub(super) viewports: Vec<ZrRuntimeViewportHandle>,
}

impl Default for WindowRegistrySlot {
    fn default() -> Self {
        Self {
            generation: NonZeroU32::MIN,
            native_window: None,
            closing: false,
            parent: None,
            children: Vec::new(),
            viewports: Vec::new(),
        }
    }
}
