use std::hint::black_box;
use std::io::{self, Read, Write};
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const READS_PER_SAMPLE: usize = 256;
const READER_BLOCK_BYTES: usize = 4 * 1_024;

#[test]
fn optimization_batch_20260826go_editor181_full_capture_uses_chunk_capacity() {
    let (mut stdout_writer, _stderr_writer, mut readers) =
        create_output_capture("capacity contract").expect("capture should open");
    let expected = vec![b'x'; OUTPUT_CAPTURE_READ_CHUNK_BYTES as usize];
    stdout_writer.write_all(&expected).expect("stdout write");

    let bytes = readers
        .stdout
        .read_available()
        .expect("capture read should succeed");

    assert_eq!(bytes, expected);
    assert_eq!(bytes.capacity(), OUTPUT_CAPTURE_READ_CHUNK_BYTES as usize);

    let mut empty = io::empty();
    let empty_bytes = read_capture_chunk(&mut empty).expect("empty capture read");
    assert!(empty_bytes.is_empty());
    assert_eq!(empty_bytes.capacity(), 0);
}

#[test]
fn optimization_batch_20260826go_editor181_capture_preallocates_read_budget() {
    let source = include_str!("../output_capture.rs");

    assert!(source.contains("read_capture_chunk(self.file.by_ref())"));
    assert!(source.contains("let mut prefix = [0; OUTPUT_CAPTURE_READ_PREFIX_BYTES]"));
    assert!(source.contains("let mut bytes = Vec::with_capacity(capacity);"));
    assert!(source.contains("reader.take(OUTPUT_CAPTURE_READ_CHUNK_BYTES)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826go_editor181_output_capture_buffer_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR181_OUTPUT_CAPTURE_BUFFER_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
reads_per_sample={READS_PER_SAMPLE} read_budget_bytes={OUTPUT_CAPTURE_READ_CHUNK_BYTES} \
reader_block_bytes={READER_BLOCK_BYTES} legacy_initial_capacity=0 \
optimized_initial_capacity={OUTPUT_CAPTURE_READ_CHUNK_BYTES} legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(preallocate: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for read in 0..READS_PER_SAMPLE {
        let mut reader = FixedChunkReader::new(OUTPUT_CAPTURE_READ_CHUNK_BYTES as usize);
        let bytes = if preallocate {
            read_capture_chunk(&mut reader).expect("fixed optimized read")
        } else {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).expect("fixed legacy read");
            bytes
        };
        checksum ^= black_box(bytes.len() ^ bytes.capacity() ^ read);
        black_box(bytes);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

struct FixedChunkReader {
    remaining: usize,
}

impl FixedChunkReader {
    const fn new(remaining: usize) -> Self {
        Self { remaining }
    }
}

impl Read for FixedChunkReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let count = self.remaining.min(buffer.len()).min(READER_BLOCK_BYTES);
        buffer[..count].fill(b'x');
        self.remaining -= count;
        Ok(count)
    }
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
