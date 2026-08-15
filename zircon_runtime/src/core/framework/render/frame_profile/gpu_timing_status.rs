use serde::{Deserialize, Serialize};

/// Explains whether a frame's missing GPU duration is expected or comparable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderGpuTimingStatus {
    #[default]
    Disabled,
    Unavailable,
    Pending,
    Deferred,
    CapacityExhausted,
    NoPasses,
    Measured,
}
