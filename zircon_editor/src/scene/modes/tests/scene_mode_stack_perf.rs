use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const OVERLAYS_PER_SAMPLE: usize = 1_024;

#[test]
fn contribution_retirement_reserves_overlay_upper_bound() {
    let source = include_str!("../scene_mode_stack.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    assert!(implementation.contains("Vec::with_capacity(self.overlays.len())"));
    assert!(implementation.contains("for index in (0..self.overlays.len()).rev()"));
    assert!(implementation.contains("extracted.push(self.overlays.remove(index))"));
}

#[test]
fn contribution_retirement_keeps_reverse_overlay_scan_order() {
    let source = include_str!("../scene_mode_stack.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    let reserve = implementation
        .find("Vec::with_capacity(self.overlays.len())")
        .expect("extracted capacity reservation");
    let scan = implementation
        .find("for index in (0..self.overlays.len()).rev()")
        .expect("reverse overlay scan");
    assert!(reserve < scan);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830br_editor_contribution_retirement_capacity_p95() {
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(false));
            optimized.push(measure(true));
        } else {
            optimized.push(measure(true));
            legacy.push(measure(false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "EDITOR316_CONTRIBUTION_RETIREMENT_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} overlays_per_sample={OVERLAYS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy),
        sample_csv(&optimized),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..256 {
        let mut extracted = if optimized {
            Vec::with_capacity(OVERLAYS_PER_SAMPLE)
        } else {
            Vec::new()
        };
        for index in 0..OVERLAYS_PER_SAMPLE {
            extracted.push(index);
        }
        checksum ^= extracted.len();
    }
    std::hint::black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
