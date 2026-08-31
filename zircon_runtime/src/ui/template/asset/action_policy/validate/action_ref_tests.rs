use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::template::UiBindingRef;

const PERF_MARKER: &str = "RUNTIME361_UI_ACTION_REF_CACHE_BENCH_V1";

fn binding() -> UiBindingRef {
    toml::from_str(
        r#"
id = "save"
event = "Click"
route = "editor"

[action]
route = "runtime"
action = "save"
"#,
    )
    .expect("action binding should parse")
}

#[test]
fn optimization_batch_20260830bi_runtime_action_ref_cache_preserves_route_and_action() {
    let binding = black_box(binding());
    let action_ref = binding.action.as_ref();
    assert_eq!(
        action_ref.and_then(|action| action.route.as_deref()),
        Some("runtime")
    );
    assert_eq!(
        action_ref.and_then(|action| action.action.as_deref()),
        Some("save")
    );
}

#[test]
fn optimization_batch_20260830bi_runtime_action_ref_cache_source_contract() {
    let source = include_str!("../validate.rs");
    assert!(source.contains("let action_ref = binding.action.as_ref()"));
    assert!(source.contains("action_ref.and_then"));
    assert_eq!(source.matches("binding.action.as_ref()").count(), 1);
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bi_runtime_action_ref_cache_p95() {
    const BINDINGS: usize = 2_000_000;
    const SAMPLES: usize = 17;
    let binding = binding();
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..BINDINGS {
                let (route, action) = if pass == 0 {
                    let route = binding
                        .action
                        .as_ref()
                        .and_then(|action| action.route.as_deref())
                        .or(binding.route.as_deref());
                    let action = binding
                        .action
                        .as_ref()
                        .and_then(|action| action.action.as_deref());
                    (route, action)
                } else {
                    let action_ref = binding.action.as_ref();
                    (
                        action_ref
                            .and_then(|action| action.route.as_deref())
                            .or(binding.route.as_deref()),
                        action_ref.and_then(|action| action.action.as_deref()),
                    )
                };
                checksum += usize::from(route == Some("runtime") && action == Some("save"));
            }
            black_box(checksum);
            let elapsed = started.elapsed().as_nanos();
            if pass == 0 {
                baseline.push(elapsed);
            } else {
                candidate.push(elapsed);
            }
        }
    }
    baseline.sort_unstable();
    candidate.sort_unstable();
    let baseline_p95 = baseline[(SAMPLES * 95).div_ceil(100) - 1];
    let candidate_p95 = candidate[(SAMPLES * 95).div_ceil(100) - 1];
    let reduction =
        100.0 * baseline_p95.saturating_sub(candidate_p95) as f64 / baseline_p95.max(1) as f64;
    println!(
        "{PERF_MARKER} bindings={BINDINGS} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}
