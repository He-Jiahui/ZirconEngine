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

    pub fn advance_time_by(&self, real_delta: Duration, max_fixed_steps: u32) -> FrameTimeSnapshot {
        self.advance_time_by_with_discontinuity(real_delta, max_fixed_steps, None)
    }

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
mod tests {
    use std::panic::{self, AssertUnwindSafe};
    use std::time::Duration;

    use crate::core::{
        CoreRuntime, TimePolicy, TimePolicyError, TimePolicyTransaction,
        TIME_FRAME_COUNT_DIAGNOSTIC,
    };

    #[test]
    fn core_handle_commits_only_valid_default_world_time_policy_transactions() {
        let runtime = CoreRuntime::new();
        let handle = runtime.handle();
        let initial = handle.time_policy();

        let receipt = handle
            .apply_time_policy(TimePolicyTransaction::new(TimePolicy::new(
                Duration::from_millis(100),
                0.5,
                Duration::from_millis(20),
            )))
            .expect("a valid time policy should commit");

        assert!(receipt.changed());
        assert_eq!(receipt.previous(), initial);
        assert_eq!(receipt.generation(), 1);
        assert_eq!(handle.time_policy(), receipt.applied());

        for (invalid_policy, expected_error) in [
            (
                TimePolicy::new(Duration::ZERO, 1.0, Duration::from_millis(16)),
                TimePolicyError::VirtualMaxDeltaZero,
            ),
            (
                TimePolicy::new(
                    Duration::from_millis(16),
                    f64::NAN,
                    Duration::from_millis(16),
                ),
                TimePolicyError::VirtualRelativeSpeedNotFinite,
            ),
            (
                TimePolicy::new(Duration::from_millis(16), -1.0, Duration::from_millis(16)),
                TimePolicyError::VirtualRelativeSpeedNegative,
            ),
            (
                TimePolicy::new(Duration::from_millis(16), 1.0, Duration::ZERO),
                TimePolicyError::FixedTimestepZero,
            ),
        ] {
            let rejection = handle
                .apply_time_policy(TimePolicyTransaction::new(invalid_policy))
                .expect_err("an invalid time policy must reject before mutation");

            assert_eq!(rejection, expected_error);
            assert_eq!(handle.time_policy(), receipt.applied());
            assert_eq!(handle.time_policy_generation(), receipt.generation());
        }
    }

    #[test]
    fn core_handle_time_accessors_recover_poisoned_outer_time_locks() {
        let runtime = CoreRuntime::new();
        let handle = runtime.handle();

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.time.lock().unwrap();
            panic!("poison core handle outer time");
        }));
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.frame_clock.lock().unwrap();
            panic!("poison core handle frame clock");
        }));
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.diagnostics.lock().unwrap();
            panic!("poison core handle time diagnostics");
        }));

        handle
            .apply_time_policy(TimePolicyTransaction::new(TimePolicy::new(
                Duration::from_millis(33),
                1.0,
                Duration::from_millis(16),
            )))
            .expect("valid policy should recover poisoned outer time");

        let advance = handle.advance_time_by(Duration::from_millis(16), 4);
        assert_eq!(advance.raw_real_delta(), Duration::from_millis(16));
        assert_eq!(handle.real_time().frame_index(), 1);

        let tick_advance = handle.tick_time(4);
        assert!(handle.real_time().frame_index() >= 2);
        assert_eq!(
            tick_advance.raw_real_delta(),
            handle.real_time().delta(),
            "tick_time should advance from the recovered frame clock delta"
        );

        let snapshot = handle.diagnostic_store_snapshot();
        assert!(
            snapshot
                .series
                .iter()
                .any(|series| series.path.as_str() == TIME_FRAME_COUNT_DIAGNOSTIC),
            "time diagnostics should be recorded through the recovered diagnostics store"
        );
    }
}
