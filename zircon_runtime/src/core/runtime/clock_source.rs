//! Injection boundary for samples that advance the runtime frame clock.
//! 为 Runtime 外帧提供可注入的采样来源；不接管操作系统超时、分析计时或 World 时钟。

use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};
use web_time::{Duration, Instant};

use thiserror::Error;

/// Supplies the monotonic samples consumed by [`super::FrameClock`].
///
/// This boundary intentionally covers frame-delta measurement only. OS
/// deadlines, file watchers, telemetry, and profiling retain their own time
/// sources because they must continue to progress while a runtime clock is
/// paused, replayed, or otherwise controlled.
/// 注入者提供可跨线程共享且不借用短期数据的采样器；返回值只用于外帧增量测量。
/// 内建帧时钟对倒退样本保留旧基线并给出零增量；来源仍应提供单调样本。
pub trait ClockSource: Send + Sync + 'static {
    /// Returns the next monotonic sample for a runtime outer-frame tick.
    /// 读取下一次 `Instant` 样本；是否推进由实现决定，手动来源的读取不会自行走时。
    fn monotonic_now(&self) -> Instant;
}

/// A caller-driven monotonic source for tests, replay, and external authorities.
///
/// The source stores an opaque `Instant` anchor together with a non-decreasing
/// elapsed duration. Replay seek must construct a new source and rebase the
/// frame clock instead of producing a negative outer-frame delta.
/// 克隆共享同一个加锁状态；任一副本推进后，其余副本读取同一已接受位置。
/// 只有显式推进修改样本，实时时钟等待不会使它自动前进。
#[derive(Clone, Debug)]
pub struct ManualClockSource {
    state: Arc<Mutex<ManualClockState>>,
}

#[derive(Clone, Copy, Debug)]
struct ManualClockState {
    now: Instant,
    elapsed: Duration,
}

impl ManualClockSource {
    /// Creates a source at a caller-owned monotonic origin.
    /// `origin` 是不透明的单调起点；返回来源的累计时长为零，不解释为日历时间。
    pub fn with_origin(origin: Instant) -> Self {
        Self {
            state: Arc::new(Mutex::new(ManualClockState {
                now: origin,
                elapsed: Duration::ZERO,
            })),
        }
    }

    /// Returns elapsed time accepted by this source since its origin.
    pub fn elapsed(&self) -> Duration {
        self.lock_state().elapsed
    }

    /// Advances the source by a non-negative delta.
    /// `delta` 相对当前已接受位置累加；Duration 或 Instant 不可表示结果时返回错误，状态不变。
    pub fn try_advance_by(&self, delta: Duration) -> Result<(), ManualClockSourceError> {
        let mut state = self.lock_state();
        let requested =
            state
                .elapsed
                .checked_add(delta)
                .ok_or(ManualClockSourceError::OutOfRange {
                    current: state.elapsed,
                    delta,
                })?;
        advance_state_to(&mut state, requested)
    }

    /// Advances the source to an absolute elapsed position from its origin.
    /// `requested` 是相对创建起点的绝对累计时长，允许原地采样；倒退或溢出均拒绝且不写状态。
    pub fn try_advance_to(&self, requested: Duration) -> Result<(), ManualClockSourceError> {
        let mut state = self.lock_state();
        advance_state_to(&mut state, requested)
    }

    fn lock_state(&self) -> MutexGuard<'_, ManualClockState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl ClockSource for ManualClockSource {
    fn monotonic_now(&self) -> Instant {
        self.lock_state().now
    }
}

// 先验证 elapsed 单调且 Instant 可表示新位置，再同时提交两个字段；任一失败都保留原时钟基线。
fn advance_state_to(
    state: &mut ManualClockState,
    requested: Duration,
) -> Result<(), ManualClockSourceError> {
    if requested < state.elapsed {
        return Err(ManualClockSourceError::NonMonotonicAdvance {
            current: state.elapsed,
            requested,
        });
    }
    let delta = requested.saturating_sub(state.elapsed);
    let Some(now) = state.now.checked_add(delta) else {
        return Err(ManualClockSourceError::OutOfRange {
            current: state.elapsed,
            delta,
        });
    };
    state.now = now;
    state.elapsed = requested;
    Ok(())
}

/// Rejection from a caller-driven monotonic clock update.
/// 两种拒绝都发生在状态提交之前；其中时长字段以该手动来源的起点为基准。
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ManualClockSourceError {
    /// `requested` 小于当前已接受的 `current`，不能用此来源回放到更早位置。
    #[error("manual clock cannot move backward from {current:?} to {requested:?}")]
    NonMonotonicAdvance {
        current: Duration,
        requested: Duration,
    },
    /// `current` 为旧累计时长，`delta` 为本次增量；时长加法或 Instant 加法无法表示结果。
    #[error("manual clock cannot represent a {delta:?} advance from {current:?}")]
    OutOfRange { current: Duration, delta: Duration },
}

/// 系统分支直接调用系统采样；注入分支持有来源的 Arc，克隆保留同一来源的生命周期。
#[derive(Clone)]
pub(crate) enum FrameClockSource {
    SystemMonotonic,
    Injected(Arc<dyn ClockSource>),
}

impl FrameClockSource {
    pub(crate) const fn system_monotonic() -> Self {
        Self::SystemMonotonic
    }

    pub(crate) fn injected(source: Arc<dyn ClockSource>) -> Self {
        Self::Injected(source)
    }

    pub(crate) fn monotonic_now(&self) -> Instant {
        match self {
            Self::SystemMonotonic => Instant::now(),
            Self::Injected(source) => source.monotonic_now(),
        }
    }
}

// 调试输出只标识来源分支，不再次采样，也不要求注入对象实现 Debug。
impl fmt::Debug for FrameClockSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SystemMonotonic => formatter.write_str("FrameClockSource::SystemMonotonic"),
            Self::Injected(_) => formatter.write_str("FrameClockSource::Injected(..)"),
        }
    }
}

#[cfg(test)]
#[path = "tests/clock_source.rs"]
mod tests;
