//! Monotonic frame-delta timing.

use std::sync::Arc;
use std::time::{Duration, Instant};

use super::clock_source::{ClockSource, FrameClockSource};

/// rebase 后的首个 tick 仍测量新基线到该帧的间隔，不合成零增量。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameClockFirstTickPolicy {
    MeasureFromRebase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockLifecycleTransition {
    Foregrounded,
    Backgrounded,
    Suspended,
    Resumed,
}

/// 宿主报告给帧时钟的可见时间断点；下一帧会携带重置来源。
///
/// 窗口和应用生命周期变化只重置采样基线，不改 World 自有的虚拟/固定时间。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockDiscontinuity {
    ApplicationLifecycle(ClockLifecycleTransition),
    WindowOcclusionChanged { occluded: bool },
    WindowSurfaceRecreated,
}

/// 记录重置采样基线的原因，供下游区分会话切换与宿主时钟断点。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameClockRebaseCause {
    Manual,
    SessionActivationCompleted,
    ClockDiscontinuity(ClockDiscontinuity),
}

/// 本次采样基线重置的凭据，供下一次外帧标注时间域代际。
///
/// 多次 rebase 后仅最新凭据随 tick 交付；代际保持不回退。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameClockRebaseReceipt {
    generation: u64,
    first_tick_policy: FrameClockFirstTickPolicy,
    cause: FrameClockRebaseCause,
}

impl FrameClockRebaseReceipt {
    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub const fn first_tick_policy(self) -> FrameClockFirstTickPolicy {
        self.first_tick_policy
    }

    pub const fn cause(self) -> FrameClockRebaseCause {
        self.cause
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FrameClockTick {
    delta: Duration,
    rebase: Option<FrameClockRebaseReceipt>,
}

impl FrameClockTick {
    pub(crate) const fn delta(self) -> Duration {
        self.delta
    }

    pub(crate) const fn rebase(self) -> Option<FrameClockRebaseReceipt> {
        self.rebase
    }
}

/// Runtime 外帧的单调采样器，和 World 推演时钟分属不同所有者。
///
/// CoreHandle::tick_time 消费采样；宿主在会话激活或窗口时间断点后先 rebase。
#[derive(Debug, Clone)]
pub struct FrameClock {
    source: FrameClockSource,
    last_tick: Instant,
    rebase_generation: u64,
    pending_rebase: Option<FrameClockRebaseReceipt>,
}

impl Default for FrameClock {
    fn default() -> Self {
        Self::with_source(FrameClockSource::system_monotonic())
    }
}

impl FrameClock {
    /// Creates a frame clock driven by an explicitly owned monotonic source.
    ///
    /// Production construction uses [`Default`], which retains the direct OS
    /// monotonic fast path. Injected sources exist for deterministic tests and
    /// future replay ownership.
    pub fn with_clock_source(source: Arc<dyn ClockSource>) -> Self {
        Self::with_source(FrameClockSource::injected(source))
    }

    fn with_source(source: FrameClockSource) -> Self {
        let last_tick = source.monotonic_now();
        Self {
            source,
            last_tick,
            rebase_generation: 0,
            pending_rebase: None,
        }
    }

    /// 重置下次外帧的测量起点；凭据会在随后一次 tick 中消费。
    pub fn rebase(&mut self) -> FrameClockRebaseReceipt {
        self.rebase_for(FrameClockRebaseCause::Manual)
    }

    pub(crate) fn rebase_for(&mut self, cause: FrameClockRebaseCause) -> FrameClockRebaseReceipt {
        let _ = self.advance_baseline_to(self.source.monotonic_now());
        self.rebase_generation = self.rebase_generation.saturating_add(1);
        let receipt = FrameClockRebaseReceipt {
            generation: self.rebase_generation,
            first_tick_policy: FrameClockFirstTickPolicy::MeasureFromRebase,
            cause,
        };
        self.pending_rebase = Some(receipt);
        receipt
    }

    pub(crate) fn tick(&mut self) -> FrameClockTick {
        let now = self.source.monotonic_now();
        let delta = self.advance_baseline_to(now).unwrap_or(Duration::ZERO);
        FrameClockTick {
            delta,
            rebase: self.pending_rebase.take(),
        }
    }

    fn advance_baseline_to(&mut self, now: Instant) -> Option<Duration> {
        // 采样早于基线时保留最后有效基线，让后续有效样本继续从此处计时。
        let delta = now.checked_duration_since(self.last_tick)?;
        self.last_tick = now;
        Some(delta)
    }
}

#[cfg(test)]
#[path = "tests/frame_clock.rs"]
mod tests;
