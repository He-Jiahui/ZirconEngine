//! Host 发布的不可变显示拓扑。窗口放置和表面租约都以稳定显示身份与发布代数校验，避免热插拔后复用旧索引。

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
#[path = "tests/cases.rs"]
mod tests;
