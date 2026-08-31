use std::hint::black_box;
use std::time::Instant;

use super::{PostProcessDepthSamplingMode, contains_ascii_case_insensitive};

const SAMPLE_PAIRS: usize = 21;
const CALLS_PER_SAMPLE: usize = 262_144;
const BACKEND_NAMES: [&str; 8] = [
    "wgpu(GL)",
    "wgpu(WebGl)",
    "wgpu(ANGLE)",
    "wgpu(vulkan)",
    "wgpu(DX12)",
    "wgpu(metal)",
    "custom-opengl-backend",
    "software-rasterizer",
];

#[test]
fn optimization_batch_20260826dc_runtime146_depth_backend_match_preserves_legacy_results() {
    for backend_name in BACKEND_NAMES {
        assert_eq!(
            PostProcessDepthSamplingMode::for_backend_name(backend_name),
            legacy_mode(backend_name),
            "backend={backend_name}"
        );
    }
    assert!(contains_ascii_case_insensitive("wgpu(AnGlE)", "angle"));
    assert!(!contains_ascii_case_insensitive("wgpu(vulkan)", "gl"));
    assert!(!contains_ascii_case_insensitive("wgpu(gl)", ""));
}

#[test]
fn optimization_batch_20260826dc_runtime146_depth_backend_match_avoids_lowercase_buffer() {
    let source = include_str!("../depth_sampling_mode.rs");

    assert!(source.contains("contains_ascii_case_insensitive(backend_name, \"gl\")"));
    assert!(source.contains("window.eq_ignore_ascii_case(expected.as_bytes())"));
    assert!(!source.contains("backend_name.to_ascii_lowercase()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dc_runtime146_depth_backend_borrowed_match_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_mode));
            optimized_samples.push(measure(PostProcessDepthSamplingMode::for_backend_name));
        } else {
            optimized_samples.push(measure(PostProcessDepthSamplingMode::for_backend_name));
            legacy_samples.push(measure(legacy_mode));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME146_DEPTH_BACKEND_BORROWED_MATCH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
calls_per_sample={CALLS_PER_SAMPLE} backend_names={} \
legacy_lowercase_allocations_per_sample={CALLS_PER_SAMPLE} \
optimized_lowercase_allocations_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        BACKEND_NAMES.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed backend match P95 {optimized_p95_ns}ns must be at most 70% of lowercase-buffer P95 {legacy_p95_ns}ns"
    );
}

fn legacy_mode(backend_name: &str) -> PostProcessDepthSamplingMode {
    let normalized = backend_name.to_ascii_lowercase();
    if normalized.contains("gl") || normalized.contains("angle") {
        PostProcessDepthSamplingMode::ViewportDepthFallback
    } else {
        PostProcessDepthSamplingMode::RawDepthTexture
    }
}

fn measure(classify: fn(&str) -> PostProcessDepthSamplingMode) -> u128 {
    let started = Instant::now();
    let mut fallback_count = 0usize;
    for index in 0..CALLS_PER_SAMPLE {
        fallback_count += matches!(
            black_box(classify(black_box(
                BACKEND_NAMES[index % BACKEND_NAMES.len()]
            ))),
            PostProcessDepthSamplingMode::ViewportDepthFallback
        ) as usize;
    }
    black_box(fallback_count);
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
