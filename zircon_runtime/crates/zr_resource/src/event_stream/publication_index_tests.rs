use super::*;
use crate::{ResourceId, ResourceKind, ResourceLocator, ResourceScheme};

fn event(id: usize, kind: ResourceEventKind, revision: u64) -> ResourceEvent {
    event_with_resource_kind(id, ResourceKind::Texture, kind, revision)
}

fn event_with_resource_kind(
    id: usize,
    resource_kind: ResourceKind,
    kind: ResourceEventKind,
    revision: u64,
) -> ResourceEvent {
    ResourceEvent {
        kind,
        resource_kind,
        id: ResourceId::from_stable_label(&format!("indexed-event-{id}")),
        locator: None,
        previous_locator: None,
        revision,
    }
}

#[test]
fn resource_event_publication_uses_an_identity_index_instead_of_a_reverse_scan() {
    let source = include_str!("../event_stream.rs");
    let state_body = source
        .split("struct ResourceEventLogState")
        .nth(1)
        .and_then(|tail| tail.split("impl Default for ResourceEventLogState").next())
        .expect("resource event log state body");
    let publish_body = source
        .split("fn publish_one")
        .nth(1)
        .and_then(|tail| tail.split("fn event_identity").next())
        .expect("resource event publish body");

    assert!(state_body.contains("latest_slot_by_identity"));
    assert!(publish_body.contains("indexed_slot"));
    assert!(!publish_body.contains(".iter().rposition"));
}

#[test]
fn resource_event_publication_does_not_shift_a_contiguous_queue_for_cold_identity_updates() {
    let source = include_str!("../event_stream.rs");
    let state_body = source
        .split("struct ResourceEventLogState")
        .nth(1)
        .and_then(|tail| tail.split("impl Default for ResourceEventLogState").next())
        .expect("resource event log state body");
    let publish_body = source
        .split("fn publish_one")
        .nth(1)
        .and_then(|tail| tail.split("fn event_identity").next())
        .expect("resource event publish body");
    let take_next_body = source
        .split("fn take_next(state")
        .nth(1)
        .and_then(|tail| tail.split("fn event_identity").next())
        .expect("resource take-next body");

    assert!(state_body.contains("ResourceEventLogEntries"));
    assert!(!state_body.contains("VecDeque<LoggedResourceEvent>"));
    assert!(!state_body.contains("BTreeMap<u64, LoggedResourceEvent>"));
    assert!(source.contains("recent_slot_by_sequence"));
    assert!(publish_body.contains("entries.remove(slot)"));
    assert!(!publish_body.contains("entries.remove(index)"));
    assert!(publish_body.contains("reusable_identity_slot"));
    assert!(!publish_body.contains("remove_current_identity_mapping"));
    assert!(take_next_body.contains("first_at_or_after(expected_sequence)"));
}

#[test]
fn resource_event_publication_samples_time_after_acquiring_the_state_lock() {
    let source = include_str!("../event_stream.rs");
    let publish_body = source
        .split("pub(crate) fn publish_permitted")
        .nth(1)
        .and_then(|tail| tail.split("pub(crate) fn publish_for_test").next())
        .expect("resource event publish body");
    let lock_position = publish_body
        .find("lock_state()")
        .expect("resource event state lock");
    let timestamp_position = publish_body
        .find("Instant::now()")
        .expect("resource publication timestamp");

    assert!(
        lock_position < timestamp_position,
        "publication time must be sampled while holding the state lock so TTL order matches sequence order"
    );
}

#[test]
fn resource_event_identity_index_keeps_resource_kinds_independent() {
    let publisher = ResourceEventPublisher::default();
    publisher.publish_for_test(event_with_resource_kind(
        7,
        ResourceKind::Texture,
        ResourceEventKind::Added,
        1,
    ));
    publisher.publish_for_test(event_with_resource_kind(
        7,
        ResourceKind::Mesh,
        ResourceEventKind::Added,
        1,
    ));
    publisher.publish_for_test(event_with_resource_kind(
        7,
        ResourceKind::Texture,
        ResourceEventKind::Updated,
        2,
    ));

    let state = publisher.hub.lock_state();
    assert_eq!(state.entries.len(), 2);
    assert_eq!(state.latest_slot_by_identity.len(), 2);
    let retained_kinds = state
        .entries
        .values()
        .map(|entry| entry.event.resource_kind)
        .collect::<Vec<_>>();
    assert_eq!(retained_kinds, [ResourceKind::Mesh, ResourceKind::Texture]);
    assert_eq!(state.coalesced_count, 1);
}

