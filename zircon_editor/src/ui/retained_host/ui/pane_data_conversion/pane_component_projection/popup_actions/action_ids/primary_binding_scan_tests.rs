use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::binding::UiEventKind;

use super::{primary_binding_refs, projected_action_ids, RetainedUiHostBindingProjection};

const SAMPLE_PAIRS: usize = 21;
const SCANS_PER_SAMPLE: usize = 8_192;
const BINDING_COUNT: usize = 256;

#[test]
fn optimization_batch_20260826eb_editor117_primary_binding_scan_preserves_first_binding_behavior() {
    let bindings = vec![
        binding("first-click", "", UiEventKind::Click),
        binding("second-click", "second.action", UiEventKind::Click),
        binding("field/change", "", UiEventKind::Change),
        binding("field/submit", "", UiEventKind::Submit),
    ];
    let primary = primary_binding_refs(&bindings);
    assert_eq!(primary.click.unwrap().binding_id, "first-click");
    assert_eq!(primary.change.unwrap().binding_id, "field/change");
    assert_eq!(primary.submit.unwrap().binding_id, "field/submit");

    let projected = projected_action_ids("Field", &bindings, None, false, false);
    assert_eq!(projected.binding_id, "first-click");
    assert_eq!(projected.action_id, "");
    assert_eq!(projected.edit_action_id, "field.change");
    assert_eq!(projected.commit_action_id, "field.submit");
}

#[test]
fn optimization_batch_20260826eb_editor117_primary_binding_scan_uses_one_loop() {
    let source = include_str!("../action_ids.rs");
    let function_start = source.find("pub(super) fn projected_action_ids").unwrap();
    let function_end = source[function_start..]
        .find("#[derive(Default)]")
        .map(|offset| function_start + offset)
        .unwrap();
    let function_source = &source[function_start..function_end];
    assert_eq!(
        function_source
            .matches("primary_binding_refs(bindings)")
            .count(),
        1
    );
    assert!(!function_source.contains("primary_click_binding_id"));
    assert!(!function_source.contains("primary_click_action_id"));
    assert!(!function_source.contains("primary_submit_action_id"));
    assert!(!function_source.contains("primary_change_action_id"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826eb_editor117_popup_action_single_binding_scan_bench() {
    let bindings = fixture_bindings();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&bindings));
            optimized_samples.push(measure_optimized(&bindings));
        } else {
            optimized_samples.push(measure_optimized(&bindings));
            legacy_samples.push(measure_legacy(&bindings));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR117_POPUP_ACTION_SINGLE_BINDING_SCAN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
scans_per_sample={SCANS_PER_SAMPLE} bindings_per_scan={BINDING_COUNT} legacy_passes=4 \
optimized_passes=1 legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single primary binding scan P95 {optimized_p95_ns}ns must be at most 70% of four-scan P95 {legacy_p95_ns}ns"
    );
}

fn binding(
    binding_id: &str,
    action_id: &str,
    event_kind: UiEventKind,
) -> RetainedUiHostBindingProjection {
    RetainedUiHostBindingProjection {
        binding_id: binding_id.to_string(),
        action_id: action_id.to_string(),
        event_kind,
        route_id: None,
        template_action_source: None,
        template_action: None,
    }
}

fn fixture_bindings() -> Vec<RetainedUiHostBindingProjection> {
    let mut bindings = (0..BINDING_COUNT)
        .map(|index| binding(&format!("filler/{index}"), "", UiEventKind::DoubleClick))
        .collect::<Vec<_>>();
    bindings[64] = binding("primary/click", "primary.click", UiEventKind::Click);
    bindings[128] = binding("primary/change", "", UiEventKind::Change);
    bindings[255] = binding("primary/submit", "", UiEventKind::Submit);
    bindings
}

fn legacy_primary_bindings(
    bindings: &[RetainedUiHostBindingProjection],
) -> (
    Option<&RetainedUiHostBindingProjection>,
    Option<&RetainedUiHostBindingProjection>,
    Option<&RetainedUiHostBindingProjection>,
    Option<&RetainedUiHostBindingProjection>,
) {
    (
        bindings
            .iter()
            .find(|binding| binding.event_kind == UiEventKind::Click),
        bindings
            .iter()
            .find(|binding| binding.event_kind == UiEventKind::Click),
        bindings
            .iter()
            .find(|binding| binding.event_kind == UiEventKind::Change),
        bindings
            .iter()
            .find(|binding| binding.event_kind == UiEventKind::Submit),
    )
}

fn measure_legacy(bindings: &[RetainedUiHostBindingProjection]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..SCANS_PER_SAMPLE {
        let (click_binding, click_action, change, submit) =
            black_box(legacy_primary_bindings(black_box(bindings)));
        checksum ^= click_binding.unwrap().binding_id.len();
        checksum ^= click_action.unwrap().action_id.len();
        checksum ^= change.unwrap().binding_id.len();
        checksum ^= submit.unwrap().binding_id.len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(bindings: &[RetainedUiHostBindingProjection]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..SCANS_PER_SAMPLE {
        let primary = black_box(primary_binding_refs(black_box(bindings)));
        checksum ^= primary.click.unwrap().binding_id.len();
        checksum ^= primary.click.unwrap().action_id.len();
        checksum ^= primary.change.unwrap().binding_id.len();
        checksum ^= primary.submit.unwrap().binding_id.len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
