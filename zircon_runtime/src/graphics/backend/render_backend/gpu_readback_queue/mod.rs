//! Graphics-backend facade for the shared WGPU readback owner.

pub(crate) use crate::rhi_wgpu::{
    GpuReadbackQueue, ReadbackCallback, ReadbackError, ReadbackPollStats, ReadbackTicket,
};
