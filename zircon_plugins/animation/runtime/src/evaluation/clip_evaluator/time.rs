//! 将播放时间限制到剪辑持续时间；循环边界的最终键取样与事件跨度由各自消费者分别处理。
use zircon_runtime::core::math::Real;

pub(super) fn resolve_sample_time(
    duration_seconds: Real,
    time_seconds: Real,
    looping: bool,
) -> Real {
    if !duration_seconds.is_finite() || duration_seconds <= Real::EPSILON {
        return 0.0;
    }
    if !time_seconds.is_finite() {
        return 0.0;
    }
    let clamped = time_seconds.max(0.0);
    if looping && clamped > duration_seconds {
        clamped.rem_euclid(duration_seconds)
    } else {
        clamped.min(duration_seconds)
    }
}