#[test]
fn evicting_an_older_identity_event_keeps_the_newer_index_mapping() {
    let publisher = ResourceEventPublisher::default();
    publisher.publish_for_test(event(0, ResourceEventKind::Renamed, 1));
    publisher.publish_for_test(event(0, ResourceEventKind::Updated, 2));
    for id in 1..RESOURCE_EVENT_LOG_ENTRY_CAPACITY {
        publisher.publish_for_test(event(id, ResourceEventKind::Renamed, 1));
    }
    publisher.publish_for_test(event(0, ResourceEventKind::Updated, 3));

    let state = publisher.hub.lock_state();
    let identity = event_identity(&event(0, ResourceEventKind::Updated, 3));
    assert_eq!(state.coalesced_count, 1);
    assert_eq!(
        indexed_slot(&state, identity)
            .and_then(|slot| state.entries.get(slot))
            .map(|entry| entry.sequence),
        state.next_sequence.map(|next_sequence| next_sequence - 1),
    );
    assert_eq!(
        state
            .entries
            .values()
            .filter(|entry| event_identity(&entry.event) == identity)
            .count(),
        1
    );
}

#[test]
fn expired_event_removes_its_current_identity_index_mapping() {
    let mut state = ResourceEventLogState::default();
    let event = event(1, ResourceEventKind::Updated, 1);
    let identity = event_identity(&event);
    let approximate_bytes = approximate_event_bytes(&event);
    state.approximate_bytes = approximate_bytes;
    let slot = state.entries.insert_back(LoggedResourceEvent {
        sequence: 1,
        published_at: Instant::now() - RESOURCE_EVENT_LOG_MAX_AGE - Duration::from_millis(1),
        approximate_bytes,
        event,
    });
    state.latest_slot_by_identity.insert(identity, slot);

    evict_expired(&mut state, Instant::now());

    assert!(state.entries.is_empty());
    assert!(!state.latest_slot_by_identity.contains_key(&identity));
}

#[test]
fn byte_budget_eviction_keeps_only_retained_identity_mappings() {
    let publisher = ResourceEventPublisher::default();
    let large_label = "x".repeat(RESOURCE_EVENT_LOG_BYTE_CAPACITY / 2);
    for id in 0..3 {
        let mut next = event(id, ResourceEventKind::Renamed, 1);
        next.locator = Some(
            ResourceLocator::new(
                ResourceScheme::Memory,
                format!("indexed-event-{id}"),
                Some(large_label.clone()),
            )
            .unwrap(),
        );
        publisher.publish_for_test(next);
    }

    let state = publisher.hub.lock_state();
    assert_eq!(state.latest_slot_by_identity.len(), state.entries.len());
    for entry in state.entries.values() {
        assert_eq!(
            indexed_slot(&state, event_identity(&entry.event))
                .and_then(|slot| state.entries.get(slot))
                .map(|indexed| indexed.sequence),
            Some(entry.sequence),
        );
    }
}

#[test]
fn indexed_publication_matches_the_scan_semantics_for_mixed_events() {
    let publisher = ResourceEventPublisher::default();
    let mut oracle = Vec::<ResourceEvent>::new();
    let mut oracle_coalesced = 0_u64;
    let mut oracle_dropped = 0_u64;

    for index in 0..10_000_usize {
        let resource_kind = if index % 5 == 0 {
            ResourceKind::Mesh
        } else {
            ResourceKind::Texture
        };
        let kind = match index % 11 {
            0 => ResourceEventKind::Removed,
            1 => ResourceEventKind::Renamed,
            2 => ResourceEventKind::ReloadFailed,
            3 | 4 => ResourceEventKind::Added,
            _ => ResourceEventKind::Updated,
        };
        let next = event_with_resource_kind(
            index.wrapping_mul(37) % 137,
            resource_kind,
            kind,
            index as u64,
        );

        if is_coalescable(next.kind) {
            if let Some(previous_index) = oracle.iter().rposition(|previous| {
                previous.id == next.id && previous.resource_kind == next.resource_kind
            }) {
                if is_coalescable(oracle[previous_index].kind) {
                    oracle.remove(previous_index);
                    oracle_coalesced = oracle_coalesced.saturating_add(1);
                }
            }
        }
        oracle.push(next.clone());
        while oracle.len() > RESOURCE_EVENT_LOG_ENTRY_CAPACITY {
            oracle.remove(0);
            oracle_dropped = oracle_dropped.saturating_add(1);
        }
        publisher.publish_for_test(next);
    }

    let state = publisher.hub.lock_state();
    assert_eq!(state.coalesced_count, oracle_coalesced);
    assert_eq!(state.dropped_count, oracle_dropped);
    assert_eq!(
        state
            .entries
            .values()
            .map(|entry| &entry.event)
            .collect::<Vec<_>>(),
        oracle.iter().collect::<Vec<_>>()
    );
}

