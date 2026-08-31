use std::hint::black_box;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use serde::Serialize;

use super::write_pretty_json;

const ARTIFACT_COUNT: usize = 20_000;
const WARMUP_ROUNDS: usize = 3;
const SAMPLE_ROUNDS: usize = 51;
const REQUIRED_PERCENT: u128 = 80;

#[derive(Serialize)]
struct BenchmarkReceipt {
    schema_version: u32,
    receipt_kind: &'static str,
    artifacts: Vec<BenchmarkArtifact>,
}

#[derive(Serialize)]
struct BenchmarkArtifact {
    logical_name: String,
    relative_path: String,
    sha256: &'static str,
    byte_length: u64,
}

#[derive(Default)]
struct CountingWriter {
    written: usize,
}

impl Write for CountingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.written = self.written.saturating_add(bytes.len());
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
#[ignore = "release-only performance evidence"]
fn streaming_receipt_write_performance_evidence() {
    let receipt = benchmark_receipt();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_writes(&receipt, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_writes(&receipt, round % 2 == 0);
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

    println!(
        "TOOLING15_STREAMING_RECEIPT_WRITE_BENCH_V1 artifacts={ARTIFACT_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 20%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 20%"
    );
}

fn benchmark_receipt() -> BenchmarkReceipt {
    BenchmarkReceipt {
        schema_version: 1,
        receipt_kind: "zircon_product_receipt",
        artifacts: (0..ARTIFACT_COUNT)
            .map(|index| BenchmarkArtifact {
                logical_name: format!("runtime-artifact-{index:05}"),
                relative_path: format!("runtime/shard_{:04}/artifact_{index:05}.dll", index % 512),
                sha256: "0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF",
                byte_length: index as u64 * 4_096,
            })
            .collect(),
    }
}

fn measure_writes(
    receipt: &BenchmarkReceipt,
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    if baseline_first {
        let baseline = measure_buffered_vector(receipt);
        let candidate = measure_streaming(receipt);
        (baseline, candidate)
    } else {
        let candidate = measure_streaming(receipt);
        let baseline = measure_buffered_vector(receipt);
        (baseline, candidate)
    }
}

fn measure_buffered_vector(receipt: &BenchmarkReceipt) -> (usize, Duration) {
    let started = Instant::now();
    let contents = serde_json::to_vec_pretty(black_box(receipt)).unwrap();
    let mut destination = CountingWriter::default();
    destination.write_all(&contents).unwrap();
    (black_box(destination.written), started.elapsed())
}

fn measure_streaming(receipt: &BenchmarkReceipt) -> (usize, Duration) {
    let started = Instant::now();
    let mut destination = CountingWriter::default();
    write_pretty_json(&mut destination, black_box(receipt)).unwrap();
    (black_box(destination.written), started.elapsed())
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    samples[(samples.len() - 1) * percentile / 100]
}
