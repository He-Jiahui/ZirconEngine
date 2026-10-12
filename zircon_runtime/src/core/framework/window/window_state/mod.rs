//! 运行期窗口状态按创建意图、最新请求、OS 观测和 Host 生效结果分别发布。
//! 各自代数独立推进，调用方须根据回执来源判断状态，不能把请求当成已经生效。

#[path = "constraints.rs"]
mod constraints;
#[path = "create_spec.rs"]
mod create_spec;
#[path = "effective/mod.rs"]
mod effective;
#[path = "error.rs"]
mod error;
#[path = "generation.rs"]
mod generation;
#[path = "geometry.rs"]
mod geometry;
#[path = "mode.rs"]
mod mode;
#[path = "observed.rs"]
mod observed;
#[path = "placement.rs"]
mod placement;
#[path = "reconciliation.rs"]
mod reconciliation;
#[path = "requested.rs"]
mod requested;
#[path = "snapshot.rs"]
mod snapshot;

// Requested creation and constraints.
pub use constraints::WindowStateResizeConstraints;
pub use create_spec::WindowCreateSpec;
pub use mode::{
    WindowExclusiveFullscreenRequest, WindowFullscreenFallback, WindowRequestedMode,
    WindowVideoModeRequest,
};
pub use placement::{WindowDisplayTarget, WindowPlacementRequest};
pub use requested::WindowRequestedState;

// Shared validation, geometry, and independent generations.
pub use error::{WindowStateField, WindowStateValidationError};
pub use generation::{
    WindowCreateGeneration, WindowEffectiveGeneration, WindowRequestedGeneration,
};
pub use geometry::{WindowLogicalExtent, WindowLogicalPosition, WindowPhysicalExtent};

// Host-observed and host-effective state authorities.
pub use effective::{
    WindowEffectiveMode, WindowEffectivePlacement, WindowEffectiveState, WindowEffectiveStateError,
    WindowExclusiveFullscreenFallback, WindowExclusiveFullscreenFallbackReason,
};
pub use observed::{
    WindowFocusState, WindowObservedMode, WindowObservedState, WindowOcclusionState,
    WindowVisibilityState,
};

// External changes and immutable state projections.
pub use reconciliation::{
    WindowExternalStatePolicy, WindowReconciliationAction, WindowReconciliationPolicy,
    WindowStateReconciliation,
};
pub use snapshot::{
    WindowCreateSnapshot, WindowEffectiveSnapshot, WindowObservedSnapshot, WindowRequestedSnapshot,
    WindowStateSnapshot,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