#[test]
fn recent_sequence_collision_falls_back_to_linked_order_without_losing_the_new_mapping() {
    let mut entries = ResourceEventLogEntries::default();
    let old_sequence = 1_u64;
    let new_sequence = old_sequence + RESOURCE_EVENT_LOG_ENTRY_CAPACITY as u64;
    let now = Instant::now();
    let old_event = event(1, ResourceEventKind::Renamed, old_sequence);
    let new_event = event(2, ResourceEventKind::Renamed, new_sequence);
    let old_slot = entries.insert_back(LoggedResourceEvent {
        sequence: old_sequence,
        published_at: now,
        approximate_bytes: approximate_event_bytes(&old_event),
        event: old_event,
    });
    let new_slot = entries.insert_back(LoggedResourceEvent {
        sequence: new_sequence,
        published_at: now,
        approximate_bytes: approximate_event_bytes(&new_event),
        event: new_event,
    });

    assert_eq!(entries.exact_slot(old_sequence), None);
    assert_eq!(entries.exact_slot(new_sequence), Some(new_slot));
    assert_eq!(
        entries
            .first_at_or_after(old_sequence)
            .map(|entry| entry.sequence),
        Some(old_sequence)
    );
    assert_eq!(entries.count_at_or_after(old_sequence), 2);

    entries.remove(old_slot).unwrap();

    assert_eq!(entries.exact_slot(new_sequence), Some(new_slot));
    assert_eq!(
        entries
            .first_at_or_after(old_sequence)
            .map(|entry| entry.sequence),
        Some(new_sequence)
    );
}

#[test]
fn stalled_resource_event_consumer_observes_a_bounded_gap() {
    let publisher = ResourceEventPublisher::default();
    let receiver = publisher.subscribe();
    for id in 0..(RESOURCE_EVENT_LOG_ENTRY_CAPACITY + 32) {
        publisher.publish_for_test(event(id, ResourceEventKind::Renamed, 1));
    }

    assert!(matches!(
        receiver.try_recv(),
        Err(ResourceEventTryRecvError::Lagged(_))
    ));
    let diagnostics = publisher.diagnostics();
    assert_eq!(diagnostics.depth, RESOURCE_EVENT_LOG_ENTRY_CAPACITY);
    assert_eq!(diagnostics.dropped_count, 32);
    assert_eq!(diagnostics.lagged_read_count, 1);
}

#[test]
fn resource_event_stream_scale_matrix_stays_bounded_at_one_thousand_and_one_hundred_thousand() {
    for event_count in [1usize, 1_000, 100_000] {
        let publisher = ResourceEventPublisher::default();
        let receiver = publisher.subscribe();
        for id in 0..event_count {
            publisher.publish_for_test(event(id, ResourceEventKind::Renamed, 1));
        }

        let retained = event_count.min(RESOURCE_EVENT_LOG_ENTRY_CAPACITY);
        let diagnostics = publisher.diagnostics();
        assert_eq!(diagnostics.depth, retained);
        assert!(diagnostics.approximate_bytes <= RESOURCE_EVENT_LOG_BYTE_CAPACITY);
        assert_eq!(diagnostics.dropped_count as usize, event_count - retained);

        if event_count > RESOURCE_EVENT_LOG_ENTRY_CAPACITY {
            assert!(matches!(
                receiver.try_recv(),
                Err(ResourceEventTryRecvError::Lagged(_))
            ));
        }
        let mut consumed = 0usize;
        while receiver.try_recv().is_ok() {
            consumed = consumed.saturating_add(1);
        }
        assert_eq!(consumed, retained);
    }
}

