use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::{
    ClockDiscontinuity, ClockDomainId, ClockLifecycleTransition, CoreRuntime,
    FrameClockRebaseCause, FrameTimeDiscontinuity, ManualClockSource, TimePolicyTransaction,
    TIME_FPS_DIAGNOSTIC, TIME_FRAME_COUNT_DIAGNOSTIC, TIME_FRAME_TIME_DIAGNOSTIC,
};
use crate::runtime_diagnostics::collect_runtime_diagnostics;

#[test]
fn core_runtime_advances_outer_real_time_and_emits_world_budget() {
    let runtime = CoreRuntime::new();

    let advance = runtime.advance_time_by(Duration::from_millis(34), 8);

    assert_eq!(advance.raw_real_delta(), Duration::from_millis(34));
    assert_eq!(advance.fixed_step_budget(), 8);
    assert_eq!(advance.outer_frame_index(), 1);
    assert_eq!(advance.discontinuity(), None);
    assert_eq!(
        advance
            .clock_domain_stamp(ClockDomainId::MonotonicReal)
            .expect("real clock stamp"),
        runtime.real_time().clock_domain_stamp()
    );
    assert_eq!(
        advance.clock_domain_stamp(ClockDomainId::WorldVirtual),
        None
    );
    assert_eq!(advance.clock_domain_stamp(ClockDomainId::WorldFixed), None);
    assert_eq!(advance.clock_domain_stamp(ClockDomainId::Render), None);
    assert_eq!(runtime.real_time().delta(), Duration::from_millis(34));
    assert_eq!(runtime.real_time().elapsed(), Duration::from_millis(34));
    assert_eq!(runtime.real_time().frame_index(), 1);
}

#[test]
fn core_runtime_uses_an_injected_monotonic_clock_source_for_frame_ticks() {
    let source = Arc::new(ManualClockSource::with_origin(Instant::now()));
    let runtime = CoreRuntime::with_clock_source(source.clone());

    source
        .try_advance_by(Duration::from_millis(20))
        .expect("manual clock source should advance");
    let snapshot = runtime.tick_time(8);

    assert_eq!(snapshot.raw_real_delta(), Duration::from_millis(20));
    assert_eq!(snapshot.fixed_step_budget(), 8);
    assert_eq!(runtime.real_time().elapsed(), Duration::from_millis(20));
}

#[test]
fn default_world_time_policy_transactions_are_validated_without_creating_derived_clocks() {
    let runtime = CoreRuntime::new();
    let original = runtime.time_policy();
    let requested = original
        .with_virtual_relative_speed(0.5)
        .with_fixed_timestep(Duration::from_millis(10));

    let receipt = runtime
        .apply_time_policy(TimePolicyTransaction::new(requested))
        .expect("valid default world policy should commit");

    assert_eq!(receipt.previous(), original);
    assert_eq!(receipt.applied(), requested);
    assert_eq!(runtime.time_policy(), requested);
    assert_eq!(runtime.time_policy_generation(), 1);
}

#[test]
fn first_tick_after_frame_clock_rebase_carries_one_typed_discontinuity() {
    let runtime = CoreRuntime::new();
    let receipt = runtime.rebase_frame_clock();

    let rebased = runtime.tick_time(8);
    let next = runtime.tick_time(8);

    assert_eq!(
        rebased.discontinuity(),
        Some(FrameTimeDiscontinuity::FrameClockRebased(receipt))
    );
    assert_eq!(next.discontinuity(), None);
}

#[test]
fn lifecycle_clock_discontinuity_rebases_the_outer_real_domain_only() {
    let runtime = CoreRuntime::new();
    let discontinuity =
        ClockDiscontinuity::ApplicationLifecycle(ClockLifecycleTransition::Suspended);

    let receipt = runtime.submit_clock_discontinuity(discontinuity);
    let snapshot = runtime.tick_time(8);
    let next = runtime.tick_time(8);

    assert_eq!(
        receipt.cause(),
        FrameClockRebaseCause::ClockDiscontinuity(discontinuity)
    );
    assert_eq!(
        snapshot.discontinuity(),
        Some(FrameTimeDiscontinuity::FrameClockRebased(receipt))
    );
    assert_eq!(next.discontinuity(), None);
    assert_eq!(
        snapshot.real_clock_domain_stamp().source_generation(),
        receipt.generation()
    );
    assert_eq!(
        next.real_clock_domain_stamp().source_generation(),
        receipt.generation()
    );
    assert_eq!(snapshot.clock_domain_stamp(ClockDomainId::WorldFixed), None);
}

#[test]
fn core_runtime_records_real_frame_diagnostics_without_global_fixed_steps() {
    let runtime = CoreRuntime::new();

    runtime.advance_time_by(Duration::from_millis(20), 8);

    let diagnostics = collect_runtime_diagnostics(&runtime.handle()).store;
    let frame_time = series_value(&diagnostics, TIME_FRAME_TIME_DIAGNOSTIC).unwrap();
    let fps = series_value(&diagnostics, TIME_FPS_DIAGNOSTIC).unwrap();
    let frame_count = series_value(&diagnostics, TIME_FRAME_COUNT_DIAGNOSTIC).unwrap();

    assert_eq!(frame_time, 20.0);
    assert!((fps - 50.0).abs() < 0.000_001);
    assert_eq!(frame_count, 1.0);
    assert!(diagnostics
        .series
        .iter()
        .all(|series| series.path.as_str() != "time.fixed_steps"));
}

#[test]
fn fixed_step_plan_separates_interpolation_fraction_from_total_debt() {
    use crate::core::framework::time::FixedStepPlan;

    let plan = FixedStepPlan::new(
        2,
        Duration::from_millis(10),
        Duration::from_millis(20),
        Duration::from_millis(25),
    );

    assert_eq!(plan.debt_duration(), Duration::from_millis(25));
    assert_eq!(plan.debt_whole_steps(), 2);
    assert!((plan.interpolation_fraction() - 0.5).abs() < f32::EPSILON);
}

fn series_value(
    snapshot: &crate::core::diagnostics::DiagnosticStoreSnapshot,
    path: &str,
) -> Option<f64> {
    snapshot
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .and_then(|series| series.current)
}
