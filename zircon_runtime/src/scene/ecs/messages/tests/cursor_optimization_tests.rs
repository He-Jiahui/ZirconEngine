use std::hint::black_box;
use std::time::Instant;

use super::{Message, MessageCursor, MessageReadIter, Messages};
use crate::scene::ecs::messages::queue::MessageRetention;

const SAMPLE_PAIRS: usize = 31;
const BENCH_MESSAGES: usize = 65_536;
const READS_PER_SAMPLE: usize = 32;

struct SampleMessage(u64);

impl Message for SampleMessage {}

#[test]
fn runtime60_message_cursor_range_read_preserves_partial_consumption() {
    let mut messages = Messages::<SampleMessage>::default();
    messages.set_retention(MessageRetention::new(8, usize::MAX, u64::MAX));
    for value in 0..4 {
        messages.write(SampleMessage(value));
    }
    let mut cursor = MessageCursor::<SampleMessage>::default();
    let mut first_read = cursor.read(Some(&messages));
    assert_eq!(
        first_read.next().map(|(id, message)| (id.id(), message.0)),
        Some((0, 0))
    );
    drop(first_read);

    assert_eq!(cursor.unread_count(Some(&messages)), 3);
    let remaining = cursor
        .read(Some(&messages))
        .map(|(id, message)| (id.id(), message.0))
        .collect::<Vec<_>>();
    assert_eq!(remaining, vec![(1, 1), (2, 2), (3, 3)]);
    assert_eq!(cursor.unread_count(Some(&messages)), 0);
    assert_eq!(cursor.read(Some(&messages)).count(), 0);
    assert_eq!(cursor.dropped_count(), 0);
}

#[test]
fn runtime60_message_cursor_range_read_handles_wrapped_queue_with_nonzero_start() {
    let mut messages = Messages::<SampleMessage>::default();
    messages.set_retention(MessageRetention::new(3, usize::MAX, u64::MAX));
    let last_written = (0..128_usize)
        .find(|&value| {
            messages.write(SampleMessage(value as u64));
            !messages.messages.as_slices().1.is_empty()
        })
        .expect("retained VecDeque wraps within the bounded write sequence");
    assert_eq!(messages.len(), 3);

    let mut cursor = MessageCursor::<SampleMessage>::default();
    let mut first_read = cursor.read(Some(&messages));
    assert_eq!(
        first_read.next().map(|(id, message)| (id.id(), message.0)),
        Some((last_written - 2, (last_written - 2) as u64))
    );
    drop(first_read);
    assert_eq!(messages.read_window_start(cursor.next_id), (1, 0));
    assert_eq!(cursor.unread_count(Some(&messages)), 2);

    let remaining = cursor
        .read(Some(&messages))
        .map(|(id, message)| (id.id(), message.0))
        .collect::<Vec<_>>();
    assert_eq!(
        remaining,
        vec![
            (last_written - 1, (last_written - 1) as u64),
            (last_written, last_written as u64)
        ]
    );
    assert_eq!(cursor.unread_count(Some(&messages)), 0);
    assert_eq!(cursor.read(Some(&messages)).count(), 0);
    assert_eq!(cursor.dropped_count(), (last_written - 2) as u64);
}

#[test]
fn runtime60_message_cursor_range_read_reports_retention_gap_once() {
    let mut messages = Messages::<SampleMessage>::default();
    messages.set_retention(MessageRetention::new(3, usize::MAX, u64::MAX));
    for value in 0..6 {
        messages.write(SampleMessage(value));
    }
    let mut cursor = MessageCursor::<SampleMessage>::default();
    let retained = cursor
        .read(Some(&messages))
        .map(|(id, message)| (id.id(), message.0))
        .collect::<Vec<_>>();
    assert_eq!(retained, vec![(3, 3), (4, 4), (5, 5)]);
    assert_eq!(cursor.dropped_count(), 3);
    assert_eq!(cursor.read(Some(&messages)).count(), 0);
    assert_eq!(cursor.dropped_count(), 3);
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime60_message_cursor_range_read_bench() {
    let mut messages = Messages::<SampleMessage>::default();
    messages.set_retention(MessageRetention::new(BENCH_MESSAGES, usize::MAX, u64::MAX));
    for value in 0..BENCH_MESSAGES as u64 {
        messages.write(SampleMessage(value));
    }
    assert_eq!(messages.len(), BENCH_MESSAGES);
    let messages = black_box(&messages);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_empty_reads(messages, true));
            optimized_samples.push(measure_empty_reads(messages, false));
        } else {
            optimized_samples.push(measure_empty_reads(messages, false));
            legacy_samples.push(measure_empty_reads(messages, true));
        }
    }
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    println!(
        "RUNTIME60_MESSAGE_CURSOR_RANGE_READ_BENCH_V1 sample_pairs={SAMPLE_PAIRS} retained_entries={BENCH_MESSAGES} empty_reads_per_sample={READS_PER_SAMPLE} target=optimized_p95_at_most_half_legacy legacy_p50_ns={} optimized_p50_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_p99_ns={} optimized_p99_ns={} legacy_raw_ns={legacy_samples:?} optimized_raw_ns={optimized_samples:?}",
        nearest_rank(&legacy_samples, 50),
        nearest_rank(&optimized_samples, 50),
        nearest_rank(&legacy_samples, 99),
        nearest_rank(&optimized_samples, 99),
    );
    assert!(optimized_p95.saturating_mul(2) <= legacy_p95);
}

fn measure_empty_reads(messages: &Messages<SampleMessage>, legacy: bool) -> u64 {
    let mut cursor = MessageCursor::<SampleMessage>::default();
    cursor.clear(Some(messages));
    assert_eq!(cursor.unread_count(Some(messages)), 0);
    let started = Instant::now();
    for _ in 0..READS_PER_SAMPLE {
        let iter = if legacy {
            legacy_read(&mut cursor, messages)
        } else {
            cursor.read(Some(messages))
        };
        assert!(black_box(iter).next().is_none());
    }
    let elapsed_ns = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    assert_eq!(cursor.unread_count(Some(messages)), 0);
    assert_eq!(cursor.dropped_count(), 0);
    black_box(cursor);
    elapsed_ns
}

// Preserve the pre-change read path as a Release comparator.
fn legacy_read<'a>(
    cursor: &'a mut MessageCursor<SampleMessage>,
    messages: &'a Messages<SampleMessage>,
) -> MessageReadIter<'a, SampleMessage> {
    let (start, dropped) = if cursor.generation == messages.generation() {
        messages.read_window_start(cursor.next_id)
    } else {
        (0, 0)
    };
    cursor.dropped_count = cursor.dropped_count.saturating_add(dropped as u64);
    cursor.next_id = messages
        .messages
        .get(start)
        .map(|message| message.id.id())
        .unwrap_or_else(|| messages.next_id());
    cursor.generation = messages.generation();
    let mut iter = messages.messages.iter();
    for _ in 0..start {
        let _ = iter.next();
    }
    MessageReadIter::tracked(iter, cursor)
}

fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
