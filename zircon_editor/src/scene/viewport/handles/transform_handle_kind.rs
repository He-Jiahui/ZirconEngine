use serde::{Deserialize, Serialize};

/// The concrete handle family used while the transform scene mode is active.
///
/// This deliberately does not represent scene selection or mode activation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformHandleKind {
    Move,
    Rotate,
    Scale,
}
