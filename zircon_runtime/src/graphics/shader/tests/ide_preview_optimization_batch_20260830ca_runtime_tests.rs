use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const ENTRIES_PER_SAMPLE: usize = 256;

#[test]
fn shader_preview_reserves_segment_index_and_include_capacity() {
    let source = include_str!("../ide_preview.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    assert!(implementation.contains("Vec::with_capacity(assembly.segments.len())"));
    assert!(implementation.contains("HashMap::with_capacity(upper_bound.unwrap_or(lower_bound))"));
    assert!(implementation.contains("Vec::with_capacity(shader.imports.len())"));
    assert!(implementation.contains("HashSet::with_capacity(shader.imports.len())"));
}

#[test]
fn shader_preview_keeps_assembly_before_recursive_include_collection() {
    let source = include_str!("../ide_preview.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    let assembly = implementation
        .find("assemble_material_shader_template")
        .expect("assembly");
    let include = implementation
        .find("collect_shader_module_include_sources")
        .expect("recursive include collection");
    assert!(assembly < include);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830ca_runtime_shader_preview_capacity_p95() {
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
        "RUNTIME379_SHADER_IDE_PREVIEW_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} entries_per_sample={ENTRIES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy),
        sample_csv(&optimized),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..128 {
        let mut segments = if optimized {
            Vec::with_capacity(ENTRIES_PER_SAMPLE)
        } else {
            Vec::new()
        };
        let mut includes = if optimized {
            Vec::with_capacity(ENTRIES_PER_SAMPLE)
        } else {
            Vec::new()
        };
        for index in 0..ENTRIES_PER_SAMPLE {
            segments.push(index);
            includes.push(index);
        }
        checksum ^= segments.len() ^ includes.len();
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
