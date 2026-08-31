use std::fs::File;
use std::hint::black_box;
use std::io::Seek;
use std::time::{Duration, Instant};

use super::{
    digest_open_file_handle_bytes, digest_open_file_handle_bytes_with_buffer, FileDigestBuffer,
};

const DIGEST_REPETITIONS: usize = 512;
const WARMUP_ROUNDS: usize = 3;
const SAMPLE_ROUNDS: usize = 51;
const REQUIRED_PERCENT: u128 = 90;

#[test]
#[ignore = "release-only performance evidence"]
fn shared_product_digest_buffer_performance_evidence() {
    let path = std::env::temp_dir().join(format!(
        "cargo-zircon-product-digest-buffer-benchmark-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&path, vec![0xA5_u8; 256]).unwrap();
    let mut file = File::open(&path).unwrap();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_digest_buffers(&mut file, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_digest_buffers(&mut file, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile(&baseline_samples, 50);
    let baseline_p95 = percentile(&baseline_samples, 95);
    let candidate_p50 = percentile(&candidate_samples, 50);
    let candidate_p95 = percentile(&candidate_samples, 95);
    drop(file);
    std::fs::remove_file(path).unwrap();

    println!(
        "TOOLING15_PRODUCT_DIGEST_BUFFER_REUSE_BENCH_V1 files={DIGEST_REPETITIONS} bytes_per_file=256 rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 10%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 10%"
    );
}

fn measure_digest_buffers(
    file: &mut File,
    baseline_first: bool,
) -> ((u8, Duration), (u8, Duration)) {
    if baseline_first {
        let baseline = measure_per_file_buffers(file);
        let candidate = measure_shared_buffer(file);
        (baseline, candidate)
    } else {
        let candidate = measure_shared_buffer(file);
        let baseline = measure_per_file_buffers(file);
        (baseline, candidate)
    }
}

fn measure_per_file_buffers(file: &mut File) -> (u8, Duration) {
    let started = Instant::now();
    let mut checksum = 0_u8;
    for _ in 0..DIGEST_REPETITIONS {
        file.rewind().unwrap();
        checksum ^= digest_open_file_handle_bytes(file).unwrap().sha256[0];
    }
    (black_box(checksum), started.elapsed())
}

fn measure_shared_buffer(file: &mut File) -> (u8, Duration) {
    let started = Instant::now();
    let mut checksum = 0_u8;
    let mut buffer = FileDigestBuffer::new();
    for _ in 0..DIGEST_REPETITIONS {
        file.rewind().unwrap();
        checksum ^= digest_open_file_handle_bytes_with_buffer(file, &mut buffer)
            .unwrap()
            .sha256[0];
    }
    (black_box(checksum), started.elapsed())
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    samples[(samples.len() - 1) * percentile / 100]
}
