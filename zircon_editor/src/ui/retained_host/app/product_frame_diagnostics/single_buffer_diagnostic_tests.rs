use std::hint::black_box;
use std::time::Instant;

use crate::ui::host::project_access::percent_encode_diagnostic_token;

use super::build_product_frame_diagnostic;

const SAMPLE_PAIRS: usize = 31;
const DIAGNOSTICS_PER_SAMPLE: usize = 20_000;

#[test]
fn optimization_batch_20260829aa_editor246_product_frame_diagnostic_preserves_bytes() {
    let translation = values(["1.25", "-2.5", "3 & 4"]);
    let scale = values(["1", "1/2", "\u{96ea}"]);
    let optimized = build_product_frame_diagnostic(
        "E:/Projects/Zircon Build",
        &42_u64,
        "Main Camera/Primary",
        &translation,
        &scale,
    );

    assert_eq!(
        optimized,
        legacy_product_frame_diagnostic(
            "E:/Projects/Zircon Build",
            &42_u64,
            "Main Camera/Primary",
            &translation,
            &scale,
        )
    );
    assert!(optimized.contains("project_path=E%3A%2FProjects%2FZircon%20Build"));
    assert!(optimized.contains("selected_node_name=Main%20Camera%2FPrimary"));
    assert!(optimized.contains("inspector_scale_z=%E9%9B%AA"));
}

#[test]
fn optimization_batch_20260829aa_editor246_product_frame_diagnostic_uses_one_buffer() {
    let source = include_str!("../product_frame_diagnostics.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let body = implementation
        .split("fn build_product_frame_diagnostic")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn emit_product_frame_log").next())
        .expect("product frame diagnostic builder");

    assert!(implementation.contains("struct PercentEncodedDiagnosticToken"));
    assert!(body.contains("String::with_capacity"));
    assert!(body.contains("PercentEncodedDiagnosticToken("));
    assert!(body.contains("&mut diagnostic"));
    assert!(!body.contains("percent_encode_diagnostic_token("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829aa_editor246_single_buffer_product_frame_diagnostic_bench() {
    let project_path = "E:/Projects/Zircon Engine/bench fixtures/product frame";
    let selected_name = "Virtual Geometry Camera / Production";
    let translation = values(["1024.125", "-2048.5", "32 & 64"]);
    let scale = values(["1.0", "0.5/2", "\u{96ea}-scale"]);
    let selected_node_id = u64::MAX;
    assert_eq!(
        build_product_frame_diagnostic(
            project_path,
            &selected_node_id,
            selected_name,
            &translation,
            &scale,
        ),
        legacy_product_frame_diagnostic(
            project_path,
            &selected_node_id,
            selected_name,
            &translation,
            &scale,
        )
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(
                false,
                project_path,
                selected_node_id,
                selected_name,
                &translation,
                &scale,
            ));
            optimized_samples.push(measure(
                true,
                project_path,
                selected_node_id,
                selected_name,
                &translation,
                &scale,
            ));
        } else {
            optimized_samples.push(measure(
                true,
                project_path,
                selected_node_id,
                selected_name,
                &translation,
                &scale,
            ));
            legacy_samples.push(measure(
                false,
                project_path,
                selected_node_id,
                selected_name,
                &translation,
                &scale,
            ));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR246_SINGLE_BUFFER_PRODUCT_FRAME_DIAGNOSTIC_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
diagnostics_per_sample={DIAGNOSTICS_PER_SAMPLE} encoded_token_count=8 \
legacy_result_allocations_per_diagnostic=9 optimized_result_allocations_per_diagnostic=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn values(values: [&str; 3]) -> [String; 3] {
    values.map(str::to_string)
}

fn legacy_product_frame_diagnostic(
    project_path: &str,
    selected_node_id: &impl std::fmt::Display,
    selected_node_name: &str,
    translation: &[String; 3],
    scale: &[String; 3],
) -> String {
    format!(
        "editor_product_frame_diagnostics project_path={} selected_node_id={} selected_node_name={} inspector_translation_x={} inspector_translation_y={} inspector_translation_z={} inspector_scale_x={} inspector_scale_y={} inspector_scale_z={}",
        percent_encode_diagnostic_token(project_path),
        selected_node_id,
        percent_encode_diagnostic_token(selected_node_name),
        percent_encode_diagnostic_token(&translation[0]),
        percent_encode_diagnostic_token(&translation[1]),
        percent_encode_diagnostic_token(&translation[2]),
        percent_encode_diagnostic_token(&scale[0]),
        percent_encode_diagnostic_token(&scale[1]),
        percent_encode_diagnostic_token(&scale[2]),
    )
}

fn measure(
    optimized: bool,
    project_path: &str,
    selected_node_id: u64,
    selected_node_name: &str,
    translation: &[String; 3],
    scale: &[String; 3],
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..DIAGNOSTICS_PER_SAMPLE {
        let diagnostic = if optimized {
            build_product_frame_diagnostic(
                black_box(project_path),
                black_box(&selected_node_id),
                black_box(selected_node_name),
                black_box(translation),
                black_box(scale),
            )
        } else {
            legacy_product_frame_diagnostic(
                black_box(project_path),
                black_box(&selected_node_id),
                black_box(selected_node_name),
                black_box(translation),
                black_box(scale),
            )
        };
        checksum = checksum.wrapping_add(black_box(diagnostic).len());
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
