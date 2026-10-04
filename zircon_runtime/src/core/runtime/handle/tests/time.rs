use std::panic::{self, AssertUnwindSafe};
use std::time::Duration;

use crate::core::{
    CoreRuntime, TimePolicy, TimePolicyError, TimePolicyTransaction, TIME_FRAME_COUNT_DIAGNOSTIC,
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
