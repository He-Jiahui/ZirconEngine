use std::hint::black_box;
use std::mem::size_of;
use std::time::Instant;

use super::{bounded_batch_reserve_additional, Message, MessageId, MessageRetention, Messages};

const SAMPLE_PAIRS: usize = 31;
const BENCH_MESSAGES: usize = 65_536;
const BENCH_RETAINED: usize = 1_024;

#[derive(Debug, PartialEq, Eq)]
struct SampleMessage(u64);

impl Message for SampleMessage {}

#[test]
fn runtime60_message_batch_reserve_preserves_ids_retained_tail_and_drop_metrics() {
    let retention = MessageRetention::new(8, usize::MAX, u64::MAX);
    let mut optimized = Messages::<SampleMessage>::default();
    optimized.set_retention(retention);
    let mut legacy = Messages::<SampleMessage>::default();
    legacy.set_retention(retention);

    let actual_ids = optimized.write_batch((0..4_096).map(SampleMessage));
    let expected_ids = legacy_write_batch(&mut legacy, (0..4_096).map(SampleMessage));
    let actual_id_numbers = actual_ids.iter().map(|id| id.id()).collect::<Vec<_>>();
    let expected_id_numbers = expected_ids.iter().map(|id| id.id()).collect::<Vec<_>>();
    assert_eq!(actual_id_numbers, expected_id_numbers);
    assert_eq!(actual_id_numbers, (0..4_096).collect::<Vec<_>>());

    let actual_tail = optimized
        .messages
        .iter()
        .map(|entry| (entry.id.id(), entry.message.0))
        .collect::<Vec<_>>();
    let expected_tail = legacy
        .messages
        .iter()
        .map(|entry| (entry.id.id(), entry.message.0))
        .collect::<Vec<_>>();
    assert_eq!(actual_tail, expected_tail);
    assert_eq!(
        actual_tail,
        (4_088..4_096).map(|id| (id, id as u64)).collect::<Vec<_>>()
    );

    let metrics = optimized.retention_metrics();
    assert_eq!(metrics, legacy.retention_metrics());
    assert_eq!(metrics.retained_entries, 8);
    assert_eq!(metrics.budget_dropped_entries, 4_088);
    assert_eq!(
        metrics.budget_dropped_bytes,
        4_088 * size_of::<SampleMessage>() as u64
    );
    assert!(optimized.messages.capacity().saturating_mul(16) <= legacy.messages.capacity());
}

#[test]
fn runtime60_message_batch_reserve_handles_zero_unbounded_and_overfull_limits() {
    assert_eq!(bounded_batch_reserve_additional(0, 0, 32), 1);
    assert_eq!(bounded_batch_reserve_additional(4, 2, 32), 1);
    assert_eq!(bounded_batch_reserve_additional(4, usize::MAX, 32), 32);
    assert_eq!(
        bounded_batch_reserve_additional(usize::MAX, usize::MAX, 1),
        0
    );

    let mut zero = Messages::<SampleMessage>::default();
    zero.set_retention(MessageRetention::new(0, usize::MAX, u64::MAX));
    let zero_ids = zero.write_batch((0..32).map(SampleMessage));
    let mut zero_legacy = Messages::<SampleMessage>::default();
    zero_legacy.set_retention(MessageRetention::new(0, usize::MAX, u64::MAX));
    let legacy_zero_ids = legacy_write_batch(&mut zero_legacy, (0..32).map(SampleMessage));
    assert_eq!(
        zero_ids.iter().map(|id| id.id()).collect::<Vec<_>>(),
        legacy_zero_ids.iter().map(|id| id.id()).collect::<Vec<_>>()
    );
    assert_eq!(
        zero_ids.iter().map(|id| id.id()).collect::<Vec<_>>(),
        (0..32).collect::<Vec<_>>()
    );
    assert_eq!(zero.len(), 0);
    assert_eq!(zero.retention_metrics().budget_dropped_entries, 32);
    assert_eq!(zero.retention_metrics(), zero_legacy.retention_metrics());
    assert!(zero.messages.capacity().saturating_mul(4) <= zero_legacy.messages.capacity());

    let mut unbounded = Messages::<SampleMessage>::default();
    unbounded.set_retention(MessageRetention::new(usize::MAX, usize::MAX, u64::MAX));
    let ids = unbounded.write_batch((0..32).map(SampleMessage));
    assert_eq!(ids.len(), 32);
    assert_eq!(unbounded.len(), 32);
    assert_eq!(unbounded.retention_metrics().budget_dropped_entries, 0);
    assert!(unbounded.messages.capacity() >= 32);
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime60_message_queue_bounded_batch_reserve_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_batch(true));
            optimized_samples.push(measure_batch(false));
        } else {
            optimized_samples.push(measure_batch(false));
            legacy_samples.push(measure_batch(true));
        }
    }
    let legacy_raw = legacy_samples.iter().map(|(ns, _)| *ns).collect::<Vec<_>>();
    let optimized_raw = optimized_samples
        .iter()
        .map(|(ns, _)| *ns)
        .collect::<Vec<_>>();
    println!(
        "RUNTIME60_MESSAGE_QUEUE_BOUNDED_BATCH_RESERVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} messages_per_sample={BENCH_MESSAGES} retained_entries={BENCH_RETAINED} legacy_capacity={} optimized_capacity={} capacity_target=optimized_at_most_1_of_16 legacy_p50_ns={} optimized_p50_ns={} legacy_p95_ns={} optimized_p95_ns={} legacy_p99_ns={} optimized_p99_ns={} legacy_raw_ns={legacy_raw:?} optimized_raw_ns={optimized_raw:?}",
        legacy_samples[0].1,
        optimized_samples[0].1,
        nearest_rank(&legacy_raw, 50),
        nearest_rank(&optimized_raw, 50),
        nearest_rank(&legacy_raw, 95),
        nearest_rank(&optimized_raw, 95),
        nearest_rank(&legacy_raw, 99),
        nearest_rank(&optimized_raw, 99),
    );
    for ((_, legacy_capacity), (_, optimized_capacity)) in
        legacy_samples.iter().zip(optimized_samples.iter())
    {
        assert!(optimized_capacity.saturating_mul(16) <= *legacy_capacity);
    }
}

fn measure_batch(legacy: bool) -> (u64, usize) {
    let mut queue = Messages::<SampleMessage>::default();
    queue.set_retention(MessageRetention::new(BENCH_RETAINED, usize::MAX, u64::MAX));
    let started = Instant::now();
    let ids = if legacy {
        legacy_write_batch(&mut queue, (0..BENCH_MESSAGES as u64).map(SampleMessage))
    } else {
        queue.write_batch((0..BENCH_MESSAGES as u64).map(SampleMessage))
    };
    let elapsed_ns = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    assert_eq!(ids.len(), BENCH_MESSAGES);
    assert_eq!(queue.len(), BENCH_RETAINED);
    black_box((&ids, &queue.messages));
    (elapsed_ns, queue.messages.capacity())
}

fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn legacy_write_batch(
    queue: &mut Messages<SampleMessage>,
    messages: impl IntoIterator<Item = SampleMessage>,
) -> Vec<MessageId<SampleMessage>> {
    let messages = messages.into_iter();
    let (lower_bound, _) = messages.size_hint();
    queue.messages.reserve(lower_bound);
    let mut ids = Vec::with_capacity(lower_bound);
    for message in messages {
        ids.push(queue.write_at_frame(message, 0));
    }
    ids
}
