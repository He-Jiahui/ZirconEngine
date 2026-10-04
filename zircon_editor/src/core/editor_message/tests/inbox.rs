use std::hint::black_box;
use std::time::Instant;

use crate::core::editor_message::{
    EditorMessage, EditorMessagePayload, EditorMessageProtocol, EditorTopic, FocusMessage,
    SelectionDomain,
};
use crate::core::play::PlayInstanceId;

use super::{EditorMessageDelivery, EditorMessageInbox, EditorMessageInboxLimits};

const PLANNER_ITERATIONS: usize = 50_000;
const SAMPLE_PAIRS: usize = 17;

fn latest_delivery(sequence: u64, revision: u64) -> EditorMessageDelivery {
    latest_delivery_in_domain(sequence, revision, SelectionDomain::edit_scene())
}

fn latest_delivery_in_domain(
    sequence: u64,
    revision: u64,
    domain: SelectionDomain,
) -> EditorMessageDelivery {
    EditorMessageDelivery::with_sequence(
        EditorMessageProtocol::Publish,
        EditorTopic::parse("editor.inbox.order").expect("valid inbox test topic"),
        EditorMessage::new(EditorMessagePayload::Focus(
            FocusMessage::SelectionChanged { domain, revision },
        )),
        sequence,
    )
}

fn bounded_delivery(sequence: u64, revision: u64) -> EditorMessageDelivery {
    EditorMessageDelivery::with_sequence(
        EditorMessageProtocol::Publish,
        EditorTopic::parse("editor.inbox.order").expect("valid inbox test topic"),
        EditorMessage::custom(
            crate::core::editor_message::EditorMessageSchemaId::editor("inbox.order.v1").unwrap(),
            serde_json::json!({ "revision": revision }),
        ),
        sequence,
    )
}

fn legacy_bounded_evictions_for(
    inbox: &EditorMessageInbox,
    incoming_sequence: u64,
    incoming_bytes: usize,
) -> Option<Vec<u64>> {
    let mut retained_bytes = inbox.retained_bytes;
    let mut bounded_depth = inbox.bounded_depth;
    let mut evictions = Vec::new();
    for sequence in &inbox.bounded_order {
        if bounded_depth < inbox.limits.bounded_capacity
            && retained_bytes
                .checked_add(incoming_bytes)
                .is_some_and(|bytes| bytes <= inbox.limits.retained_bytes_capacity)
        {
            return Some(evictions);
        }
        if *sequence >= incoming_sequence {
            return None;
        }
        let delivery = inbox.deliveries.get(sequence)?;
        retained_bytes = retained_bytes.checked_sub(delivery.retained_bytes())?;
        bounded_depth = bounded_depth.checked_sub(1)?;
        evictions.push(*sequence);
    }
    (bounded_depth < inbox.limits.bounded_capacity
        && retained_bytes
            .checked_add(incoming_bytes)
            .is_some_and(|bytes| bytes <= inbox.limits.retained_bytes_capacity))
    .then_some(evictions)
}

fn full_bounded_inbox(capacity: usize) -> EditorMessageInbox {
    let mut inbox = EditorMessageInbox::new(EditorMessageInboxLimits::new(1, capacity, 1));
    for sequence in 0..u64::try_from(capacity).unwrap() {
        inbox.enqueue(bounded_delivery(sequence, sequence));
    }
    inbox
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

#[test]
fn out_of_order_latest_delivery_keeps_the_highest_sequence() {
    let mut inbox = EditorMessageInbox::new(EditorMessageInboxLimits::default());
    inbox.enqueue(latest_delivery(2, 2));
    inbox.enqueue(latest_delivery(1, 1));

    let deliveries = inbox.deliveries();
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].sequence(), 2);
    assert_eq!(
        deliveries[0].message(),
        &EditorMessage::new(EditorMessagePayload::Focus(
            FocusMessage::SelectionChanged {
                domain: SelectionDomain::edit_scene(),
                revision: 2,
            },
        ))
    );
}

#[test]
fn latest_selection_delivery_is_partitioned_by_play_instance() {
    let mut inbox = EditorMessageInbox::new(EditorMessageInboxLimits::default());
    let first = PlayInstanceId::for_test(1);
    let second = PlayInstanceId::for_test(2);

    inbox.enqueue(latest_delivery_in_domain(
        1,
        1,
        SelectionDomain::edit_scene(),
    ));
    inbox.enqueue(latest_delivery_in_domain(
        2,
        1,
        SelectionDomain::play_scene(first),
    ));
    inbox.enqueue(latest_delivery_in_domain(
        3,
        1,
        SelectionDomain::play_scene(second),
    ));
    inbox.enqueue(latest_delivery_in_domain(
        4,
        2,
        SelectionDomain::play_scene(first),
    ));

    assert_eq!(
        inbox
            .deliveries()
            .iter()
            .map(EditorMessageDelivery::sequence)
            .collect::<Vec<_>>(),
        [1, 3, 4]
    );
    assert_eq!(inbox.stats(4).coalesced(), 1);
}

