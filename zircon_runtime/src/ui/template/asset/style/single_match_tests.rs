use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::template::{UiSelector, UiSelectorToken};

use super::{UiRuntimeSelectorMatchExt, UiSelectorMatchNode};

const PERF_MARKER: &str = "RUNTIME359_UI_SELECTOR_SINGLE_MATCH_BENCH_V1";

#[test]
fn optimization_batch_20260830bg_runtime_selector_fast_paths_preserve_results() {
    let classes = ["selected".to_string()];
    let selector = black_box(UiSelector::parse(".selected").expect("class selector should parse"));
    let node = black_box(UiSelectorMatchNode {
        component: "Button",
        control_id: None,
        classes: &classes,
        is_host: false,
        states: &[],
    });
    assert!(selector.matches_path(&[node]));

    let selector = UiSelector::parse("Button.selected").expect("compound selector should parse");
    assert!(selector.matches_path(&[node]));
    assert!(matches!(
        selector.segments[0].tokens[0],
        UiSelectorToken::Type(_)
    ));
}

#[test]
fn optimization_batch_20260830bg_runtime_selector_fast_paths_source_contract() {
    let source = include_str!("../style.rs");
    assert!(source.contains("if self.segments.len() == 1"));
    assert!(source.contains("if segment.tokens.len() == 1"));
    assert!(source.contains("fn matches_token("));
    let normalized = source.replace("\r\n", "\n");
    let fast_path = normalized
        .find("if self.segments.len() == 1")
        .expect("single-segment fast path");
    let fallback = normalized
        .find("let mut path_index = path.len() - 1;\n        let mut selector_index")
        .expect("multi-segment fallback traversal");
    assert!(fast_path < fallback);
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bg_runtime_selector_single_match_p95() {
    const MATCHES: usize = 2_000_000;
    const SAMPLES: usize = 17;
    let classes = ["selected".to_string()];
    let selector = UiSelector::parse(".selected").expect("class selector should parse");
    let node = UiSelectorMatchNode {
        component: "Button",
        control_id: None,
        classes: &classes,
        is_host: false,
        states: &[],
    };
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..MATCHES {
                let selector = black_box(&selector);
                let node = black_box(&node);
                let matched = if pass == 0 {
                    legacy_single_segment_match(selector, node)
                } else {
                    selector.matches_path(std::slice::from_ref(node))
                };
                checksum += usize::from(matched);
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
        "{PERF_MARKER} matches={MATCHES} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}

#[inline(never)]
fn legacy_single_segment_match(selector: &UiSelector, node: &UiSelectorMatchNode<'_>) -> bool {
    let path = std::slice::from_ref(node);
    if path.is_empty() || selector.segments.is_empty() {
        return false;
    }
    let path_index = path.len() - 1;
    let selector_index = selector.segments.len() - 1;
    selector.segments[selector_index]
        .tokens
        .iter()
        .all(|token| match token {
            UiSelectorToken::Class(class_name) => path[path_index]
                .classes
                .iter()
                .any(|class| class.as_str() == class_name.as_str()),
            _ => false,
        })
}
