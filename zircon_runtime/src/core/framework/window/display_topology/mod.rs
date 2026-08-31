mod capabilities;
mod display_id;
mod error;
mod geometry;
mod replacement;
mod snapshot;

pub use capabilities::{DisplayColorSpace, DisplayFeatureState, DisplayOutputCapabilities};
pub use display_id::{DisplayId, DisplayIdentityError, DisplayKind};
pub use error::DisplayTopologyError;
pub use geometry::{
    DisplayLogicalInsets, DisplayLogicalRect, DisplayOrientation, DisplayPhysicalRect,
};
pub use replacement::{DisplayTopologyReplacement, DisplayTopologyReplacementError};
pub use snapshot::{
    DisplayObservation, DisplaySnapshot, DisplayTopologyGeneration, DisplayTopologySnapshot,
};

#[cfg(test)]
mod tests;
