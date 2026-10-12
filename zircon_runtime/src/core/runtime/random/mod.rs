//! Deterministic seed authority and random-stream execution.
//! Runtime 持有种子代际与稳定键下的随机流进度；调用方通过租约抽样并在归还后保存进度。
//! 此服务只覆盖随机状态，完整模拟恢复还需由上层统一捕获 World、时钟和调度状态。

#[path = "authority.rs"]
mod authority;
#[path = "derivation.rs"]
mod derivation;
#[path = "error.rs"]
mod error;
#[path = "lease.rs"]
mod lease;
#[path = "limits.rs"]
mod limits;
#[path = "registry.rs"]
mod registry;
#[path = "service.rs"]
mod service;
#[path = "stream.rs"]
mod stream;
#[path = "stream_error.rs"]
mod stream_error;

pub use error::RandomServiceError;
pub use lease::RandomStreamLease;
pub use limits::RandomServiceLimits;
pub use service::RandomService;
pub use stream::RandomStream;
pub use stream_error::RandomStreamError;

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
