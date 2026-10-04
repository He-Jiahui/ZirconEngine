//! 独立 UI 与后端测试使用的异步读回适配器；拷贝录制和映射交付分离，提交/完成权威属于调用方。
//!
//! Shared asynchronous WGPU buffer readback lifecycle.

mod queue;
mod staging_ring;
mod texture_readback;
mod ticket;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
#[cfg(test)]
#[path = "tests/texture_tests.rs"]
mod texture_tests;

pub use queue::{GpuReadbackQueue, ReadbackPollStats};
pub use ticket::{ReadbackCallback, ReadbackError, ReadbackTicket};
