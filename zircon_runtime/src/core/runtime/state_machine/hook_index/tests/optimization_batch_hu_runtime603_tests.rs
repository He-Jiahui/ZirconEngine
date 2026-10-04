use std::hint::black_box;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum BenchmarkState {
    Loading,
    Running,
}

fn benchmark_index() -> StateHookIndex<BenchmarkState> {
    let mut index = StateHookIndex::default();
    index.register_on_exit(OnExit::new(BenchmarkState::Loading), |_| {});
    index.register_on_transition(
        OnTransition::new(BenchmarkState::Loading, BenchmarkState::Running),
        |_| {},
    );
    index.register_on_enter(OnEnter::new(BenchmarkState::Running), |_| {});
    index
}

fn transition_event() -> StateTransitionEvent<BenchmarkState> {
    StateTransitionEvent::new(
        Some(BenchmarkState::Loading),
        Some(BenchmarkState::Running),
        false,
    )
}

fn legacy_hook_snapshot(
    index: &StateHookIndex<BenchmarkState>,
    event: StateTransitionEvent<BenchmarkState>,
) -> (
    StateTransitionEvent<BenchmarkState>,
    Vec<StateHook<BenchmarkState>>,
    Vec<StateHook<BenchmarkState>>,
    Vec<StateHook<BenchmarkState>>,
) {
    let exit_hooks = index
        .on_exit
        .get(event.exited.as_ref().expect("exited state"))
        .cloned()
        .unwrap_or_default();
    let transition_hooks = index
        .on_transition
        .get(event.exited.as_ref().expect("exited state"))
        .and_then(|targets| targets.get(event.entered.as_ref().expect("entered state")))
        .cloned()
        .unwrap_or_default();
    let enter_hooks = index
        .on_enter
        .get(event.entered.as_ref().expect("entered state"))
        .cloned()
        .unwrap_or_default();
    (event, exit_hooks, transition_hooks, enter_hooks)
}

#[test]
fn optimization_batch_hu_runtime603_single_snapshot_preserves_hook_order() {
    let observed = Arc::new(Mutex::new(Vec::new()));
    let mut index = StateHookIndex::default();
    for label in ["exit-1", "exit-2"] {
        let observed = Arc::clone(&observed);
        index.register_on_exit(OnExit::new(BenchmarkState::Loading), move |_| {
            observed.lock().unwrap().push(label);
        });
    }
    for label in ["transition-1", "transition-2"] {
        let observed = Arc::clone(&observed);
        index.register_on_transition(
            OnTransition::new(BenchmarkState::Loading, BenchmarkState::Running),
            move |_| observed.lock().unwrap().push(label),
        );
    }
    for label in ["enter-1", "enter-2"] {
        let observed = Arc::clone(&observed);
        index.register_on_enter(OnEnter::new(BenchmarkState::Running), move |_| {
            observed.lock().unwrap().push(label);
        });
    }

    index.dispatch(transition_event()).run();

    assert_eq!(
        *observed.lock().unwrap(),
        [
            "exit-1",
            "exit-2",
            "transition-1",
            "transition-2",
            "enter-1",
            "enter-2",
        ]
    );
}

#[test]
fn optimization_batch_hu_runtime603_dispatch_uses_one_ordered_hook_buffer() {
    let index_source = include_str!("../../hook_index.rs");
    let dispatch = index_source
        .split("pub(crate) fn dispatch")
        .nth(1)
        .expect("dispatch implementation")
        .split("#[cfg(test)]")
        .next()
        .expect("bounded dispatch implementation");
    let dispatch_source = include_str!("../../hook.rs");

    assert!(dispatch.contains("Vec::with_capacity"));
    assert!(dispatch.contains("ordered_hooks.extend"));
    assert!(dispatch_source.contains("ordered_hooks: Vec<StateHook<T>>"));
    assert!(!dispatch_source.contains("exit_hooks: Vec<StateHook<T>>"));
    assert!(!dispatch_source.contains("transition_hooks: Vec<StateHook<T>>"));
    assert!(!dispatch_source.contains("enter_hooks: Vec<StateHook<T>>"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_hu_runtime603_state_hook_single_buffer_performance_evidence() {
    const ITERATIONS: usize = 16_384;
    const SAMPLE_PAIRS: usize = 17;
    let index = benchmark_index();
    let event = transition_event();
    let measure_legacy = || {
        let started = Instant::now();
        for _ in 0..ITERATIONS {
            black_box(legacy_hook_snapshot(&index, event.clone()));
        }
        started.elapsed().as_nanos().max(1)
    };
    let measure_single = || {
        let started = Instant::now();
        for _ in 0..ITERATIONS {
            black_box(index.dispatch(event.clone()));
        }
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_legacy());
        black_box(measure_single());
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut single_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            single_samples.push(measure_single());
        } else {
            single_samples.push(measure_single());
            legacy_samples.push(measure_legacy());
        }
    }
    legacy_samples.sort_unstable();
    single_samples.sort_unstable();
    let legacy_p50 = legacy_samples[8];
    let legacy_p95 = legacy_samples[16];
    let single_p50 = single_samples[8];
    let single_p95 = single_samples[16];
    println!(
        "RUNTIME603_STATE_HOOK_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 single_first_pairs=8 iterations={ITERATIONS} hook_count=3 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} single_p50_ns={single_p50} single_p95_ns={single_p95} allocations_per_transition=3->1 target_ratio_bp=7500"
    );
    assert!(
        single_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_500),
        "single hook buffer P95 {single_p95} ns exceeded 75% of legacy {legacy_p95} ns"
    );
}
