use std::hint::black_box;
use std::time::Instant;

use super::{failed_cargo_invocation_diagnostic, successful_cargo_invocation_diagnostic};

const SAMPLE_PAIRS: usize = 21;
const FORMATS_PER_SAMPLE: usize = 65_536;

#[test]
fn optimization_batch_20260826dr_editor107_export_command_diagnostics_preserve_output() {
    let command = vec![
        "cargo".to_string(),
        "build".to_string(),
        "--release".to_string(),
    ];
    assert_eq!(
        successful_cargo_invocation_diagnostic("export cargo build", &command),
        "export cargo build succeeded: cargo build --release"
    );
    assert_eq!(
        failed_cargo_invocation_diagnostic("export cargo build", Some(101), &command),
        "export cargo build failed with status Some(101): cargo build --release"
    );
    assert_eq!(
        failed_cargo_invocation_diagnostic("export cargo build", None, &[]),
        "export cargo build failed with status None: "
    );
    assert_eq!(
        failed_cargo_invocation_diagnostic("export cargo build", Some(i32::MIN), &command),
        "export cargo build failed with status Some(-2147483648): cargo build --release"
    );
}

#[test]
fn optimization_batch_20260826dr_editor107_export_command_diagnostics_use_one_buffer() {
    let command = fixture_command();
    let diagnostic = successful_cargo_invocation_diagnostic("export cargo build", &command);
    assert_eq!(diagnostic.capacity(), diagnostic.len());

    let source = include_str!("../diagnostics.rs");
    assert!(source.contains("joined_command_len(command)"));
    assert!(source.contains("push_command(&mut diagnostic, command);"));
    assert!(source.contains("write!(&mut diagnostic, \"{status_code:?}\")"));
    assert!(!source.contains("invocation.command.join(\" \")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dr_editor107_export_command_direct_buffer_bench() {
    let command = fixture_command();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&command, legacy_success));
            optimized_samples.push(measure(&command, optimized_success));
        } else {
            optimized_samples.push(measure(&command, optimized_success));
            legacy_samples.push(measure(&command, legacy_success));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR107_EXPORT_COMMAND_DIRECT_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
formats_per_sample={FORMATS_PER_SAMPLE} legacy_allocations_per_format=2 \
optimized_allocations_per_format=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "direct export command diagnostic P95 {optimized_p95_ns}ns must be at most 70% of join formatting P95 {legacy_p95_ns}ns"
    );
}

fn fixture_command() -> Vec<String> {
    (0..16)
        .map(|index| format!("--export-option-production-{index:02}=enabled"))
        .collect()
}

fn legacy_success(command: &[String]) -> String {
    format!("export cargo build succeeded: {}", command.join(" "))
}

fn optimized_success(command: &[String]) -> String {
    successful_cargo_invocation_diagnostic("export cargo build", command)
}

fn measure(command: &[String], render: fn(&[String]) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..FORMATS_PER_SAMPLE {
        checksum ^= black_box(render(black_box(command))).len();
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
