use std::sync::Arc;

use super::{
    WindowPhysicalExtent, WindowPlacementRequest, WindowRequestedMode, WindowStateResizeConstraints,
};

/// Desired mutable state for one live window. It excludes focus, current
/// monitor, DPI, and other OS observations. Presentation negotiation is owned
/// by the graphics surface contract rather than this platform request.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowRequestedState {
    title: Arc<str>,
    placement: WindowPlacementRequest,
    mode: WindowRequestedMode,
    physical_extent: WindowPhysicalExtent,
    resize_constraints: WindowStateResizeConstraints,
    resizable: bool,
    decorated: bool,
    visible: bool,
}

impl WindowRequestedState {
    pub fn new(
        title: impl Into<Arc<str>>,
        placement: WindowPlacementRequest,
        mode: WindowRequestedMode,
        physical_extent: WindowPhysicalExtent,
        resize_constraints: WindowStateResizeConstraints,
        resizable: bool,
        decorated: bool,
        visible: bool,
    ) -> Self {
        Self {
            title: title.into(),
            placement,
            mode,
            physical_extent,
            resize_constraints,
            resizable,
            decorated,
            visible,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub const fn placement(&self) -> &WindowPlacementRequest {
        &self.placement
    }

    pub const fn mode(&self) -> &WindowRequestedMode {
        &self.mode
    }

    pub const fn physical_extent(&self) -> WindowPhysicalExtent {
        self.physical_extent
    }

    pub const fn resize_constraints(&self) -> WindowStateResizeConstraints {
        self.resize_constraints
    }

    pub const fn resizable(&self) -> bool {
        self.resizable
    }

    pub const fn decorated(&self) -> bool {
        self.decorated
    }

    pub const fn visible(&self) -> bool {
        self.visible
    }
}
