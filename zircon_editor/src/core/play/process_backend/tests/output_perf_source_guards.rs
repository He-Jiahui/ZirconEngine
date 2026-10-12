use std::hint::black_box;
use std::io::Cursor;
use std::mem;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crossbeam_channel::bounded;

use super::{
    BoundedLineDecoder, OutputByteBudget, PlayOutputCounters, PlayOutputLine, PlayOutputPump,
    PlayOutputStream, PLAY_OUTPUT_DRAIN_BYTE_LIMIT, PLAY_OUTPUT_MAX_LINE_BYTES,
    PLAY_OUTPUT_QUEUE_BYTE_CAPACITY, PLAY_OUTPUT_READ_CHUNK_BYTES,
};

const STREAMING_DECODE_SAMPLE_PAIRS: usize = 17;

fn legacy_push(decoder: &mut BoundedLineDecoder, input: &[u8]) -> Vec<super::DecodedOutputLine> {
    let mut lines = Vec::new();
    for byte in input {
        if *byte == b'\n' {
            lines.push(decoder.finish_line());
        } else if decoder.bytes.len() < decoder.max_bytes {
            decoder.bytes.push(*byte);
        } else {
            decoder.truncated_bytes = decoder.truncated_bytes.saturating_add(1);
        }
    }
    lines
}

fn elapsed_micros(run: impl FnOnce()) -> u128 {
    let started = Instant::now();
    run();
    started.elapsed().as_micros()
}

fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * 95).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

fn pump_with_queued_lines(lines: &[String]) -> (PlayOutputPump, Arc<OutputByteBudget>) {
    let (sender, receiver) = bounded(lines.len());
    let queue_bytes = Arc::new(OutputByteBudget::new(PLAY_OUTPUT_QUEUE_BYTE_CAPACITY));
    for text in lines {
        let queued_text = text.clone();
        let queued_bytes = queued_text
            .capacity()
            .saturating_add(mem::size_of::<PlayOutputLine>());
        assert!(queue_bytes.try_reserve(queued_bytes));
        sender
            .send(PlayOutputLine {
                stream: PlayOutputStream::Stdout,
                text: queued_text,
                truncated_bytes: 0,
                queued_bytes,
                captured_at: Instant::now(),
            })
            .expect("fixture queue should accept each line");
    }
    drop(sender);
    (
        PlayOutputPump {
            receiver,
            reader_completion: super::ReaderCompletion::new(0),
            deferred: Mutex::new(None),
            queue_bytes: Arc::clone(&queue_bytes),
            counters: Arc::new(PlayOutputCounters::default()),
        },
        queue_bytes,
    )
}

#[test]
fn bounded_decoder_truncates_an_unterminated_line_without_retaining_its_tail() {
    let mut decoder = BoundedLineDecoder::new(8);
    assert!(decoder.push(b"0123456789", |_| true));

    let line = decoder
        .finish()
        .expect("unterminated output must flush once");
    assert_eq!(line.text, "01234567");
    assert_eq!(line.truncated_bytes, 2);
}

#[test]
fn optimization_batch_20260826_editor07_play_output_streaming_decode_preserves_line_order() {
    let mut decoder = BoundedLineDecoder::new(64);
    let mut lines = Vec::new();

    assert!(decoder.push(b"first\r\nsecond\nthird", |line| {
        lines.push(line.text);
        true
    }));
    lines.push(decoder.finish().unwrap().text);

    assert_eq!(lines, ["first", "second", "third"]);
}

#[test]
fn optimization_batch_20260826_editor07_play_output_streaming_decode_stops_on_consumer_rejection() {
    let mut decoder = BoundedLineDecoder::new(64);
    let mut lines = Vec::new();

    assert!(!decoder.push(b"first\nsecond\nthird\n", |line| {
        lines.push(line.text);
        lines.len() < 2
    }));

    assert_eq!(lines, ["first", "second"]);
}