#[test]
fn resource_event_log_coalesces_updates_but_preserves_lifecycle_edges() {
    let publisher = ResourceEventPublisher::default();
    let receiver = publisher.subscribe();
    publisher.publish_for_test(event(1, ResourceEventKind::Added, 1));
    publisher.publish_for_test(event(1, ResourceEventKind::Updated, 2));
    publisher.publish_for_test(event(1, ResourceEventKind::Renamed, 2));
    publisher.publish_for_test(event(1, ResourceEventKind::Updated, 3));
    publisher.publish_for_test(event(1, ResourceEventKind::Removed, 3));

    assert!(matches!(
        receiver.try_recv(),
        Err(ResourceEventTryRecvError::Lagged(_))
    ));
    assert_eq!(
        receiver.try_recv().unwrap().kind,
        ResourceEventKind::Updated
    );
    assert_eq!(
        receiver.try_recv().unwrap().kind,
        ResourceEventKind::Renamed
    );
    assert_eq!(
        receiver.try_recv().unwrap().kind,
        ResourceEventKind::Updated
    );
    assert_eq!(
        receiver.try_recv().unwrap().kind,
        ResourceEventKind::Removed
    );
    assert_eq!(publisher.diagnostics().coalesced_count, 1);
}

#[test]
fn resource_event_log_enforces_the_byte_budget_independently_from_entry_count() {
    let publisher = ResourceEventPublisher::default();
    let receiver = publisher.subscribe();
    let large_label = "x".repeat(RESOURCE_EVENT_LOG_BYTE_CAPACITY / 2);
    for id in 0..3 {
        let mut resource_event = event(id, ResourceEventKind::Renamed, 1);
        resource_event.locator = Some(
            ResourceLocator::new(
                ResourceScheme::Memory,
                format!("event-{id}"),
                Some(large_label.clone()),
            )
            .unwrap(),
        );
        publisher.publish_for_test(resource_event);
    }

    assert!(matches!(
        receiver.try_recv(),
        Err(ResourceEventTryRecvError::Lagged(_))
    ));
    let diagnostics = publisher.diagnostics();
    assert!(diagnostics.depth < 3);
    assert!(diagnostics.approximate_bytes <= RESOURCE_EVENT_LOG_BYTE_CAPACITY);
    assert!(diagnostics.dropped_count > 0);
}

#[test]
fn resource_event_receiver_disconnects_after_the_last_publisher_is_dropped() {
    let publisher = ResourceEventPublisher::default();
    let receiver = publisher.subscribe();
    drop(publisher);

    assert_eq!(
        receiver.try_recv(),
        Err(ResourceEventTryRecvError::Disconnected)
    );
}

#[test]
fn blocking_resource_event_receiver_wakes_when_the_last_publisher_is_dropped() {
    let publisher = ResourceEventPublisher::default();
    let receiver = publisher.subscribe();
    let (started_tx, started_rx) = std::sync::mpsc::sync_channel(0);
    let (result_tx, result_rx) = std::sync::mpsc::sync_channel(1);
    let receiver_thread = std::thread::spawn(move || {
        started_tx.send(()).unwrap();
        result_tx.send(receiver.recv()).unwrap();
    });

    started_rx.recv().unwrap();
    drop(publisher);

    assert_eq!(
        result_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("the final publisher drop must wake a blocking receiver"),
        Err(ResourceEventRecvError::Disconnected)
    );
    receiver_thread.join().unwrap();
}

#[test]
fn ten_thousand_subscribers_share_one_logged_event() {
    let publisher = ResourceEventPublisher::default();
    let receivers = (0..10_000)
        .map(|_| publisher.subscribe())
        .collect::<Vec<_>>();

    publisher.publish_for_test(event(7, ResourceEventKind::Renamed, 3));

    assert_eq!(publisher.diagnostics().depth, 1);
    for receiver in receivers {
        let received = receiver.try_recv().unwrap();
        assert_eq!(received.revision, 3);
        assert_eq!(received.kind, ResourceEventKind::Renamed);
    }
    assert_eq!(publisher.diagnostics().depth, 1);
}

#[test]
fn resource_event_log_expires_old_entries_by_ttl() {
    let mut state = ResourceEventLogState::default();
    let event = event(1, ResourceEventKind::Renamed, 1);
    let approximate_bytes = approximate_event_bytes(&event);
    state.approximate_bytes = approximate_bytes;
    state.entries.insert_back(LoggedResourceEvent {
        sequence: 1,
        published_at: Instant::now() - RESOURCE_EVENT_LOG_MAX_AGE - Duration::from_millis(1),
        approximate_bytes,
        event,
    });

    evict_expired(&mut state, Instant::now());

    assert!(state.entries.is_empty());
    assert_eq!(state.approximate_bytes, 0);
    assert_eq!(state.dropped_count, 1);
}

