use std::sync::Arc;

use crate::core::framework::window::DisplayTopologyGeneration;

use super::super::{WindowPhysicalExtent, WindowStateResizeConstraints};
use super::{WindowEffectiveMode, WindowEffectivePlacement, WindowEffectiveStateError};

/// Host-accepted window configuration published in a terminal command
/// receipt. It excludes focus and occlusion, which remain OS observations.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowEffectiveState {
    title: Arc<str>,
    placement: WindowEffectivePlacement,
    mode: WindowEffectiveMode,
    physical_extent: WindowPhysicalExtent,
    resize_constraints: WindowStateResizeConstraints,
    resizable: bool,
    decorated: bool,
    visible: bool,
    display_topology_generation: DisplayTopologyGeneration,
}

impl WindowEffectiveState {
    pub fn new(
        title: impl Into<Arc<str>>,
        placement: WindowEffectivePlacement,
        mode: WindowEffectiveMode,
        physical_extent: WindowPhysicalExtent,
        resize_constraints: WindowStateResizeConstraints,
        resizable: bool,
        decorated: bool,
        visible: bool,
        display_topology_generation: DisplayTopologyGeneration,
    ) -> Result<Self, WindowEffectiveStateError> {
        if let Some(mode_output) = mode.output() {
            if mode_output != placement.display() {
                return Err(WindowEffectiveStateError::FullscreenOutputMismatch {
                    placement_display: placement.display().clone(),
                    mode_output: mode_output.clone(),
                });
            }
        }

        Ok(Self {
            title: title.into(),
            placement,
            mode,
            physical_extent,
            resize_constraints,
            resizable,
            decorated,
            visible,
            display_topology_generation,
        })
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub const fn placement(&self) -> &WindowEffectivePlacement {
        &self.placement
    }

    pub const fn mode(&self) -> &WindowEffectiveMode {
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

    pub const fn display_topology_generation(&self) -> DisplayTopologyGeneration {
        self.display_topology_generation
    }
}
