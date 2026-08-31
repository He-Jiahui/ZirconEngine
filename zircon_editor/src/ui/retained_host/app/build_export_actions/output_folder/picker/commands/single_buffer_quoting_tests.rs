use std::hint::black_box;
use std::time::Instant;

#[cfg(target_os = "macos")]
use super::apple_script_string;
#[cfg(target_os = "windows")]
use super::powershell_single_quoted;

const SAMPLE_PAIRS: usize = 31;
const QUOTES_PER_SAMPLE: usize = 100_000;

#[test]
fn optimization_batch_20260829ab_editor247_folder_picker_quoting_preserves_bytes() {
    #[cfg(target_os = "windows")]
    for value in [
        "C:\\Projects\\Zircon",
        "C:\\Projects\\Owner's Build\\\u{96ea}",
        "'leading and trailing'",
    ] {
        assert_eq!(
            powershell_single_quoted(value),
            legacy_powershell_single_quoted(value)
        );
    }

    #[cfg(target_os = "macos")]
    for value in [
        "/Projects/Zircon",
        "/Projects/Owner's \"Build\"",
        "\\quoted\\",
    ] {
        assert_eq!(
            apple_script_string(value),
            legacy_apple_script_string(value)
        );
    }
}

#[test]
fn optimization_batch_20260829ab_editor247_folder_picker_quoting_uses_one_buffer() {
    let source = include_str!("../commands.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let powershell = implementation
        .split("fn powershell_single_quoted")
        .nth(1)
        .and_then(|body| body.split("fn apple_script_string").next())
        .expect("PowerShell quoting helper");
    let apple_script = implementation
        .split("fn apple_script_string")
        .nth(1)
        .expect("AppleScript quoting helper");

    assert!(powershell.contains("String::with_capacity"));
    assert!(powershell.contains("quoted.push"));
    assert!(!powershell.contains(".replace("));
    assert!(!powershell.contains("format!("));
    assert!(apple_script.contains("String::with_capacity"));
    assert!(apple_script.contains("quoted.push"));
    assert!(!apple_script.contains(".replace("));
    assert!(!apple_script.contains("format!("));
}

#[cfg(target_os = "windows")]
#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ab_editor247_single_buffer_folder_picker_quoting_bench() {
    let value = "C:\\Projects\\Owner's Long Export Build\\Scenes\\Production's Final Scene";
    assert_eq!(
        powershell_single_quoted(value),
        legacy_powershell_single_quoted(value)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, value));
            optimized_samples.push(measure(true, value));
        } else {
            optimized_samples.push(measure(true, value));
            legacy_samples.push(measure(false, value));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR247_SINGLE_BUFFER_FOLDER_PICKER_QUOTING_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
quotes_per_sample={QUOTES_PER_SAMPLE} input_bytes={} embedded_quote_count=2 \
legacy_result_allocations_per_quote=2 optimized_result_allocations_per_quote=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        value.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

#[cfg(target_os = "windows")]
fn legacy_powershell_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(target_os = "macos")]
fn legacy_apple_script_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(target_os = "windows")]
fn measure(optimized: bool, value: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..QUOTES_PER_SAMPLE {
        let quoted = if optimized {
            powershell_single_quoted(black_box(value))
        } else {
            legacy_powershell_single_quoted(black_box(value))
        };
        checksum = checksum.wrapping_add(black_box(quoted).len());
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
