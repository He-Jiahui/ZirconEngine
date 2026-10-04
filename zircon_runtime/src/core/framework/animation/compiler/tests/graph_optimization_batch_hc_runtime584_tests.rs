use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn optimization_batch_hc_runtime584_valid_references_preserve_indexes_without_diagnostics() {
    let indexes = BTreeMap::from([("target".to_owned(), 7)]);
    let mut diagnostics = Vec::new();
    let resolved = resolve_references(
        ["target", "target"],
        &indexes,
        &mut diagnostics,
        AnimationCompileElement::GraphNode("blend".to_owned()),
    );

    assert_eq!(resolved, [7, 7]);
    assert!(diagnostics.is_empty());
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hc_runtime584_animation_reference_borrow_p95() {
    const SAMPLE_PAIRS: usize = 21;
    const REFERENCES: usize = 65_536;
    let indexes = BTreeMap::from([("target".to_owned(), 7)]);
    let element = AnimationCompileElement::GraphNode("animation-node/".repeat(128));
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(false, &indexes, &element, REFERENCES));
            optimized.push(measure(true, &indexes, &element, REFERENCES));
        } else {
            optimized.push(measure(true, &indexes, &element, REFERENCES));
            legacy.push(measure(false, &indexes, &element, REFERENCES));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME584_ANIMATION_REFERENCE_BORROW_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
references={REFERENCES} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50),
        "borrowed valid-reference diagnostics must improve P95 by at least 50%"
    );
}

fn measure(
    optimized: bool,
    indexes: &BTreeMap<String, usize>,
    element: &AnimationCompileElement,
    references: usize,
) -> u128 {
    let started = Instant::now();
    let mut diagnostics = Vec::new();
    let mut resolved = 0_usize;
    for _ in 0..references {
        let index = if optimized {
            resolve_reference("target", indexes, &mut diagnostics, black_box(element))
        } else {
            resolve_reference_legacy("target", indexes, &mut diagnostics, black_box(element))
        };
        resolved ^= index.expect("fixture reference should resolve");
    }
    black_box((resolved, diagnostics));
    started.elapsed().as_nanos().max(1)
}

fn resolve_reference_legacy(
    reference: &str,
    indexes: &BTreeMap<String, usize>,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
    element: &AnimationCompileElement,
) -> Option<usize> {
    let owned_element = element.clone();
    match indexes.get(reference) {
        Some(index) => Some(*index),
        None => {
            push_error(
                diagnostics,
                UNKNOWN_NODE_REFERENCE,
                owned_element,
                format!("graph node reference `{reference}` does not exist"),
            );
            None
        }
    }
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
