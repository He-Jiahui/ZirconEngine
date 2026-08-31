use std::hint::black_box;
use std::time::Instant;

use super::{AssetUri, shader_stub_source_header};

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 100_000;

#[test]
fn optimization_batch_20260829ah_runtime307_stub_headers_preserve_exact_text() {
    let uri = AssetUri::parse("package://zircon/shaders/surface.wgsl#vertex").unwrap();

    assert_eq!(
        shader_stub_source_header("zr_surface", None),
        "// Zircon shader IDE stub: zr_surface\n\n"
    );
    assert_eq!(
        shader_stub_source_header("zr_surface", Some(&uri)),
        "// Zircon shader IDE stub: zr_surface\n\
// Source asset: package://zircon/shaders/surface.wgsl#vertex\n\n"
    );
}

#[test]
fn optimization_batch_20260829ah_runtime307_stub_headers_write_one_buffer() {
    let source = include_str!("../ide_env_generation.rs");
    let builder = source
        .split("fn shader_stub_source_header")
        .nth(1)
        .expect("stub header builder")
        .split("fn shader_source_hash")
        .next()
        .expect("stub header builder body");

    assert!(builder.contains("String::with_capacity"));
    assert_eq!(builder.matches("writeln!(header").count(), 2);
    assert!(!builder.contains("let mut header = format!"));
    assert!(!builder.contains("push_str(&format!"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ah_runtime307_single_buffer_shader_ide_stub_header_bench() {
    let uri = AssetUri::parse(
        "package://zircon/shaders/material/surface_definition_with_a_long_name.wgsl#vertex",
    )
    .unwrap();
    let import_path = "zircon::material::surface_definition_with_a_long_name";
    assert_eq!(
        shader_stub_source_header(import_path, Some(&uri)),
        legacy_shader_stub_source_header(import_path, Some(&uri))
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, import_path, &uri));
            optimized_samples.push(measure(true, import_path, &uri));
        } else {
            optimized_samples.push(measure(true, import_path, &uri));
            legacy_samples.push(measure(false, import_path, &uri));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME307_SINGLE_BUFFER_SHADER_IDE_STUB_HEADER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} legacy_independent_string_buffers_per_build=2 \
optimized_independent_string_buffers_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_shader_stub_source_header(import_path: &str, source_uri: Option<&AssetUri>) -> String {
    let mut header = format!("// Zircon shader IDE stub: {import_path}\n");
    if let Some(source_uri) = source_uri {
        header.push_str(&format!("// Source asset: {source_uri}\n"));
    }
    header.push('\n');
    header
}

fn measure(optimized: bool, import_path: &str, source_uri: &AssetUri) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let header = if optimized {
            shader_stub_source_header(black_box(import_path), Some(black_box(source_uri)))
        } else {
            legacy_shader_stub_source_header(black_box(import_path), Some(black_box(source_uri)))
        };
        checksum = checksum.wrapping_add(black_box(header).len());
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
