//! Shared asynchronous WGPU buffer readback lifecycle.

mod queue;
mod staging_ring;
mod ticket;

#[cfg(test)]
mod tests;

pub use queue::{GpuReadbackQueue, ReadbackPollStats};
pub use ticket::{ReadbackCallback, ReadbackError, ReadbackTicket};
