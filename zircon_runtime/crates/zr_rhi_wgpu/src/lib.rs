//! `wgpu` capability mapping and native UI presentation support.
//!
//! Scene/offscreen device ownership lives in `graphics::backend`; the native retained-UI surface
//! owns its surface-compatible device here. Both use the shared timer implementation below with
//! real `wgpu` objects. The deterministic host-mirror device exists only for RHI contract tests.

#[cfg(test)]
mod bind_group_validation;
mod capabilities;
#[cfg(test)]
mod command_validation;
#[cfg(test)]
mod device;
mod gpu_pass_timer;
mod gpu_pipeline_statistics;
mod gpu_readback_queue;
#[cfg(test)]
mod pipeline_validation;
#[cfg(test)]
mod render_pass_validation;
#[cfg(test)]
mod resource_validation;
#[cfg(test)]
mod texture_copy;
mod ui_surface;

pub use capabilities::wgpu_backend_caps;
pub use gpu_pass_timer::{
    GpuPassTimer, GpuPassTimestampScope, GpuPassTiming, GpuTimerFrameResult,
    DEFAULT_GPU_TIMER_MAX_PASSES, GPU_TIMESTAMP_REQUIRED_FEATURES,
};
pub use gpu_pipeline_statistics::{
    GpuPassPipelineStatistics, GpuPipelineStatistics, GpuPipelineStatisticsFrameResult,
    GpuPipelineStatisticsScope, GpuPipelineStatisticsTimer,
    DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES, GPU_PIPELINE_STATISTICS_REQUIRED_FEATURES,
};
pub use gpu_readback_queue::{
    GpuReadbackQueue, ReadbackCallback, ReadbackError, ReadbackPollStats, ReadbackTicket,
};
pub use ui_surface::WgpuUiSurfacePresenter;

#[cfg(test)]
use device::{DeterministicRhiContractCommandList, DeterministicRhiContractDevice};

#[cfg(test)]
mod tests;
