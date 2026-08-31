use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::editor_event::{
    EditorEventJournal, EditorEventJournalStore, EditorEventListenerRegistry, EditorEventRecord,
    EditorEventRetentionPolicy, SharedEditorEventRecord,
};
use crate::core::editor_message::SharedEditorMessageBus;

use super::EditorEventStamp;
use super::state::EditorEventSequenceState;

/// Journal, listener, sequence, and revision owner for editor events.
pub struct EditorEventService {
    sequence_state: Mutex<EditorEventSequenceState>,
    journal: Mutex<EditorEventJournalStore>,
    listeners: Mutex<EditorEventListenerRegistry>,
    bus: SharedEditorMessageBus,
}

impl EditorEventService {
    pub fn new(bus: SharedEditorMessageBus) -> Self {
        Self::with_retention_policy(bus, EditorEventRetentionPolicy::default())
    }

    pub fn with_retention_policy(
        bus: SharedEditorMessageBus,
        retention_policy: EditorEventRetentionPolicy,
    ) -> Self {
        Self {
            sequence_state: Mutex::new(EditorEventSequenceState::default()),
            journal: Mutex::new(EditorEventJournalStore::new(retention_policy.journal)),
            listeners: Mutex::new(EditorEventListenerRegistry::new(retention_policy.listeners)),
            bus,
        }
    }

    pub fn bus(&self) -> &SharedEditorMessageBus {
        &self.bus
    }

    pub(crate) fn begin_event(&self) -> EditorEventStamp {
        self.allocate_stamp(true)
    }

    pub(crate) fn begin_observation(&self) -> EditorEventStamp {
        self.allocate_stamp(false)
    }

    pub(crate) fn record(&self, record: EditorEventRecord) {
        let record = Arc::new(SharedEditorEventRecord::new(record));
        {
            self.lock_journal().push(Arc::clone(&record));
        }
        let routes = { self.lock_listeners().delivery_routes() };
        deliver_matching_arc(
            &routes,
            record,
            |route, record| route.accepts(record.record()),
            |route, record| route.enqueue(record),
        );
    }

    pub fn journal(&self) -> EditorEventJournal {
        self.lock_journal().snapshot()
    }

    fn allocate_stamp(&self, advances_revision: bool) -> EditorEventStamp {
        let mut state = self.lock_sequence_state();
        state.next_event_id = state.next_event_id.saturating_add(1);
        state.next_sequence = state.next_sequence.saturating_add(1);
        let before_revision = state.revision;
        if advances_revision {
            state.revision = state.revision.saturating_add(1);
        }
        EditorEventStamp {
            event_id: crate::core::editor_event::EditorEventId::new(state.next_event_id),
            sequence: crate::core::editor_event::EditorEventSequence::new(state.next_sequence),
            before_revision,
            after_revision: state.revision,
        }
    }

    fn lock_sequence_state(&self) -> MutexGuard<'_, EditorEventSequenceState> {
        self.sequence_state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_journal(&self) -> MutexGuard<'_, EditorEventJournalStore> {
        self.journal
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn lock_listeners(&self) -> MutexGuard<'_, EditorEventListenerRegistry> {
        self.listeners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[inline]
fn deliver_matching_arc<T, R>(
    routes: &[R],
    payload: Arc<T>,
    mut accepts: impl FnMut(&R, &T) -> bool,
    mut deliver: impl FnMut(&R, Arc<T>),
) {
    let mut pending_route = None;
    for route in routes {
        if accepts(route, &payload) {
            if let Some(previous_route) = pending_route.replace(route) {
                deliver(previous_route, Arc::clone(&payload));
            }
        }
    }
    if let Some(route) = pending_route {
        deliver(route, payload);
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::sync::Arc;
    use std::time::Instant;

    use super::deliver_matching_arc;

    const SAMPLE_PAIRS: usize = 17;
    const RECORDS_PER_SAMPLE: usize = 65_536;

    #[test]
    fn optimization_batch_hi_editor590_last_route_takes_the_existing_payload_owner() {
        let routes = [0, 1, 2, 3];
        let mut observed_strong_counts = Vec::new();
        let mut deliveries = Vec::new();

        deliver_matching_arc(
            &routes,
            Arc::new([7_u8; 64]),
            |route, _| route % 2 == 1,
            |route, payload| {
                observed_strong_counts.push(Arc::strong_count(&payload));
                deliveries.push((*route, payload));
            },
        );

        assert_eq!(observed_strong_counts, [2, 2]);
        assert_eq!(
            deliveries
                .iter()
                .map(|(route, _)| *route)
                .collect::<Vec<_>>(),
            [1, 3]
        );
        assert!(Arc::ptr_eq(&deliveries[0].1, &deliveries[1].1));
    }

    #[test]
    #[ignore = "release-only editor event last-route ownership benchmark"]
    fn optimization_batch_hi_editor590_last_route_move_performance_evidence() {
        fn payloads() -> Vec<Arc<[u8; 64]>> {
            (0..RECORDS_PER_SAMPLE)
                .map(|index| Arc::new([index as u8; 64]))
                .collect()
        }

        fn measure_legacy(records: Vec<Arc<[u8; 64]>>) -> u128 {
            let started = Instant::now();
            for record in records {
                black_box(Arc::clone(&record));
            }
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(records: Vec<Arc<[u8; 64]>>) -> u128 {
            let started = Instant::now();
            for record in records {
                deliver_matching_arc(
                    &[()],
                    record,
                    |_, _| true,
                    |_, record| {
                        black_box(record);
                    },
                );
            }
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        for _ in 0..4 {
            black_box(measure_legacy(payloads()));
            black_box(measure_optimized(payloads()));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy(payloads()));
                optimized_samples.push(measure_optimized(payloads()));
            } else {
                optimized_samples.push(measure_optimized(payloads()));
                legacy_samples.push(measure_legacy(payloads()));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);

        println!(
            "EDITOR590_EVENT_LAST_ROUTE_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
records_per_sample={RECORDS_PER_SAMPLE} legacy_arc_increments_per_sample={RECORDS_PER_SAMPLE} \
legacy_arc_decrements_per_sample={} optimized_arc_increments_per_sample=0 \
optimized_arc_decrements_per_sample={RECORDS_PER_SAMPLE} legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            RECORDS_PER_SAMPLE * 2,
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns.saturating_mul(4),
            "moving the final listener payload must reduce P95 by at least 20%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
