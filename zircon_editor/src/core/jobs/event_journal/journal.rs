use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use super::gap::JOB_EVENT_JOURNAL_GAP_RETAINED_BYTES;
use super::{EditorJobEventJournalGap, EditorJobEventJournalLimits, EditorJobEventJournalSnapshot};
use crate::core::jobs::{JobEvent, JobEventKind, JobId};

#[derive(Clone, Debug)]
pub(super) struct EditorJobEventJournal {
    inner: Arc<Mutex<EditorJobEventJournalState>>,
}

impl EditorJobEventJournal {
    pub(super) fn new(limits: EditorJobEventJournalLimits) -> Self {
        Self {
            inner: Arc::new(Mutex::new(EditorJobEventJournalState::new(limits))),
        }
    }

    pub(super) fn push(&self, event: JobEvent) {
        self.lock().push(event, Instant::now());
    }

    pub(super) fn pop(&self) -> Option<EditorJobEventJournalRecord> {
        self.lock().pop(Instant::now())
    }

    pub(super) fn restore_front(&self, record: EditorJobEventJournalRecord) {
        self.lock().restore_front(record);
    }

    pub(super) fn snapshot(&self) -> EditorJobEventJournalSnapshot {
        self.lock().snapshot(Instant::now())
    }

    pub(super) fn limits(&self) -> EditorJobEventJournalLimits {
        self.lock().limits
    }

    fn lock(&self) -> MutexGuard<'_, EditorJobEventJournalState> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for EditorJobEventJournal {
    fn default() -> Self {
        Self::new(EditorJobEventJournalLimits::default())
    }
}

#[derive(Clone, Debug)]
pub(super) enum EditorJobEventJournalRecord {
    Event {
        event: JobEvent,
        queued_at: Instant,
        retained_bytes: usize,
    },
    Gap(EditorJobEventJournalGap),
}

#[derive(Debug)]
struct EditorJobEventJournalState {
    limits: EditorJobEventJournalLimits,
    next_sequence: u64,
    events: BTreeMap<u64, QueuedJobEvent>,
    latest_progress: BTreeMap<JobId, u64>,
    gap: Option<EditorJobEventJournalGap>,
    retained_event_bytes: usize,
    high_water_depth: usize,
    high_water_retained_bytes: usize,
    coalesced_progress_events: u64,
    dropped_progress_events: u64,
    dropped_lifecycle_events: u64,
    sequence_exhausted: bool,
}

#[derive(Debug)]
struct QueuedJobEvent {
    sequence: u64,
    queued_at: Instant,
    retained_bytes: usize,
    event: JobEvent,
}

impl EditorJobEventJournalState {
    fn new(limits: EditorJobEventJournalLimits) -> Self {
        Self {
            limits: limits.normalized(),
            next_sequence: 0,
            events: BTreeMap::new(),
            latest_progress: BTreeMap::new(),
            gap: None,
            retained_event_bytes: 0,
            high_water_depth: 0,
            high_water_retained_bytes: 0,
            coalesced_progress_events: 0,
            dropped_progress_events: 0,
            dropped_lifecycle_events: 0,
            sequence_exhausted: false,
        }
    }

    fn push(&mut self, event: JobEvent, now: Instant) {
        self.prune_expired(now);
        let Some(sequence) = self.next_sequence.checked_add(1) else {
            self.sequence_exhausted = true;
            self.note_unsequenced_drop(&event);
            return;
        };
        self.next_sequence = sequence;

        let event = event.with_journal_sequence(sequence);
        let retained_bytes = event.estimated_retained_bytes();
        if matches!(event.kind(), JobEventKind::Progress { .. }) {
            if let Some(previous) = self.latest_progress.remove(&event.id()) {
                self.remove_event(previous);
                self.coalesced_progress_events = self.coalesced_progress_events.saturating_add(1);
            }
        }

        if retained_bytes > self.limits.max_retained_bytes() {
            self.note_drop(sequence, &event);
            self.enforce_limits();
            self.update_high_water();
            return;
        }

        if matches!(event.kind(), JobEventKind::Progress { .. }) {
            self.latest_progress.insert(event.id(), sequence);
        }
        self.retained_event_bytes = self.retained_event_bytes.saturating_add(retained_bytes);
        self.events.insert(
            sequence,
            QueuedJobEvent {
                sequence,
                queued_at: now,
                retained_bytes,
                event,
            },
        );
        self.enforce_limits();
        self.update_high_water();
    }

    fn pop(&mut self, now: Instant) -> Option<EditorJobEventJournalRecord> {
        self.prune_expired(now);
        let gap_is_next = self.gap.as_ref().is_some_and(|gap| {
            self.events
                .first_key_value()
                .is_none_or(|(sequence, _)| gap.first_dropped_sequence() <= *sequence)
        });
        if gap_is_next {
            return self.gap.take().map(EditorJobEventJournalRecord::Gap);
        }
        let (_, queued) = self.events.pop_first()?;
        self.finish_remove(&queued);
        Some(EditorJobEventJournalRecord::Event {
            event: queued.event,
            queued_at: queued.queued_at,
            retained_bytes: queued.retained_bytes,
        })
    }

