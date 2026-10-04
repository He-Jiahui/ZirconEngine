//! Deterministic seed authority and random-stream execution.
//! Runtime 持有种子代际与稳定键下的随机流进度；调用方通过租约抽样并在归还后保存进度。
//! 此服务只覆盖随机状态，完整模拟恢复还需由上层统一捕获 World、时钟和调度状态。

mod authority;
mod derivation;
mod error;
mod lease;
mod limits;
mod registry;
mod service;
mod stream;
mod stream_error;

pub use error::RandomServiceError;
pub use lease::RandomStreamLease;
pub use limits::RandomServiceLimits;
pub use service::RandomService;
pub use stream::RandomStream;
pub use stream_error::RandomStreamError;

#[cfg(test)]
mod tests;
