use std::sync::MutexGuard;
use std::time::Duration;

use crate::core::framework::time::{
    MonotonicReal, Time, TimePolicy, TimePolicyError, TimePolicyTransaction,
};

use super::super::frame_clock::{
    ClockDiscontinuity, FrameClock, FrameClockRebaseCause, FrameClockRebaseReceipt,
};
use super::super::time::{
    FrameTimeDiscontinuity, FrameTimeSnapshot, RuntimeTimeAuthority, TimePolicyReceipt,
    TIME_FPS_DIAGNOSTIC, TIME_FRAME_COUNT_DIAGNOSTIC, TIME_FRAME_TIME_DIAGNOSTIC,
};
use super::CoreHandle;

impl CoreHandle {
    /// Returns the outer monotonic frame clock. World-derived clocks belong to Levels.
    pub fn real_time(&self) -> Time<MonotonicReal> {
        self.lock_time().real()
    }

    /// Returns the default policy used when a new Level is created.
    pub fn time_policy(&self) -> TimePolicy {
        self.lock_time().time_policy()
    }

    /// Returns the generation of the default policy for subsequently created Levels.
    pub fn time_policy_generation(&self) -> u64 {
        self.lock_time().time_policy_generation()
    }

    /// Changes the default policy for subsequently created Levels.
    ///
    /// Existing Levels retain their own timing policy and fixed debt. Live
    /// multi-World policy propagation requires an explicit Level transaction.
    pub fn apply_time_policy(
        &self,
        transaction: TimePolicyTransaction,
    ) -> Result<TimePolicyReceipt, TimePolicyError> {
        self.lock_time().apply_time_policy(transaction)
    }

    /// 用宿主提供的帧增量推进外层时间，供离线驱动或测试生成世界预算。
    /// 同一帧只应选择此入口或 [`Self::tick_time`]，避免重复推进。
    pub fn advance_time_by(&self, real_delta: Duration, max_fixed_steps: u32) -> FrameTimeSnapshot {
        self.advance_time_by_with_discontinuity(real_delta, max_fixed_steps, None)
    }

    /// 从帧时钟采样并推进一次外层帧；重基准信息随快照交给会话与世界。
    pub fn tick_time(&self, max_fixed_steps: u32) -> FrameTimeSnapshot {
        let frame_tick = self.lock_frame_clock().tick();
        self.advance_time_by_with_discontinuity(
            frame_tick.delta(),
            max_fixed_steps,
            frame_tick
                .rebase()
                .map(FrameTimeDiscontinuity::FrameClockRebased),
        )
    }

    fn advance_time_by_with_discontinuity(
        &self,
        raw_real_delta: Duration,
        max_fixed_steps: u32,
        discontinuity: Option<FrameTimeDiscontinuity>,
    ) -> FrameTimeSnapshot {
        let snapshot = {
            let mut time = self.lock_time();
            time.advance_by_with_discontinuity(raw_real_delta, max_fixed_steps, discontinuity)
        };
        record_time_diagnostics(self, snapshot);
        snapshot
    }

    pub(crate) fn rebase_frame_clock(&self) -> FrameClockRebaseReceipt {
        self.lock_frame_clock()
            .rebase_for(FrameClockRebaseCause::SessionActivationCompleted)
    }

    /// 宿主恢复或时钟跳变后重基准下一帧采样，避免把停顿时间计入模拟增量。
    pub fn submit_clock_discontinuity(
        &self,
        discontinuity: ClockDiscontinuity,
    ) -> FrameClockRebaseReceipt {
        self.lock_frame_clock()
            .rebase_for(FrameClockRebaseCause::ClockDiscontinuity(discontinuity))
    }

    fn lock_time(&self) -> MutexGuard<'_, RuntimeTimeAuthority> {
        self.inner
            .time
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_frame_clock(&self) -> MutexGuard<'_, FrameClock> {
        self.inner
            .frame_clock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn record_time_diagnostics(handle: &CoreHandle, snapshot: FrameTimeSnapshot) {
    let frame_index = snapshot.outer_frame_index();
    let real_delta_seconds = snapshot.raw_real_delta().as_secs_f64();
    let mut diagnostics = handle.lock_diagnostics();

    diagnostics.record_static(
        TIME_FRAME_COUNT_DIAGNOSTIC,
        frame_index,
        frame_index as f64,
        Some("frame"),
        &["time", "frame"],
    );
    if real_delta_seconds == 0.0 {
        return;
    }
    diagnostics.record_static(
        TIME_FRAME_TIME_DIAGNOSTIC,
        frame_index,
        real_delta_seconds * 1_000.0,
        Some("ms"),
        &["time", "frame"],
    );
    diagnostics.record_static(
        TIME_FPS_DIAGNOSTIC,
        frame_index,
        1.0 / real_delta_seconds,
        Some("hz"),
        &["time", "frame"],
    );
}

#[cfg(test)]
#[path = "tests/time.rs"]
mod tests;
