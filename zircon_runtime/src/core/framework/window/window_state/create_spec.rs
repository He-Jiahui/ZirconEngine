use crate::core::framework::window::DisplayTopologyGeneration;

use super::WindowRequestedState;

/// Immutable input to native-window creation. The host validates its display
/// generation before allocating a native object; later runtime changes use a
/// `WindowCommand<WindowRequestedState>` instead of mutating this record.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowCreateSpec {
    requested: WindowRequestedState,
    display_topology_generation: DisplayTopologyGeneration,
}

impl WindowCreateSpec {
    pub fn new(
        requested: WindowRequestedState,
        display_topology_generation: DisplayTopologyGeneration,
    ) -> Self {
        Self {
            requested,
            display_topology_generation,
        }
    }

    pub const fn requested(&self) -> &WindowRequestedState {
        &self.requested
    }

    pub const fn display_topology_generation(&self) -> DisplayTopologyGeneration {
        self.display_topology_generation
    }
}
