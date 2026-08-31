use crate::core::framework::window::DisplayId;

use super::super::WindowLogicalPosition;

/// The exact display and desktop placement accepted by the host for a window
/// command. This is distinct from placement intent and from a later OS event
/// reporting an externally moved window.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowEffectivePlacement {
    display: DisplayId,
    logical_position: WindowLogicalPosition,
}

impl WindowEffectivePlacement {
    pub fn new(display: DisplayId, logical_position: WindowLogicalPosition) -> Self {
        Self {
            display,
            logical_position,
        }
    }

    pub const fn display(&self) -> &DisplayId {
        &self.display
    }

    pub const fn logical_position(&self) -> WindowLogicalPosition {
        self.logical_position
    }
}
