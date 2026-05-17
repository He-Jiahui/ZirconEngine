use serde::{Deserialize, Serialize};

/// Current-frame file drag/drop messages forwarded from the host window backend.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileDragDropEvent {
    /// A host drag entered the window with a candidate path.
    Hovered { path: String },
    /// A host drag committed a dropped path.
    Dropped { path: String },
    /// The host drag left the window or was cancelled.
    Cancelled,
}
