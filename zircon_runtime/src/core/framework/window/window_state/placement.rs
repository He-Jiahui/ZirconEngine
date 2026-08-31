use crate::core::framework::window::DisplayId;

use super::WindowLogicalPosition;

/// Stable output targeting for creation and fullscreen requests. Current and
/// topology-local monitor indices are deliberately absent: callers either use
/// the primary output or an observed stable `DisplayId`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowDisplayTarget {
    Primary,
    Display(DisplayId),
}

/// Requested initial or runtime placement. Centering is resolved by the host
/// against the target display's observed logical usable rectangle and safe
/// area, never a physical video-mode extent.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowPlacementRequest {
    Automatic,
    CenteredOn(WindowDisplayTarget),
    AtLogical(WindowLogicalPosition),
}
