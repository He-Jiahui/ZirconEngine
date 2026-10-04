use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const GLYPHS_PER_SAMPLE: usize = 512;

#[test]
fn vertical_sdf_advance_collection_reserves_glyph_capacity() {
    let source = include_str!("../text.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("SDF text vertex implementation");

    assert!(implementation.contains("Vec::with_capacity(glyphs.len())"));
    assert!(
        implementation.contains("for (character, glyph) in text.text.chars().zip(glyphs.iter())")
    );
    assert!(implementation.contains("natural_advances.push(vertical_glyph_advance("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cl_runtime_vertical_sdf_advance_capacity_p95() {
    let text = "A".repeat(GLYPHS_PER_SAMPLE);
    let glyphs = vec![7.0_f32; GLYPHS_PER_SAMPLE];
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&text, &glyphs, false));
            optimized.push(measure(&text, &glyphs, true));
        } else {
            optimized.push(measure(&text, &glyphs, true));
            legacy.push(measure(&text, &glyphs, false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME388_VERTICAL_SDF_ADVANCE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} glyphs_per_sample={GLYPHS_PER_SAMPLE} text_encoding=ascii legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(text: &str, glyphs: &[f32], use_capacity: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..1_024 {
        let advances = if use_capacity {
            let mut advances = Vec::with_capacity(glyphs.len());
            for (character, glyph) in black_box(text).chars().zip(black_box(glyphs)) {
                advances.push((character as u32 as f32 + glyph).max(0.0));
            }
            advances
        } else {
            black_box(text)
                .chars()
                .zip(black_box(glyphs))
                .map(|(character, glyph)| (character as u32 as f32 + glyph).max(0.0))
                .collect::<Vec<_>>()
        };
        checksum ^= advances.len();
        black_box(advances);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], p: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
