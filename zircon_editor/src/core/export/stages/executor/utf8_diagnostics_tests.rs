use std::hint::black_box;
use std::time::Instant;

use super::*;

const LOG_BYTE_COUNT: usize = 1024 * 1024;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826cp_editor79_utf8_diagnostics_preserve_valid_empty_and_lossy_contracts(
) {
    let diagnostics = command_diagnostics(
        b"compile complete".to_vec(),
        vec![b'b', b'a', b'd', 0xff, b'l', b'o', b'g'],
    );

    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0], "compile complete");
    assert_eq!(diagnostics[1].as_bytes(), b"bad\xef\xbf\xbdlog");
    assert!(command_diagnostics(Vec::new(), Vec::new()).is_empty());
}

#[test]
fn optimization_batch_20260826cp_editor79_valid_utf8_adopts_owned_command_buffers() {
    let source = include_str!("../executor.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;

    assert!(source.contains("String::from_utf8(bytes)"));
    assert!(source.contains("error.as_bytes()"));
    assert!(!source.contains("String::from_utf8_lossy(&bytes).into_owned()"));
}

fn legacy_command_diagnostics(stdout: Vec<u8>, stderr: Vec<u8>) -> Vec<String> {
    [stdout, stderr]
        .into_iter()
        .filter(|bytes| !bytes.is_empty())
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .collect()
}

fn elapsed_ns(bytes: &[u8], decode: fn(Vec<u8>, Vec<u8>) -> Vec<String>) -> u128 {
    let stdout = bytes.to_vec();
    let stderr = bytes.to_vec();
    let started = Instant::now();
    let diagnostics = decode(stdout, stderr);
    assert_eq!(
        black_box(diagnostics.iter().map(String::len).sum::<usize>()),
        LOG_BYTE_COUNT * 2
    );
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &mut [u128], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260826cp_editor79_utf8_buffer_adoption_performance_evidence() {
    let bytes = vec![b'x'; LOG_BYTE_COUNT];
    for _ in 0..3 {
        assert_eq!(
            black_box(legacy_command_diagnostics(bytes.clone(), bytes.clone())),
            command_diagnostics(bytes.clone(), bytes.clone())
        );
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(elapsed_ns(&bytes, legacy_command_diagnostics));
            optimized_samples.push(elapsed_ns(&bytes, command_diagnostics));
        } else {
            optimized_samples.push(elapsed_ns(&bytes, command_diagnostics));
            legacy_samples.push(elapsed_ns(&bytes, legacy_command_diagnostics));
        }
    }

    let legacy_p50_ns = nearest_rank(&mut legacy_samples.clone(), 50);
    let legacy_p95_ns = nearest_rank(&mut legacy_samples, 95);
    let optimized_p50_ns = nearest_rank(&mut optimized_samples.clone(), 50);
    let optimized_p95_ns = nearest_rank(&mut optimized_samples, 95);
    println!(
        "EDITOR79_EXPORT_LOG_UTF8_BUFFER_ADOPTION_BENCH_V1 sample_pairs={} bytes_per_stream={} stream_count=2 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        SAMPLE_PAIRS,
        LOG_BYTE_COUNT,
        legacy_p50_ns,
        legacy_p95_ns,
        optimized_p50_ns,
        optimized_p95_ns,
        legacy_samples,
        optimized_samples,
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "owned UTF-8 buffer adoption p95 must be at least 30% below lossy-copy decoding: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
