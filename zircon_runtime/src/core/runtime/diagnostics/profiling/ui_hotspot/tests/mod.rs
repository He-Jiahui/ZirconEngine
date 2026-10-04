use super::*;
use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

const PARENT_SOURCE: &str = include_str!("../../ui_hotspot.rs");
const AGGREGATION_TESTS_SOURCE: &str = include_str!("aggregation.rs");
const EVIDENCE_TESTS_SOURCE: &str = include_str!("evidence.rs");
const GPU_ALERTS_TESTS_SOURCE: &str = include_str!("gpu_alerts.rs");
const INTERACTION_ALERTS_TESTS_SOURCE: &str = include_str!("interaction_alerts.rs");
const SUPPORT_TESTS_SOURCE: &str = include_str!("support.rs");

mod support;
use support::counter;

mod aggregation;
mod evidence;
mod gpu_alerts;
mod interaction_alerts;

#[test]
fn ui_hotspot_behavior_tests_are_folder_backed() {
    assert!(
        PARENT_SOURCE.contains("#[path = \"ui_hotspot/tests/mod.rs\"]"),
        "ui_hotspot.rs should route behavior tests through its folder-backed owner"
    );
    for forbidden in ["mod tests {", "include!(\"ui_hotspot/tests/"] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "ui_hotspot.rs should not retain inline behavior test `{forbidden}`"
        );
    }
    for (label, source) in [
        ("ui hotspot aggregation tests", AGGREGATION_TESTS_SOURCE),
        ("ui hotspot evidence tests", EVIDENCE_TESTS_SOURCE),
        ("ui hotspot GPU alert tests", GPU_ALERTS_TESTS_SOURCE),
        (
            "ui hotspot interaction alert tests",
            INTERACTION_ALERTS_TESTS_SOURCE,
        ),
        ("ui hotspot test support", SUPPORT_TESTS_SOURCE),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= 800,
            "{label} has {line_count} lines; expected at most 800"
        );
    }
}

#[test]
fn optimization_batch_ia_runtime610_ui_scenario_index_borrows_counter_text() {
    let implementation = PARENT_SOURCE
        .split("fn parse_ui_counter")
        .next()
        .expect("UI hotspot aggregation implementation");

    assert!(implementation.contains("BTreeMap<&str, UiScenarioAccumulator>"));
    assert!(implementation.contains(".entry(scenario)"));
    assert!(!implementation.contains("scenario.to_string()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_ia_runtime610_borrowed_ui_scenario_index_benchmark() {
    const SCENARIO_COUNT: usize = 128;
    const COUNTERS_PER_SCENARIO: usize = 128;
    const SAMPLE_PAIRS: usize = 17;
    let counter_names = (0..SCENARIO_COUNT)
        .flat_map(|scenario| {
            (0..COUNTERS_PER_SCENARIO)
                .map(move |metric| format!("ui.scenario_{scenario:03}.metric_{metric:03}"))
        })
        .collect::<Vec<_>>();

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut borrowed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_owned_scenario_index(&counter_names));
            borrowed_samples.push(measure_borrowed_scenario_index(&counter_names));
        } else {
            borrowed_samples.push(measure_borrowed_scenario_index(&counter_names));
            legacy_samples.push(measure_owned_scenario_index(&counter_names));
        }
    }
    legacy_samples.sort_unstable();
    borrowed_samples.sort_unstable();
    let legacy_p95_ns = legacy_samples[15];
    let borrowed_p95_ns = borrowed_samples[15];
    println!(
        "RUNTIME610_UI_HOTSPOT_BORROWED_SCENARIO_BENCH_V1 counters={} scenarios={} legacy_p95_ns={} borrowed_p95_ns={} target_ratio_bp=4500",
        counter_names.len(),
        SCENARIO_COUNT,
        legacy_p95_ns,
        borrowed_p95_ns,
    );
    assert!(
        borrowed_p95_ns.saturating_mul(10_000) <= legacy_p95_ns.saturating_mul(4_500),
        "borrowed scenario P95 {borrowed_p95_ns} ns exceeded 45% of legacy {legacy_p95_ns} ns"
    );
}

fn measure_owned_scenario_index(counter_names: &[String]) -> u128 {
    let started = Instant::now();
    let mut scenarios = BTreeMap::<String, usize>::new();
    for name in black_box(counter_names) {
        let scenario = name
            .strip_prefix("ui.")
            .and_then(|rest| rest.split_once('.'))
            .expect("benchmark counter name")
            .0;
        *scenarios.entry(scenario.to_string()).or_default() += 1;
    }
    black_box(scenarios);
    started.elapsed().as_nanos().max(1)
}

fn measure_borrowed_scenario_index(counter_names: &[String]) -> u128 {
    let started = Instant::now();
    let mut scenarios = BTreeMap::<&str, usize>::new();
    for name in black_box(counter_names) {
        let scenario = name
            .strip_prefix("ui.")
            .and_then(|rest| rest.split_once('.'))
            .expect("benchmark counter name")
            .0;
        *scenarios.entry(scenario).or_default() += 1;
    }
    black_box(scenarios);
    started.elapsed().as_nanos().max(1)
}
