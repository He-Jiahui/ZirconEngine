use std::hint::black_box;
use std::io::{self, Write};
use std::time::Instant;

use serde::Serialize;

use super::{RuntimeRequestWriter, REQUEST_WRITER_INITIAL_CAPACITY_BYTES};
use zircon_runtime_interface::ZrRuntimePayloadLimitV1;

const SAMPLE_PAIRS: usize = 21;
const ENCODINGS_PER_SAMPLE: usize = 128;
const TARGET_P95_RATIO_PERCENT: u128 = 80;

#[derive(Serialize)]
struct BenchmarkRequest {
    values: Vec<u32>,
}

#[test]
fn app08_request_encoding_matches_serde_json_bytes_and_limits_clock_checks() {
    let request = benchmark_request();
    let expected = serde_json::to_vec(&request).unwrap();
    let mut writer = RuntimeRequestWriter::new(benchmark_limit());
    serde_json::to_writer(&mut writer, &request).unwrap();

    assert_eq!(writer.bytes, expected);
    assert!(writer.deadline_checks <= 2);
    assert_eq!(writer.capacity_growths, 0);
    assert_eq!(
        writer.bytes.capacity(),
        REQUEST_WRITER_INITIAL_CAPACITY_BYTES
    );
}

#[test]
#[ignore = "managed release performance evidence"]
fn app08_request_encoding_release_benchmark_evidence() {
    let request = benchmark_request();
    let expected = serde_json::to_vec(&request).unwrap();

    let legacy_probe = encode_legacy(&request);
    let optimized_probe = encode_optimized(&request);
    assert_eq!(legacy_probe.bytes, expected);
    assert_eq!(optimized_probe.bytes, expected);
    assert!(legacy_probe.deadline_checks > optimized_probe.deadline_checks);
    assert!(legacy_probe.capacity_growths > optimized_probe.capacity_growths);

    black_box(measure_legacy(&request));
    black_box(measure_optimized(&request));
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&request));
            optimized_samples.push(measure_optimized(&request));
        } else {
            optimized_samples.push(measure_optimized(&request));
            legacy_samples.push(measure_legacy(&request));
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank(&optimized_samples, 95);
    println!(
        "APP08_REQUEST_ENCODING_PERF payload_bytes={} values=256 pairs=21 encodings_per_sample=128 order=alternating percentile=nearest-rank legacy_deadline_checks_per_encode={} optimized_deadline_checks_per_encode={} legacy_capacity_growths_per_encode={} optimized_capacity_growths_per_encode={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        expected.len(),
        legacy_probe.deadline_checks,
        optimized_probe.deadline_checks,
        legacy_probe.capacity_growths,
        optimized_probe.capacity_growths,
        nearest_rank(&legacy_samples, 50),
        legacy_p95_ns,
        nearest_rank(&optimized_samples, 50),
        optimized_p95_ns,
        legacy_samples,
        optimized_samples,
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_RATIO_PERCENT),
        "amortized request encoding P95 {optimized_p95_ns}ns must be at most {TARGET_P95_RATIO_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_request() -> BenchmarkRequest {
    BenchmarkRequest {
        values: (0..256).collect(),
    }
}

fn benchmark_limit() -> ZrRuntimePayloadLimitV1 {
    ZrRuntimePayloadLimitV1::new(256 * 1024, 512, 25_000)
}

struct EncodingProbe {
    bytes: Vec<u8>,
    deadline_checks: usize,
    capacity_growths: usize,
}

fn encode_optimized(request: &BenchmarkRequest) -> EncodingProbe {
    let mut writer = RuntimeRequestWriter::new(benchmark_limit());
    serde_json::to_writer(&mut writer, request).unwrap();
    writer.check_deadline().unwrap();
    EncodingProbe {
        bytes: writer.bytes,
        deadline_checks: writer.deadline_checks,
        capacity_growths: writer.capacity_growths,
    }
}

fn encode_legacy(request: &BenchmarkRequest) -> EncodingProbe {
    let mut writer = LegacyRequestWriter::default();
    serde_json::to_writer(&mut writer, request).unwrap();
    writer.check_deadline();
    EncodingProbe {
        bytes: writer.bytes,
        deadline_checks: writer.deadline_checks,
        capacity_growths: writer.capacity_growths,
    }
}

fn measure_legacy(request: &BenchmarkRequest) -> u128 {
    let started = Instant::now();
    let mut bytes = 0_usize;
    for _ in 0..ENCODINGS_PER_SAMPLE {
        bytes = bytes.saturating_add(encode_legacy(black_box(request)).bytes.len());
    }
    black_box(bytes);
    started.elapsed().as_nanos() / ENCODINGS_PER_SAMPLE as u128
}

fn measure_optimized(request: &BenchmarkRequest) -> u128 {
    let started = Instant::now();
    let mut bytes = 0_usize;
    for _ in 0..ENCODINGS_PER_SAMPLE {
        bytes = bytes.saturating_add(encode_optimized(black_box(request)).bytes.len());
    }
    black_box(bytes);
    started.elapsed().as_nanos() / ENCODINGS_PER_SAMPLE as u128
}

struct LegacyRequestWriter {
    bytes: Vec<u8>,
    started: Instant,
    deadline_checks: usize,
    capacity_growths: usize,
}

impl Default for LegacyRequestWriter {
    fn default() -> Self {
        Self {
            bytes: Vec::new(),
            started: Instant::now(),
            deadline_checks: 0,
            capacity_growths: 0,
        }
    }
}

impl LegacyRequestWriter {
    fn check_deadline(&mut self) {
        self.deadline_checks = self.deadline_checks.saturating_add(1);
        black_box(self.started.elapsed());
    }
}

impl Write for LegacyRequestWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.check_deadline();
        let prior_capacity = self.bytes.capacity();
        self.bytes.extend_from_slice(bytes);
        if self.bytes.capacity() > prior_capacity {
            self.capacity_growths = self.capacity_growths.saturating_add(1);
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
