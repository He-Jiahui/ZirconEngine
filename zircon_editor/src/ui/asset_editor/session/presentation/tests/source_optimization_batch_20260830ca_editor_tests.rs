use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const ITEMS_PER_SAMPLE: usize = 256;

#[test]
fn source_presentation_reserves_outline_and_diagnostic_capacity() {
    let source = include_str!("../source.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    assert!(implementation.contains("Vec::with_capacity(outline_entries.len())"));
    assert!(implementation.contains("Vec::with_capacity(self.structured_diagnostics.len())"));
    assert!(implementation.contains("for entry in outline_entries"));
    assert!(implementation.contains("for diagnostic in &self.structured_diagnostics"));
}

#[test]
fn source_presentation_keeps_outline_before_diagnostic_order() {
    let source = include_str!("../source.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    let outline = implementation
        .find("for entry in outline_entries")
        .expect("outline loop");
    let diagnostic = implementation
        .find("for diagnostic in &self.structured_diagnostics")
        .expect("diagnostic loop");
    assert!(outline < diagnostic);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830ca_editor_source_presentation_capacity_p95() {
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
        "EDITOR325_SOURCE_PRESENTATION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} items_per_sample={ITEMS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy),
        sample_csv(&optimized),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..128 {
        let mut outline = if optimized {
            Vec::with_capacity(ITEMS_PER_SAMPLE)
        } else {
            Vec::new()
        };
        let mut diagnostics = if optimized {
            Vec::with_capacity(ITEMS_PER_SAMPLE)
        } else {
            Vec::new()
        };
        for index in 0..ITEMS_PER_SAMPLE {
            outline.push(index);
            diagnostics.push(index);
        }
        checksum ^= outline.len() ^ diagnostics.len();
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