    fn restore_front(&mut self, record: EditorJobEventJournalRecord) {
        match record {
            EditorJobEventJournalRecord::Gap(gap) => self.merge_gap(gap),
            EditorJobEventJournalRecord::Event {
                event,
                queued_at,
                retained_bytes,
            } => {
                let sequence = event.journal_sequence();
                if self.events.contains_key(&sequence) {
                    return;
                }
                if matches!(event.kind(), JobEventKind::Progress { .. }) {
                    if self
                        .latest_progress
                        .get(&event.id())
                        .is_some_and(|latest_sequence| *latest_sequence > sequence)
                    {
                        self.coalesced_progress_events =
                            self.coalesced_progress_events.saturating_add(1);
                        return;
                    }
                    self.latest_progress.insert(event.id(), sequence);
                }
                self.retained_event_bytes =
                    self.retained_event_bytes.saturating_add(retained_bytes);
                self.events.entry(sequence).or_insert(QueuedJobEvent {
                    sequence,
                    queued_at,
                    retained_bytes,
                    event,
                });
            }
        }
        self.enforce_limits();
        self.update_high_water();
    }

    fn snapshot(&mut self, now: Instant) -> EditorJobEventJournalSnapshot {
        self.prune_expired(now);
        EditorJobEventJournalSnapshot {
            depth: self.depth(),
            retained_bytes: self.retained_bytes(),
            oldest_age: self
                .events
                .first_key_value()
                .map(|(_, queued)| now.saturating_duration_since(queued.queued_at)),
            high_water_depth: self.high_water_depth,
            high_water_retained_bytes: self.high_water_retained_bytes,
            coalesced_progress_events: self.coalesced_progress_events,
            dropped_progress_events: self.dropped_progress_events,
            dropped_lifecycle_events: self.dropped_lifecycle_events,
            sequence_exhausted: self.sequence_exhausted,
        }
    }

    fn prune_expired(&mut self, now: Instant) {
        loop {
            let expired = self.events.first_key_value().is_some_and(|(_, queued)| {
                now.saturating_duration_since(queued.queued_at) > self.limits.max_oldest_age()
            });
            if !expired {
                break;
            }
            let Some((_, queued)) = self.events.pop_first() else {
                break;
            };
            self.finish_remove(&queued);
            self.note_drop(queued.sequence, &queued.event);
        }
        self.enforce_limits();
        self.update_high_water();
    }

    fn enforce_limits(&mut self) {
        while self.depth() > self.limits.max_entries()
            || self.retained_bytes() > self.limits.max_retained_bytes()
        {
            let Some((_, queued)) = self.events.pop_first() else {
                break;
            };
            self.finish_remove(&queued);
            self.note_drop(queued.sequence, &queued.event);
        }
    }

    fn remove_event(&mut self, sequence: u64) {
        if let Some(queued) = self.events.remove(&sequence) {
            self.finish_remove(&queued);
        }
    }

    fn finish_remove(&mut self, queued: &QueuedJobEvent) {
        self.retained_event_bytes = self
            .retained_event_bytes
            .saturating_sub(queued.retained_bytes);
        if matches!(queued.event.kind(), JobEventKind::Progress { .. })
            && self.latest_progress.get(&queued.event.id()) == Some(&queued.sequence)
        {
            self.latest_progress.remove(&queued.event.id());
        }
    }

    fn note_unsequenced_drop(&mut self, event: &JobEvent) {
        if matches!(event.kind(), JobEventKind::Progress { .. }) {
            self.dropped_progress_events = self.dropped_progress_events.saturating_add(1);
        } else {
            self.dropped_lifecycle_events = self.dropped_lifecycle_events.saturating_add(1);
            self.merge_gap(EditorJobEventJournalGap::single(u64::MAX));
        }
    }

    fn note_drop(&mut self, sequence: u64, event: &JobEvent) {
        if matches!(event.kind(), JobEventKind::Progress { .. }) {
            self.dropped_progress_events = self.dropped_progress_events.saturating_add(1);
        } else {
            self.dropped_lifecycle_events = self.dropped_lifecycle_events.saturating_add(1);
            self.merge_gap(EditorJobEventJournalGap::single(sequence));
        }
    }

