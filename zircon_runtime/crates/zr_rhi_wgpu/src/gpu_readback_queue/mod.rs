//! Shared asynchronous WGPU buffer readback lifecycle.

mod queue;
mod staging_ring;
mod texture_readback;
mod ticket;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod texture_tests;

pub use queue::{GpuReadbackQueue, ReadbackPollStats};
pub use ticket::{ReadbackCallback, ReadbackError, ReadbackTicket};
