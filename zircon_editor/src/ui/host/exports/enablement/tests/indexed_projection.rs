use std::hint::black_box;
use std::time::Instant;

use super::project_editor_capabilities;

const MARKER: &str = "EDITOR188_CAPABILITY_TARGET_INDEX_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const REPEATS: usize = 1_024;

#[test]
fn optimization_batch_20260826gv_editor188_capability_projection_preserves_mode_semantics() {
    let previous = strings(["zeta", "alpha", "zeta", "keep"]);
    let targets = strings(["zeta", "beta", "beta"]);

    assert_eq!(
        project_editor_capabilities(&previous, &targets, false),
        strings(["alpha", "keep"])
    );
    assert_eq!(
        project_editor_capabilities(&previous, &targets, true),
        strings(["alpha", "beta", "keep", "zeta"])
    );
}

#[test]
fn optimization_batch_20260826gv_editor188_capability_projection_indexes_targets_once() {
    let source = include_str!("../capabilities.rs");
    let implementation = source
        .split("fn project_editor_capabilities")
        .nth(1)
        .and_then(|tail| tail.split("#[cfg(test)]").next())
        .expect("capability projection implementation");
    assert!(implementation.contains("targets.iter().map(String::as_str).collect"));
    assert!(implementation.contains("target_index.contains(existing.as_str())"));
    assert!(!implementation.contains(".any(|capability| capability == existing)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gv_editor188_capability_target_index_bench() {
    let previous = (0..1_024)
        .map(|index| format!("editor.capability.{index:04}"))
        .collect::<Vec<_>>();
    let targets = (0..768)
        .map(|index| format!("editor.capability.{index:04}"))
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(
                &previous,
                &targets,
                legacy_project_editor_capabilities,
            ));
            optimized_samples.push(measure(&previous, &targets, project_editor_capabilities));
        } else {
            optimized_samples.push(measure(&previous, &targets, project_editor_capabilities));
            legacy_samples.push(measure(
                &previous,
                &targets,
                legacy_project_editor_capabilities,
            ));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "indexed target matching must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn strings<const N: usize>(values: [&str; N]) -> Vec<String> {
    values.into_iter().map(str::to_string).collect()
}

fn legacy_project_editor_capabilities(
    previous: &[String],
    targets: &[String],
    enabled: bool,
) -> Vec<String> {
    debug_assert!(!enabled);
    let mut capabilities = previous.to_vec();
    capabilities.retain(|existing| !targets.iter().any(|capability| capability == existing));
    capabilities
}

fn measure(
    previous: &[String],
    targets: &[String],
    implementation: fn(&[String], &[String], bool) -> Vec<String>,
) -> u64 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..REPEATS {
        let projected = implementation(black_box(previous), black_box(targets), false);
        checksum = checksum.wrapping_add(projected.len());
        black_box(&projected);
    }
    black_box(checksum);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
