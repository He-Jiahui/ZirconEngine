use std::hint::black_box;
use std::time::Instant;

use crate::ui::binding::EditorUiBinding;

use super::{
    material_lab_binding_entry, material_lab_structural_child_binding_entry,
    material_lab_template_bindings, MATERIAL_LAB_BINDING_SPECS,
    MATERIAL_LAB_STRUCTURAL_CHILD_BINDING_SPECS,
};

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 100_000;

#[test]
fn optimization_batch_20260829ag_editor252_material_lab_bindings_preserve_order() {
    let optimized = material_lab_template_bindings();
    let legacy = legacy_material_lab_template_bindings();

    assert_eq!(optimized, legacy);
    assert_eq!(
        optimized.len(),
        MATERIAL_LAB_BINDING_SPECS.len() + MATERIAL_LAB_STRUCTURAL_CHILD_BINDING_SPECS.len()
    );
}

#[test]
fn optimization_batch_20260829ag_editor252_material_lab_bindings_reserve_once() {
    let source = include_str!("../material_lab_template_bindings.rs");
    let builder = source
        .split("pub(super) fn material_lab_template_bindings")
        .nth(1)
        .expect("material lab binding builder")
        .split("#[derive(Clone, Copy)]")
        .next()
        .expect("material lab binding builder body");

    assert!(builder.contains("Vec::with_capacity("));
    assert!(builder.contains("MATERIAL_LAB_BINDING_SPECS.len()"));
    assert!(builder.contains("MATERIAL_LAB_STRUCTURAL_CHILD_BINDING_SPECS.len()"));
    assert_eq!(builder.matches("bindings.extend(").count(), 2);
    assert!(!builder.contains("collect::<Vec<_>>"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ag_editor252_single_allocation_material_lab_bindings_bench() {
    let primary = (0..MATERIAL_LAB_BINDING_SPECS.len()).collect::<Vec<_>>();
    let structural = (primary.len()
        ..primary.len() + MATERIAL_LAB_STRUCTURAL_CHILD_BINDING_SPECS.len())
        .collect::<Vec<_>>();
    assert_eq!(
        optimized_merge(&primary, &structural),
        legacy_merge(&primary, &structural)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, &primary, &structural));
            optimized_samples.push(measure(true, &primary, &structural));
        } else {
            optimized_samples.push(measure(true, &primary, &structural));
            legacy_samples.push(measure(false, &primary, &structural));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR252_SINGLE_ALLOCATION_MATERIAL_LAB_BINDINGS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} primary_bindings={} structural_bindings={} \
legacy_buffer_growth_operations_per_build=2 optimized_buffer_growth_operations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        primary.len(),
        structural.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_material_lab_template_bindings() -> Vec<(String, EditorUiBinding)> {
    let mut bindings = MATERIAL_LAB_BINDING_SPECS
        .iter()
        .map(|spec| material_lab_binding_entry(spec.binding_id, spec.event_kind))
        .collect::<Vec<_>>();
    bindings.extend(
        MATERIAL_LAB_STRUCTURAL_CHILD_BINDING_SPECS
            .iter()
            .map(|spec| {
                material_lab_structural_child_binding_entry(
                    spec.binding_id,
                    spec.control_id,
                    spec.event_kind,
                )
            }),
    );
    bindings
}

fn legacy_merge(primary: &[usize], structural: &[usize]) -> Vec<usize> {
    let mut bindings = primary.to_vec();
    bindings.extend_from_slice(structural);
    bindings
}

fn optimized_merge(primary: &[usize], structural: &[usize]) -> Vec<usize> {
    let mut bindings = Vec::with_capacity(primary.len() + structural.len());
    bindings.extend_from_slice(primary);
    bindings.extend_from_slice(structural);
    bindings
}

fn measure(optimized: bool, primary: &[usize], structural: &[usize]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let bindings = if optimized {
            optimized_merge(black_box(primary), black_box(structural))
        } else {
            legacy_merge(black_box(primary), black_box(structural))
        };
        checksum = checksum.wrapping_add(black_box(bindings).len());
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
