//! 偏好设置持久化的三层边界：适配器编排、overlay 可见状态和 worker 后端调用。
//! 后端共享对象通过 Arc 交给 worker；capability 约束原始 I/O 的调用入口。

mod adapter;
mod overlay;
mod work;

pub(crate) use adapter::{
    PreferencePersistenceAdapter, PreferencePersistenceDiagnostics, PreferencePersistenceLimits,
    PreferencePersistenceLimitsError, PreferencePersistenceQuote,
    MAX_PREFERENCE_FAILURE_DETAIL_BYTES, MAX_PREFERENCE_VALUE_BYTES,
};
pub(crate) use overlay::PreferenceOverlayDiagnostics;
pub use work::PreferenceBackendWorkAuthority;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