    fn merge_gap(&mut self, mut gap: EditorJobEventJournalGap) {
        if let Some(current) = self.gap.take() {
            gap.merge(current);
        }

        let covered_sequences = self
            .events
            .range(gap.first_dropped_sequence()..=gap.last_dropped_sequence())
            .map(|(sequence, _)| *sequence)
            .collect::<Vec<_>>();
        for sequence in covered_sequences {
            let Some(queued) = self.events.remove(&sequence) else {
                continue;
            };
            self.finish_remove(&queued);
            if matches!(queued.event.kind(), JobEventKind::Progress { .. }) {
                self.dropped_progress_events = self.dropped_progress_events.saturating_add(1);
            } else {
                self.dropped_lifecycle_events = self.dropped_lifecycle_events.saturating_add(1);
                gap.merge(EditorJobEventJournalGap::single(sequence));
            }
        }
        self.gap = Some(gap);
    }

    fn depth(&self) -> usize {
        self.events
            .len()
            .saturating_add(usize::from(self.gap.is_some()))
    }

    fn retained_bytes(&self) -> usize {
        self.retained_event_bytes
            .saturating_add(if self.gap.is_some() {
                JOB_EVENT_JOURNAL_GAP_RETAINED_BYTES
            } else {
                0
            })
    }

    fn update_high_water(&mut self) {
        self.high_water_depth = self.high_water_depth.max(self.depth());
        self.high_water_retained_bytes = self.high_water_retained_bytes.max(self.retained_bytes());
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::super::gap::JOB_EVENT_JOURNAL_GAP_RETAINED_BYTES;
    use super::{EditorJobEventJournal, EditorJobEventJournalRecord};
    use crate::core::jobs::{
        EditorJobEventJournalLimits, JobCategory, JobEvent, JobEventKind, JobId,
    };

    #[test]
    fn newer_gap_does_not_overtake_an_older_retained_event() {
        let retained = lifecycle_event(1, "retained");
        let retained_bytes = retained.estimated_retained_bytes();
        let max_retained_bytes = retained_bytes + JOB_EVENT_JOURNAL_GAP_RETAINED_BYTES;
        let journal =
            EditorJobEventJournal::new(EditorJobEventJournalLimits::new(8, max_retained_bytes));

        journal.push(retained);
        journal.push(lifecycle_event(2, &"x".repeat(max_retained_bytes + 1)));

        assert!(matches!(
            journal.pop(),
            Some(EditorJobEventJournalRecord::Event { event, .. })
                if event.journal_sequence() == 1
        ));
        assert!(matches!(
            journal.pop(),
            Some(EditorJobEventJournalRecord::Gap(gap))
                if gap.first_dropped_sequence() == 2
                    && gap.last_dropped_sequence() == 2
        ));
    }

    #[test]
    fn merged_gap_absorbs_retained_events_between_dropped_sequences() {
        let retained = lifecycle_event(1, "retained");
        let retained_bytes = retained.estimated_retained_bytes();
        let max_retained_bytes = retained_bytes
            .saturating_mul(2)
            .saturating_add(JOB_EVENT_JOURNAL_GAP_RETAINED_BYTES);
        let oversized = "x".repeat(max_retained_bytes + 1);
        let journal =
            EditorJobEventJournal::new(EditorJobEventJournalLimits::new(8, max_retained_bytes));

        journal.push(retained);
        journal.push(lifecycle_event(2, &oversized));
        journal.push(lifecycle_event(3, "between-gaps"));
        journal.push(lifecycle_event(4, &oversized));

        assert!(matches!(
            journal.pop(),
            Some(EditorJobEventJournalRecord::Event { event, .. })
                if event.journal_sequence() == 1
        ));
        assert!(matches!(
            journal.pop(),
            Some(EditorJobEventJournalRecord::Gap(gap))
                if gap.dropped_lifecycle_events() == 3
                    && gap.first_dropped_sequence() == 2
                    && gap.last_dropped_sequence() == 4
        ));
        assert!(journal.pop().is_none());
    }

    #[test]
    fn restoring_backpressured_progress_preserves_the_newer_coalescing_index() {
        let journal = EditorJobEventJournal::default();
        journal.push(progress_event("first"));
        let backpressured = journal.pop().expect("first progress event");

        journal.push(progress_event("second"));
        journal.restore_front(backpressured);
        journal.push(progress_event("third"));

        assert!(matches!(
            journal.pop(),
            Some(EditorJobEventJournalRecord::Event { event, .. })
                if matches!(event.kind(), JobEventKind::Progress { message, .. } if message == "third")
        ));
        assert!(journal.pop().is_none());
        assert_eq!(journal.snapshot().coalesced_progress_events(), 2);
    }

    fn lifecycle_event(id: u64, label: &str) -> JobEvent {
        JobEvent::new(
            JobId::new(id),
            Arc::<str>::from(label),
            JobCategory::Misc,
            JobEventKind::Started,
        )
    }

    fn progress_event(message: &str) -> JobEvent {
        JobEvent::new(
            JobId::new(1),
            Arc::<str>::from("progress"),
            JobCategory::Misc,
            JobEventKind::Progress {
                completed: 1,
                total: 3,
                message: message.to_string(),
            },
        )
    }
}
