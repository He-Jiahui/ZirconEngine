use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 21;
const LAYOUTS_PER_SAMPLE: usize = 32;
const LINES_PER_LAYOUT: usize = 256;
const GLYPHS_PER_LINE: usize = 64;

#[test]
fn optimization_batch_20260826ge_editor172_checked_capacity_covers_all_glyphs() {
    let line_lengths = [3_usize, 5, 8];
    let glyph_capacity = line_lengths
        .iter()
        .try_fold(0_usize, |capacity, glyphs| capacity.checked_add(*glyphs))
        .expect("bounded glyph capacity");
    let mut glyphs = Vec::with_capacity(glyph_capacity);
    for (line, glyph_count) in line_lengths.into_iter().enumerate() {
        glyphs.extend((0..glyph_count).map(|glyph| (line, glyph)));
    }

    assert_eq!(glyphs.len(), 16);
    assert!(glyphs.capacity() >= glyphs.len());
    assert_eq!(
        [usize::MAX, 1]
            .into_iter()
            .try_fold(0_usize, usize::checked_add),
        None
    );
}

#[test]
fn optimization_batch_20260826ge_editor172_artifact_preflights_before_glyph_allocation() {
    let source = include_str!("../artifact.rs");
    let faces = source
        .find("artifact_layout.artifact_raster_faces()?")
        .expect("raster face preflight");
    let capacity = source
        .find("let glyph_capacity = lines.iter().try_fold")
        .expect("checked glyph capacity preflight");
    let allocation = source
        .find("let mut glyphs = Vec::with_capacity(glyph_capacity);")
        .expect("glyph allocation");

    assert!(faces < capacity && capacity < allocation);
    assert!(source.contains("capacity.checked_add(glyphs.len())"));
    assert!(source.contains("let mut font_indices = HashMap::new();"));
    assert!(source.contains("let mut raster_fonts = Vec::new();"));
    assert!(!source.contains("let mut glyphs = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ge_editor172_artifact_glyph_capacity_bench() {
    let line_lengths = [GLYPHS_PER_LINE; LINES_PER_LAYOUT];
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&line_lengths, false));
            optimized_samples.push(measure(&line_lengths, true));
        } else {
            optimized_samples.push(measure(&line_lengths, true));
            legacy_samples.push(measure(&line_lengths, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR172_ARTIFACT_GLYPH_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
layouts_per_sample={LAYOUTS_PER_SAMPLE} lines_per_layout={LINES_PER_LAYOUT} \
glyphs_per_line={GLYPHS_PER_LINE} legacy_preallocated_glyph_vectors=0 \
optimized_preallocated_glyph_vectors={LAYOUTS_PER_SAMPLE} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(line_lengths: &[usize; LINES_PER_LAYOUT], reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for layout in 0..LAYOUTS_PER_SAMPLE {
        let mut glyphs = if reserve {
            let glyph_capacity = line_lengths
                .iter()
                .try_fold(0_usize, |capacity, glyphs| capacity.checked_add(*glyphs))
                .expect("bounded benchmark glyph capacity");
            Vec::with_capacity(glyph_capacity)
        } else {
            Vec::new()
        };
        for (line, glyph_count) in line_lengths.iter().copied().enumerate() {
            for glyph in 0..glyph_count {
                let value = layout ^ line ^ glyph;
                glyphs.push([value; 8]);
            }
        }
        checksum ^= black_box(glyphs.capacity() ^ glyphs.len());
        black_box(&glyphs);
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