#[test]
fn out_of_order_bounded_delivery_evicts_the_lowest_sequence() {
    let mut inbox = EditorMessageInbox::new(EditorMessageInboxLimits::new(1, 1, 1));
    inbox.enqueue(bounded_delivery(2, 2));
    inbox.enqueue(bounded_delivery(1, 1));

    let deliveries = inbox.deliveries();
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].sequence(), 2);
}

#[test]
fn optimization_batch_20260826d_editor48_inbox_eviction_plans_store_only_counts() {
    let source = include_str!("../inbox.rs")
        .split_once("mod tests {")
        .unwrap()
        .0;
    let latest_eviction = source
        .split("fn remove_oldest_latest(&mut self)")
        .nth(1)
        .and_then(|body| body.split("fn remove_oldest_bounded(&mut self)").next())
        .expect("indexed latest eviction implementation");
    let bounded_eviction = source
        .split("fn remove_oldest_bounded(&mut self)")
        .nth(1)
        .and_then(|body| body.split("fn can_add_bytes(&self").next())
        .expect("indexed bounded eviction implementation");

    assert!(source.contains("fn bounded_eviction_count_for("));
    assert!(source.contains("fn latest_eviction_count_for("));
    assert!(source.contains("fn latest_replacement_eviction_count_for("));
    assert!(bounded_eviction.contains(".bounded_order"));
    assert!(bounded_eviction.contains(".pop_first()"));
    assert!(latest_eviction.contains(".latest_order"));
    assert!(latest_eviction.contains(".pop_first()"));
    assert!(!source.contains("let mut evictions = Vec::new()"));
}

#[test]
fn optimization_batch_20260826d_editor48_inbox_rolling_eviction_preserves_order_and_stats() {
    let mut inbox = EditorMessageInbox::new(EditorMessageInboxLimits::new(1, 8, 1));
    for sequence in 0..64 {
        inbox.enqueue(bounded_delivery(sequence, sequence));
    }

    assert_eq!(
        inbox
            .deliveries()
            .iter()
            .map(EditorMessageDelivery::sequence)
            .collect::<Vec<_>>(),
        (56..64).collect::<Vec<_>>()
    );
    let stats = inbox.stats(64);
    assert_eq!(stats.depth(), 8);
    assert_eq!(stats.bounded_depth(), 8);
    assert_eq!(stats.dropped(), 56);
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260826d_editor48_inbox_eviction_plan_performance_evidence() {
    let inbox = full_bounded_inbox(1_024);
    let incoming = bounded_delivery(1_024, 1_024);
    let incoming_bytes = incoming.retained_bytes();

    assert_eq!(
        legacy_bounded_evictions_for(&inbox, incoming.sequence(), incoming_bytes)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        inbox
            .bounded_eviction_count_for(incoming.sequence(), incoming_bytes)
            .unwrap(),
        1
    );

    let measure_legacy = || {
        elapsed_micros(|| {
            for _ in 0..PLANNER_ITERATIONS {
                black_box(
                    legacy_bounded_evictions_for(
                        black_box(&inbox),
                        incoming.sequence(),
                        incoming_bytes,
                    )
                    .unwrap(),
                );
            }
        })
    };
    let measure_optimized = || {
        elapsed_micros(|| {
            for _ in 0..PLANNER_ITERATIONS {
                black_box(
                    inbox
                        .bounded_eviction_count_for(incoming.sequence(), incoming_bytes)
                        .unwrap(),
                );
            }
        })
    };
    for _ in 0..3 {
        black_box(measure_legacy());
        black_box(measure_optimized());
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
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
        "EDITOR48_INBOX_EVICTION_COUNT_PLAN_BENCH_V1 sample_pairs={} planner_iterations={} retained_deliveries={} evictions_per_plan=1 legacy_temporary_plan_allocations={} optimized_temporary_plan_allocations=0 legacy_copied_sequences={} optimized_copied_sequences=0 legacy_p95_us={} optimized_p95_us={} legacy_samples_us={:?} optimized_samples_us={:?}",
        SAMPLE_PAIRS,
        PLANNER_ITERATIONS,
        inbox.deliveries.len(),
        PLANNER_ITERATIONS,
        PLANNER_ITERATIONS,
        legacy_p95,
        optimized_p95,
        legacy_samples,
        optimized_samples,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "count-only eviction planning p95 must be at least 30% below allocating sequence plans: legacy={legacy_p95}us optimized={optimized_p95}us"
    );
}