#[test]
fn optimization_batch_20260826_editor07_play_output_streaming_decode_has_no_per_chunk_line_vector()
{
    let source = include_str!("../output.rs");
    let decoder = source
        .split_once("impl BoundedLineDecoder")
        .unwrap()
        .1
        .split_once("fn truncate_to_byte_limit")
        .unwrap()
        .0;

    assert!(decoder.contains("mut emit: impl FnMut(DecodedOutputLine) -> bool"));
    assert!(decoder.contains("if !emit(self.finish_line())"));
    assert!(!decoder.contains("let mut lines = Vec::new()"));
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260826_editor07_play_output_streaming_decode_performance_evidence() {
    let input = vec![b'\n'; PLAY_OUTPUT_READ_CHUNK_BYTES];
    let expected_lines = input.len();

    for _ in 0..4 {
        let mut legacy = BoundedLineDecoder::new(PLAY_OUTPUT_MAX_LINE_BYTES);
        black_box(legacy_push(&mut legacy, black_box(&input)));
        let mut optimized = BoundedLineDecoder::new(PLAY_OUTPUT_MAX_LINE_BYTES);
        assert!(optimized.push(black_box(&input), |line| {
            black_box(line);
            true
        }));
    }

    let mut legacy_samples = Vec::with_capacity(STREAMING_DECODE_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(STREAMING_DECODE_SAMPLE_PAIRS);
    for sample_index in 0..STREAMING_DECODE_SAMPLE_PAIRS {
        let measure_legacy = || {
            elapsed_micros(|| {
                let mut decoder = BoundedLineDecoder::new(PLAY_OUTPUT_MAX_LINE_BYTES);
                let lines = legacy_push(&mut decoder, black_box(&input));
                assert_eq!(black_box(lines.len()), expected_lines);
            })
        };
        let measure_optimized = || {
            elapsed_micros(|| {
                let mut decoder = BoundedLineDecoder::new(PLAY_OUTPUT_MAX_LINE_BYTES);
                let mut line_count = 0usize;
                assert!(decoder.push(black_box(&input), |line| {
                    black_box(line);
                    line_count = line_count.saturating_add(1);
                    true
                }));
                assert_eq!(black_box(line_count), expected_lines);
            })
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95 = nearest_rank_p95(&mut legacy_samples);
    let optimized_p95 = nearest_rank_p95(&mut optimized_samples);
    println!(
        "EDITOR07_PLAY_OUTPUT_STREAMING_DECODE_BENCH_V1 sample_pairs={} chunk_bytes={} decoded_lines={} legacy_temporary_line_vectors_per_chunk=1 optimized_temporary_line_vectors_per_chunk=0 legacy_p95_us={} optimized_p95_us={} legacy_samples_us={:?} optimized_samples_us={:?}",
        STREAMING_DECODE_SAMPLE_PAIRS,
        PLAY_OUTPUT_READ_CHUNK_BYTES,
        expected_lines,
        legacy_p95,
        optimized_p95,
        legacy_samples,
        optimized_samples,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(75),
        "streaming decode p95 must be at least 25% below the temporary-vector path: legacy={legacy_p95}us optimized={optimized_p95}us"
    );
}

#[test]
fn output_byte_budget_rejects_overflow_and_releases_consumed_bytes() {
    let budget = OutputByteBudget::new(8);
    assert!(budget.try_reserve(5));
    assert!(!budget.try_reserve(4));
    assert_eq!(budget.used(), 5);

    budget.release(5);
    assert_eq!(budget.used(), 0);
    assert!(budget.try_reserve(8));
}

#[test]
fn play_output_reader_owner_uses_runtime_task_pool_and_completion_accounting() {
    let source = include_str!("../output.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production output implementation");
    let join_handle = ["Join", "Handle"].concat();
    let thread_builder = ["thread", "::", "Builder"].concat();
    assert!(implementation.contains("TaskPoolDescriptor::io()"));
    assert!(implementation
        .contains("map_err(|error| PlayOutputCaptureError::without_readers(error.to_string()))"));
    assert!(implementation.contains("ReaderCompletionGuard"));
    assert!(!implementation.contains(&join_handle));
    assert!(!implementation.contains(&thread_builder));
}

#[test]
fn captured_long_line_reports_truncation_with_bounded_rendered_output() {
    let mut stdout = vec![b'x'; PLAY_OUTPUT_MAX_LINE_BYTES + 1];
    stdout.push(b'\n');
    let pump = PlayOutputPump::capture(Cursor::new(stdout), Cursor::new(Vec::<u8>::new()))
        .expect("fixture readers should start");

    let diagnostics = pump.finish();
    let line = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.starts_with("process.stdout: "))
        .expect("stdout line should be preserved");
    assert!(line.len() <= "process.stdout: ".len() + PLAY_OUTPUT_MAX_LINE_BYTES + 96);
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.starts_with("process.output_truncated_lines=1")));
}

#[test]
fn live_drain_defers_a_line_that_exceeds_the_remaining_byte_budget() {
    let line = "x".repeat(PLAY_OUTPUT_MAX_LINE_BYTES);
    let (pump, queue_bytes) =
        pump_with_queued_lines(&[line.clone(), line.clone(), line.clone(), line.clone()]);

    let first_drain = pump.drain();
    let first_output = first_drain
        .iter()
        .filter(|diagnostic| diagnostic.starts_with("process.stdout: "))
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(first_output.len(), 3);
    assert!(first_output
        .iter()
        .all(|diagnostic| diagnostic.ends_with(line.as_str())));
    assert!(
        queue_bytes.used() > 0,
        "deferred line must keep its reservation"
    );

    let second_drain = pump.drain();
    let expected = format!("process.stdout: {line}");
    assert_eq!(
        second_drain
            .iter()
            .filter(|diagnostic| diagnostic.starts_with("process.stdout: "))
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec![expected.as_str()]
    );
    assert_eq!(queue_bytes.used(), 0);
    assert!(PLAY_OUTPUT_DRAIN_BYTE_LIMIT < 4 * ("process.stdout: ".len() + line.len()));
}

#[test]
fn live_drain_enforces_the_line_budget_without_losing_the_next_line() {
    let lines = (0..65)
        .map(|index| format!("line-{index}"))
        .collect::<Vec<_>>();
    let (pump, queue_bytes) = pump_with_queued_lines(&lines);

    let first_drain = pump.drain();
    let first_output = first_drain
        .iter()
        .filter(|diagnostic| diagnostic.starts_with("process.stdout: "))
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(first_output.len(), 64);
    assert_eq!(
        first_output.first().copied(),
        Some("process.stdout: line-0")
    );
    assert_eq!(
        first_output.last().copied(),
        Some("process.stdout: line-63")
    );
    assert!(queue_bytes.used() > 0);

    assert_eq!(
        pump.drain()
            .iter()
            .filter(|diagnostic| diagnostic.starts_with("process.stdout: "))
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["process.stdout: line-64"]
    );
    assert_eq!(queue_bytes.used(), 0);
}

#[test]
fn live_output_drain_has_a_per_poll_line_budget() {
    let source = include_str!("../output.rs");
    let legacy_line_reader = ["read", "_until"].concat();
    assert!(source.contains("PLAY_OUTPUT_DRAIN_LIMIT"));
    assert!(source.contains("PLAY_OUTPUT_DRAIN_BYTE_LIMIT"));
    assert!(source.contains("PLAY_OUTPUT_DRAIN_TIME_BUDGET"));
    assert!(!source.contains(&legacy_line_reader));
}
