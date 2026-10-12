//! 独立 UI 与后端测试使用的异步读回适配器；拷贝录制和映射交付分离，提交/完成权威属于调用方。
//!
//! Shared asynchronous WGPU buffer readback lifecycle.

#[path = "queue.rs"]
mod queue;
#[path = "staging_ring.rs"]
mod staging_ring;
#[path = "texture_readback.rs"]
mod texture_readback;
#[path = "ticket.rs"]
mod ticket;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
#[cfg(test)]
#[path = "tests/texture_tests.rs"]
mod texture_tests;

pub use queue::{GpuReadbackQueue, ReadbackPollStats};
pub use ticket::{ReadbackCallback, ReadbackError, ReadbackTicket};