#[test]
fn resource_event_log_reports_a_gap_after_eviction_empties_the_log() {
    let mut state = ResourceEventLogState {
        next_sequence: Some(9),
        dropped_count: 8,
        ..Default::default()
    };
    let mut cursor = Some(1);

    let read = take_next(&mut state, &mut cursor);

    assert!(matches!(
        read,
        EventRead::Lagged(ResourceEventGap {
            expected_sequence: 1,
            oldest_available_sequence: Some(9),
        })
    ));
    assert_eq!(cursor, Some(9));
    assert_eq!(state.lagged_read_count, 1);
}

#[test]
fn optimization_wave_20260824_runtime64_resource_event_cursor_preserves_sparse_sequence_gaps() {
    let now = Instant::now();
    let mut state = ResourceEventLogState::default();
    for sequence in [2_u64, 4, 9] {
        let event = event(sequence as usize, ResourceEventKind::Renamed, sequence);
        state.entries.insert_back(LoggedResourceEvent {
            sequence,
            published_at: now,
            approximate_bytes: approximate_event_bytes(&event),
            event,
        });
    }
    state.next_sequence = Some(10);

    let mut cursor = Some(3);
    assert!(matches!(
        take_next(&mut state, &mut cursor),
        EventRead::Lagged(ResourceEventGap {
            expected_sequence: 3,
            oldest_available_sequence: Some(4),
        })
    ));
    assert_eq!(cursor, Some(4));
    assert!(matches!(
        take_next(&mut state, &mut cursor),
        EventRead::Event(_)
    ));
    assert_eq!(cursor, Some(5));
    assert!(matches!(
        take_next(&mut state, &mut cursor),
        EventRead::Lagged(ResourceEventGap {
            expected_sequence: 5,
            oldest_available_sequence: Some(9),
        })
    ));
    assert_eq!(cursor, Some(9));
}

#[test]
fn optimization_wave_20260824_runtime64_resource_event_cursor_uses_indexed_lookup() {
    let source = include_str!("../event_stream.rs");
    let len_body = source
        .split("pub fn len(&self) -> usize")
        .nth(1)
        .and_then(|tail| tail.split("pub fn is_empty").next())
        .expect("resource receiver len body");
    let take_next_body = source
        .split("fn take_next(state")
        .nth(1)
        .and_then(|tail| tail.split("fn is_coalescable").next())
        .expect("resource take_next body");

    assert!(len_body.contains("count_at_or_after(cursor)"));
    assert!(!len_body.contains(".filter("));
    assert!(take_next_body.contains("first_at_or_after(expected_sequence)"));
    assert!(!take_next_body.contains(".find("));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_wave_20260824_runtime64_resource_event_cursor_lookup_evidence() {
    const ENTRIES: usize = RESOURCE_EVENT_LOG_ENTRY_CAPACITY;
    const QUERIES: usize = 250_000;
    const MAX_ELAPSED_NS: u128 = 500_000_000;

    let now = Instant::now();
    let mut entries = ResourceEventLogEntries::default();
    for sequence in 1..=ENTRIES as u64 {
        let event = event(sequence as usize, ResourceEventKind::Renamed, sequence);
        entries.insert_back(LoggedResourceEvent {
            sequence,
            published_at: now,
            approximate_bytes: approximate_event_bytes(&event),
            event,
        });
    }
    let cursor = ENTRIES as u64;
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..QUERIES {
        checksum ^= entries
            .first_at_or_after(cursor)
            .map(|entry| entry.sequence as usize)
            .unwrap_or(0);
    }
    std::hint::black_box(checksum);
    let elapsed_ns = started.elapsed().as_nanos();

    assert!(
        elapsed_ns <= MAX_ELAPSED_NS,
        "partitioned resource cursor lookup took {elapsed_ns}ns; limit is {MAX_ELAPSED_NS}ns"
    );

    let legacy_comparisons = ENTRIES.saturating_mul(QUERIES);
    let comparisons_per_query_bound = usize::BITS as usize - ENTRIES.leading_zeros() as usize;
    let optimized_comparison_bound = comparisons_per_query_bound.saturating_mul(QUERIES);
    let comparison_reduction_bps = legacy_comparisons
        .saturating_sub(optimized_comparison_bound)
        .saturating_mul(10_000)
        / legacy_comparisons;
    println!(
        "RESOURCE_EVENT_CURSOR_BENCH_V2 entries={ENTRIES} queries={QUERIES} legacy_comparisons={legacy_comparisons} optimized_comparison_bound={optimized_comparison_bound} comparison_reduction_bps={comparison_reduction_bps} elapsed_ns={elapsed_ns} max_elapsed_ns={MAX_ELAPSED_NS}"
    );
}
