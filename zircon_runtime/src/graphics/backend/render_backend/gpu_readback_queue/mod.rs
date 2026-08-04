//! Graphics-backend facade for the shared WGPU readback owner.

pub(crate) use zr_rhi_wgpu::{
    GpuReadbackQueue, ReadbackCallback, ReadbackError, ReadbackPollStats, ReadbackTicket,
};
