use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

use super::*;

const IDENTITY_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_it_editor630_preserves_duplicate_component_error() {
    let source = r#"
version = 1

[[components]]
component_id = "duplicate"
document_id = "res://ui/editor/first.zui"
binding_namespace = "first"

[[components]]
component_id = "duplicate"
document_id = "res://ui/editor/second.zui"
binding_namespace = "second"
"#;

    assert_eq!(
        parse_editor_component_catalog_manifest(source),
        Err(EditorComponentCatalogManifestError::DuplicateComponent {
            component_id: "duplicate".to_string(),
        })
    );
}

#[test]
fn optimization_batch_it_editor630_hashes_catalog_membership_with_exact_bounds() {
    let source = include_str!("../../catalog.rs");
    let parse_body = source
        .split("pub fn parse_editor_component_catalog_manifest")
        .nth(1)
        .expect("component catalog parser remains present")
        .split("fn is_builtin_editor_component_document_id")
        .next()
        .expect("component catalog parser remains bounded");

    assert!(parse_body.contains("HashSet::with_capacity(catalog.components.len())"));
    assert!(parse_body.contains("HashSet::with_capacity(descriptor.slots.len())"));
    assert!(parse_body.contains("HashSet::with_capacity(descriptor.props.len())"));
    assert!(!parse_body.contains("let mut component_ids = BTreeSet::new();"));
    assert!(!parse_body.contains("let mut slot_names = BTreeSet::new();"));
    assert!(!parse_body.contains("let mut property_names = BTreeSet::new();"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_it_editor630_hash_catalog_membership_benchmark() {
    let identities = (0..IDENTITY_COUNT)
        .map(|index| format!("editor.component.{index:08}"))
        .collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_membership(&identities, false));
        black_box(measure_membership(&identities, true));
    }

    let mut ordered_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut hash_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            ordered_samples.push(measure_membership(&identities, false));
            hash_samples.push(measure_membership(&identities, true));
        } else {
            hash_samples.push(measure_membership(&identities, true));
            ordered_samples.push(measure_membership(&identities, false));
        }
    }

    let ordered_p95 = percentile(&ordered_samples, 95);
    let hash_p95 = percentile(&hash_samples, 95);
    let improvement_percent =
        ordered_p95.saturating_sub(hash_p95).saturating_mul(100) / ordered_p95.max(1);
    println!(
        "EDITOR630_HASH_COMPONENT_CATALOG_MEMBERSHIP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} identity_count={IDENTITY_COUNT} ordered_ns={} hash_ns={} ordered_p95_ns={ordered_p95} hash_p95_ns={hash_p95} improvement_percent={improvement_percent} threshold_percent=60",
        csv(&ordered_samples),
        csv(&hash_samples),
    );
    assert!(hash_p95 <= ordered_p95 * 40 / 100);
}

fn measure_membership(identities: &[String], hash: bool) -> u128 {
    let started = Instant::now();
    if hash {
        let mut seen = HashSet::with_capacity(identities.len());
        for identity in identities {
            black_box(seen.insert(black_box(identity.as_str())));
        }
        black_box(seen);
    } else {
        let mut seen = BTreeSet::new();
        for identity in identities {
            black_box(seen.insert(black_box(identity.as_str())));
        }
        black_box(seen);
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
