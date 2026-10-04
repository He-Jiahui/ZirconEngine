use crate::core::framework::render::RenderGpuTimingStatus;
use crate::graphics::backend::{GpuTimerFrameObservation, GpuTimerFrameStatus};

/// 将 RHI 计时观察映射到 Runtime 可见状态，保留禁用、不可用和延迟到达的区别。
pub(in crate::graphics::scene::scene_renderer::core) fn render_gpu_timing_status(
    timing_requested: bool,
    timer_available: bool,
    observation: Option<GpuTimerFrameObservation>,
) -> RenderGpuTimingStatus {
    if !timing_requested {
        return RenderGpuTimingStatus::Disabled;
    }
    if !timer_available {
        return RenderGpuTimingStatus::Unavailable;
    }
    match observation.map(|observation| observation.status) {
        Some(GpuTimerFrameStatus::Pending) => RenderGpuTimingStatus::Pending,
        Some(GpuTimerFrameStatus::Deferred) | None => RenderGpuTimingStatus::Deferred,
        Some(GpuTimerFrameStatus::CapacityExhausted) => RenderGpuTimingStatus::CapacityExhausted,
        Some(GpuTimerFrameStatus::NoPasses) => RenderGpuTimingStatus::NoPasses,
    }
}

#[cfg(test)]
#[path = "tests/gpu_timing_status.rs"]
mod tests;
