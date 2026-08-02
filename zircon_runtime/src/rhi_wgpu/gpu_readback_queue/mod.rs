//! Shared asynchronous WGPU buffer readback lifecycle.

mod queue;
mod staging_ring;
mod ticket;

#[cfg(test)]
mod tests;

pub(crate) use queue::{GpuReadbackQueue, ReadbackPollStats};
pub(crate) use ticket::{ReadbackCallback, ReadbackError, ReadbackTicket};
