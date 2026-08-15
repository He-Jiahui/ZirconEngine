use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::{
    EditorAnimationEvent, EditorEvent, EditorEventRecord, EditorEventTransient, EditorViewportEvent,
};

const MIB: usize = 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorEventRetentionBudget {
    max_records: usize,
    max_bytes: usize,
    max_age: Duration,
}

impl EditorEventRetentionBudget {
    pub fn new(max_records: usize, max_bytes: usize, max_age: Duration) -> Result<Self, String> {
        if max_records == 0 {
            return Err("editor event retention max_records must be greater than zero".to_string());
        }
        if max_bytes == 0 {
            return Err("editor event retention max_bytes must be greater than zero".to_string());
        }
        if max_age.is_zero() {
            return Err("editor event retention max_age must be greater than zero".to_string());
        }
        Ok(Self {
            max_records,
            max_bytes,
            max_age,
        })
    }

    pub fn max_records(&self) -> usize {
        self.max_records
    }

    pub fn max_bytes(&self) -> usize {
        self.max_bytes
    }

    pub fn max_age(&self) -> Duration {
        self.max_age
    }

    fn snapshot(&self) -> EditorEventRetentionBudgetSnapshot {
        EditorEventRetentionBudgetSnapshot {
            max_records: self.max_records,
            max_bytes: self.max_bytes,
            max_age_millis: duration_millis(self.max_age),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorEventRetentionBudgetSnapshot {
    pub max_records: usize,
    pub max_bytes: usize,
    pub max_age_millis: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorEventRetentionBudgets {
    pub durable_replay: EditorEventRetentionBudget,
    pub frame_local: EditorEventRetentionBudget,
    pub latest_state: EditorEventRetentionBudget,
}

impl EditorEventRetentionBudgets {
    pub fn new(
        durable_replay: EditorEventRetentionBudget,
        frame_local: EditorEventRetentionBudget,
        latest_state: EditorEventRetentionBudget,
    ) -> Self {
        Self {
            durable_replay,
            frame_local,
            latest_state,
        }
    }

    fn budget_for(&self, class: EditorEventRetentionClass) -> &EditorEventRetentionBudget {
        match class {
            EditorEventRetentionClass::DurableReplay => &self.durable_replay,
            EditorEventRetentionClass::FrameLocal => &self.frame_local,
            EditorEventRetentionClass::LatestState => &self.latest_state,
        }
    }

    fn snapshot(&self) -> EditorEventRetentionBudgetsSnapshot {
        EditorEventRetentionBudgetsSnapshot {
            durable_replay: self.durable_replay.snapshot(),
            frame_local: self.frame_local.snapshot(),
            latest_state: self.latest_state.snapshot(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorEventRetentionBudgetsSnapshot {
    pub durable_replay: EditorEventRetentionBudgetSnapshot,
    pub frame_local: EditorEventRetentionBudgetSnapshot,
    pub latest_state: EditorEventRetentionBudgetSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorEventRetentionPolicy {
    pub journal: EditorEventRetentionBudgets,
    pub listeners: EditorEventRetentionBudgets,
}

impl EditorEventRetentionPolicy {
    pub fn new(
        journal: EditorEventRetentionBudgets,
        listeners: EditorEventRetentionBudgets,
    ) -> Self {
        Self { journal, listeners }
    }
}

impl Default for EditorEventRetentionPolicy {
    fn default() -> Self {
        Self {
            journal: EditorEventRetentionBudgets::new(
                budget(16_384, 64 * MIB, Duration::from_secs(24 * 60 * 60)),
                budget(512, 4 * MIB, Duration::from_secs(2)),
                budget(256, 4 * MIB, Duration::from_secs(30 * 60)),
            ),
            listeners: EditorEventRetentionBudgets::new(
                budget(1_024, 16 * MIB, Duration::from_secs(10 * 60)),
                budget(128, MIB, Duration::from_secs(2)),
                budget(128, 2 * MIB, Duration::from_secs(10 * 60)),
            ),
        }
    }
}

fn budget(max_records: usize, max_bytes: usize, max_age: Duration) -> EditorEventRetentionBudget {
    EditorEventRetentionBudget::new(max_records, max_bytes, max_age)
        .expect("default editor event retention budget is valid")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorEventRetentionClass {
    DurableReplay,
    FrameLocal,
    LatestState,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorEventRetentionClassDiagnostics {
    pub retained_records: usize,
    pub retained_bytes: usize,
    pub dropped_records: u64,
    pub coalesced_records: u64,
    pub first_dropped_sequence: Option<u64>,
    pub last_dropped_sequence: Option<u64>,
    pub first_retained_sequence: Option<u64>,
    pub last_retained_sequence: Option<u64>,
    pub oldest_retained_age_millis: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorEventRetentionDiagnostics {
    pub durable_replay: EditorEventRetentionClassDiagnostics,
    pub frame_local: EditorEventRetentionClassDiagnostics,
    pub latest_state: EditorEventRetentionClassDiagnostics,
}

impl EditorEventRetentionDiagnostics {
    pub fn retained_records(&self) -> usize {
        self.durable_replay.retained_records
            + self.frame_local.retained_records
            + self.latest_state.retained_records
    }

    pub fn retained_bytes(&self) -> usize {
        self.durable_replay.retained_bytes
            + self.frame_local.retained_bytes
            + self.latest_state.retained_bytes
    }

    pub fn dropped_records(&self) -> u64 {
        self.durable_replay.dropped_records
            + self.frame_local.dropped_records
            + self.latest_state.dropped_records
    }

    pub fn coalesced_records(&self) -> u64 {
        self.durable_replay.coalesced_records
            + self.frame_local.coalesced_records
            + self.latest_state.coalesced_records
    }

    pub fn first_dropped_sequence(&self) -> Option<u64> {
        [
            self.durable_replay.first_dropped_sequence,
            self.frame_local.first_dropped_sequence,
            self.latest_state.first_dropped_sequence,
        ]
        .into_iter()
        .flatten()
        .min()
    }

    pub fn last_dropped_sequence(&self) -> Option<u64> {
        [
            self.durable_replay.last_dropped_sequence,
            self.frame_local.last_dropped_sequence,
            self.latest_state.last_dropped_sequence,
        ]
        .into_iter()
        .flatten()
        .max()
    }

    pub fn first_retained_sequence(&self) -> Option<u64> {
        [
            self.durable_replay.first_retained_sequence,
            self.frame_local.first_retained_sequence,
            self.latest_state.first_retained_sequence,
        ]
        .into_iter()
        .flatten()
        .min()
    }

    pub fn last_retained_sequence(&self) -> Option<u64> {
        [
            self.durable_replay.last_retained_sequence,
            self.frame_local.last_retained_sequence,
            self.latest_state.last_retained_sequence,
        ]
        .into_iter()
        .flatten()
        .max()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
enum EditorEventLatestStateKey {
    PointerPosition,
    ViewportSize,
    TimelineCursor,
    HoverNode,
    FocusNode,
    PressNode,
    DrawerResize(String),
}

#[derive(Debug)]
pub(crate) struct SharedEditorEventRecord {
    record: EditorEventRecord,
    encoded_bytes: usize,
    class: EditorEventRetentionClass,
    latest_state_key: Option<EditorEventLatestStateKey>,
}

impl SharedEditorEventRecord {
    pub(crate) fn new(record: EditorEventRecord) -> Self {
        let encoded_bytes = serde_json::to_vec(&record)
            .map(|encoded| encoded.len())
            .unwrap_or_else(|_| std::mem::size_of_val(&record));
        let class = retention_class(&record.event);
        let latest_state_key = latest_state_key(&record.event);
        Self {
            record,
            encoded_bytes,
            class,
            latest_state_key,
        }
    }

    pub(crate) fn record(&self) -> &EditorEventRecord {
        &self.record
    }

    fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }
}

#[derive(Debug)]
struct RetainedEditorEvent {
    payload: Arc<SharedEditorEventRecord>,
    retained_at: Instant,
}

#[derive(Debug, Default)]
struct RetentionQueue {
    entries: BTreeMap<u64, RetainedEditorEvent>,
    retained_by_age: BTreeSet<(Instant, u64)>,
    retained_by_event_sequence: BTreeSet<(u64, u64)>,
    latest_state_sequences: HashMap<EditorEventLatestStateKey, u64>,
    retained_bytes: usize,
    dropped_records: u64,
    coalesced_records: u64,
    first_dropped_sequence: Option<u64>,
    last_dropped_sequence: Option<u64>,
}

impl RetentionQueue {
    fn push(
        &mut self,
        delivery_cursor: u64,
        payload: Arc<SharedEditorEventRecord>,
        budget: &EditorEventRetentionBudget,
        now: Instant,
    ) {
        self.prune_expired(budget, now);
        if payload.class == EditorEventRetentionClass::LatestState {
            if let Some(key) = payload.latest_state_key.as_ref() {
                if let Some(previous_cursor) = self.latest_state_sequences.get(key).copied() {
                    self.coalesced_records = self.coalesced_records.saturating_add(1);
                    let previous_sequence = self
                        .entries
                        .get(&previous_cursor)
                        .map(|entry| entry.payload.record.sequence.0)
                        .unwrap_or_default();
                    if previous_sequence >= payload.record.sequence.0 {
                        return;
                    }
                    self.remove_cursor(previous_cursor, false);
                }
            }
        }

        self.remove_cursor(delivery_cursor, false);
        self.retained_bytes = self.retained_bytes.saturating_add(payload.encoded_bytes());
        self.retained_by_age.insert((now, delivery_cursor));
        self.retained_by_event_sequence
            .insert((payload.record.sequence.0, delivery_cursor));
        if let Some(key) = payload.latest_state_key.clone() {
            self.latest_state_sequences.insert(key, delivery_cursor);
        }
        self.entries.insert(
            delivery_cursor,
            RetainedEditorEvent {
                payload,
                retained_at: now,
            },
        );
        while self.entries.len() > budget.max_records || self.retained_bytes > budget.max_bytes {
            self.drop_front();
        }
    }

    fn prune_expired(&mut self, budget: &EditorEventRetentionBudget, now: Instant) {
        while let Some((retained_at, delivery_cursor)) = self.retained_by_age.iter().next().copied()
        {
            if now.saturating_duration_since(retained_at) <= budget.max_age {
                break;
            }
            self.remove_cursor(delivery_cursor, true);
        }
    }

    fn drop_front(&mut self) {
        if let Some(delivery_cursor) = self
            .entries
            .first_key_value()
            .map(|(delivery_cursor, _)| *delivery_cursor)
        {
            self.remove_cursor(delivery_cursor, true);
        }
    }

    fn remove_cursor(&mut self, delivery_cursor: u64, counts_as_drop: bool) {
        let Some(entry) = self.entries.remove(&delivery_cursor) else {
            return;
        };
        let sequence = entry.payload.record.sequence.0;
        self.retained_by_age
            .remove(&(entry.retained_at, delivery_cursor));
        self.retained_by_event_sequence
            .remove(&(sequence, delivery_cursor));
        if let Some(key) = entry.payload.latest_state_key.as_ref() {
            if self.latest_state_sequences.get(key) == Some(&delivery_cursor) {
                self.latest_state_sequences.remove(key);
            }
        }
        self.retained_bytes = self
            .retained_bytes
            .saturating_sub(entry.payload.encoded_bytes());
        if counts_as_drop {
            self.dropped_records = self.dropped_records.saturating_add(1);
            self.first_dropped_sequence = Some(
                self.first_dropped_sequence
                    .map_or(sequence, |first| first.min(sequence)),
            );
            self.last_dropped_sequence = Some(
                self.last_dropped_sequence
                    .map_or(sequence, |last| last.max(sequence)),
            );
        }
    }

    fn acknowledge_through_delivery_cursor(&mut self, delivery_cursor: u64) -> usize {
        let mut removed = 0usize;
        while let Some(retained_cursor) = self
            .entries
            .first_key_value()
            .map(|(retained_cursor, _)| *retained_cursor)
        {
            if retained_cursor > delivery_cursor {
                break;
            }
            self.remove_cursor(retained_cursor, false);
            removed = removed.saturating_add(1);
        }
        removed
    }

    fn next_after(&self, sequence: u64) -> Option<(u64, &RetainedEditorEvent)> {
        self.entries
            .range((
                std::ops::Bound::Excluded(sequence),
                std::ops::Bound::Unbounded,
            ))
            .next()
            .map(|(sequence, entry)| (*sequence, entry))
    }

    fn next_event_after(
        &self,
        after: Option<(u64, u64)>,
    ) -> Option<(u64, u64, &RetainedEditorEvent)> {
        let entry = match after {
            Some(after) => self
                .retained_by_event_sequence
                .range((std::ops::Bound::Excluded(after), std::ops::Bound::Unbounded))
                .next(),
            None => self.retained_by_event_sequence.iter().next(),
        }?;
        self.entries
            .get(&entry.1)
            .map(|retained| (entry.0, entry.1, retained))
    }

    fn diagnostics(&self, now: Instant) -> EditorEventRetentionClassDiagnostics {
        EditorEventRetentionClassDiagnostics {
            retained_records: self.entries.len(),
            retained_bytes: self.retained_bytes,
            dropped_records: self.dropped_records,
            coalesced_records: self.coalesced_records,
            first_dropped_sequence: self.first_dropped_sequence,
            last_dropped_sequence: self.last_dropped_sequence,
            first_retained_sequence: self
                .retained_by_event_sequence
                .first()
                .map(|(sequence, _)| *sequence),
            last_retained_sequence: self
                .retained_by_event_sequence
                .last()
                .map(|(sequence, _)| *sequence),
            oldest_retained_age_millis: self.retained_by_age.first().map(|(retained_at, _)| {
                duration_millis(now.saturating_duration_since(*retained_at))
            }),
        }
    }
}

#[derive(Debug)]
pub(crate) struct EditorEventRetentionPage {
    pub(crate) records: Vec<EditorEventRetentionPageRecord>,
    pub(crate) has_more: bool,
}

#[derive(Debug)]
pub(crate) struct EditorEventRetentionPageRecord {
    pub(crate) delivery_cursor: u64,
    pub(crate) payload: Arc<SharedEditorEventRecord>,
}

#[derive(Debug)]
pub(crate) struct EditorEventRetentionStore {
    budgets: EditorEventRetentionBudgets,
    next_delivery_cursor: u64,
    durable_replay: RetentionQueue,
    frame_local: RetentionQueue,
    latest_state: RetentionQueue,
}

impl EditorEventRetentionStore {
    pub(crate) fn new(budgets: EditorEventRetentionBudgets) -> Self {
        Self {
            budgets,
            next_delivery_cursor: 0,
            durable_replay: RetentionQueue::default(),
            frame_local: RetentionQueue::default(),
            latest_state: RetentionQueue::default(),
        }
    }

    pub(crate) fn push(&mut self, payload: Arc<SharedEditorEventRecord>) {
        let now = Instant::now();
        let class = payload.class;
        let budget = self.budgets.budget_for(class).clone();
        self.next_delivery_cursor = self.next_delivery_cursor.saturating_add(1);
        let delivery_cursor = self.next_delivery_cursor;
        self.queue_mut(class)
            .push(delivery_cursor, payload, &budget, now);
    }

    pub(crate) fn records(&mut self) -> Vec<Arc<SharedEditorEventRecord>> {
        let now = Instant::now();
        self.prune_expired(now);
        let queues = [&self.durable_replay, &self.frame_local, &self.latest_state];
        let mut candidates = [
            queues[0]
                .next_event_after(None)
                .map(|(sequence, cursor, _)| (sequence, cursor)),
            queues[1]
                .next_event_after(None)
                .map(|(sequence, cursor, _)| (sequence, cursor)),
            queues[2]
                .next_event_after(None)
                .map(|(sequence, cursor, _)| (sequence, cursor)),
        ];
        let total_records = queues.iter().map(|queue| queue.entries.len()).sum();
        let mut records = Vec::with_capacity(total_records);
        while let Some((queue_index, (sequence, delivery_cursor))) = candidates
            .iter()
            .enumerate()
            .filter_map(|(index, candidate)| candidate.map(|candidate| (index, candidate)))
            .min_by_key(|(_, candidate)| *candidate)
        {
            let Some(entry) = queues[queue_index].entries.get(&delivery_cursor) else {
                candidates[queue_index] = None;
                continue;
            };
            records.push(Arc::clone(&entry.payload));
            candidates[queue_index] = queues[queue_index]
                .next_event_after(Some((sequence, delivery_cursor)))
                .map(|(next_sequence, next_cursor, _)| (next_sequence, next_cursor));
        }
        records
    }

    pub(crate) fn records_page_after_delivery_cursor(
        &mut self,
        after_delivery_cursor: u64,
        max_records: usize,
    ) -> EditorEventRetentionPage {
        let now = Instant::now();
        self.prune_expired(now);
        let queues = [&self.durable_replay, &self.frame_local, &self.latest_state];
        let mut candidates = [
            queues[0]
                .next_after(after_delivery_cursor)
                .map(|(cursor, _)| cursor),
            queues[1]
                .next_after(after_delivery_cursor)
                .map(|(cursor, _)| cursor),
            queues[2]
                .next_after(after_delivery_cursor)
                .map(|(cursor, _)| cursor),
        ];
        let total_records = queues.iter().map(|queue| queue.entries.len()).sum();
        let mut records = Vec::with_capacity(max_records.min(total_records));
        while records.len() < max_records {
            let Some((queue_index, delivery_cursor)) = candidates
                .iter()
                .enumerate()
                .filter_map(|(index, candidate)| candidate.map(|cursor| (index, cursor)))
                .min_by_key(|(_, cursor)| *cursor)
            else {
                break;
            };
            let Some(entry) = queues[queue_index].entries.get(&delivery_cursor) else {
                candidates[queue_index] = None;
                continue;
            };
            records.push(EditorEventRetentionPageRecord {
                delivery_cursor,
                payload: Arc::clone(&entry.payload),
            });
            candidates[queue_index] = queues[queue_index]
                .next_after(delivery_cursor)
                .map(|(next_cursor, _)| next_cursor);
        }
        EditorEventRetentionPage {
            records,
            has_more: candidates.iter().any(Option::is_some),
        }
    }

    pub(crate) fn acknowledge_through_delivery_cursor(&mut self, delivery_cursor: u64) -> usize {
        self.durable_replay
            .acknowledge_through_delivery_cursor(delivery_cursor)
            + self
                .frame_local
                .acknowledge_through_delivery_cursor(delivery_cursor)
            + self
                .latest_state
                .acknowledge_through_delivery_cursor(delivery_cursor)
    }

    pub(crate) fn diagnostics(&mut self) -> EditorEventRetentionDiagnostics {
        let now = Instant::now();
        self.prune_expired(now);
        EditorEventRetentionDiagnostics {
            durable_replay: self.durable_replay.diagnostics(now),
            frame_local: self.frame_local.diagnostics(now),
            latest_state: self.latest_state.diagnostics(now),
        }
    }

    pub(crate) fn budgets(&self) -> EditorEventRetentionBudgetsSnapshot {
        self.budgets.snapshot()
    }

    fn prune_expired(&mut self, now: Instant) {
        self.durable_replay
            .prune_expired(&self.budgets.durable_replay, now);
        self.frame_local
            .prune_expired(&self.budgets.frame_local, now);
        self.latest_state
            .prune_expired(&self.budgets.latest_state, now);
    }

    fn queue_mut(&mut self, class: EditorEventRetentionClass) -> &mut RetentionQueue {
        match class {
            EditorEventRetentionClass::DurableReplay => &mut self.durable_replay,
            EditorEventRetentionClass::FrameLocal => &mut self.frame_local,
            EditorEventRetentionClass::LatestState => &mut self.latest_state,
        }
    }
}

fn duration_millis(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

fn retention_class(event: &EditorEvent) -> EditorEventRetentionClass {
    match event {
        EditorEvent::Viewport(
            EditorViewportEvent::PointerMoved { .. } | EditorViewportEvent::Resized { .. },
        )
        | EditorEvent::Animation(EditorAnimationEvent::ScrubTimeline { .. })
        | EditorEvent::Transient(
            EditorEventTransient::HoverNode { .. }
            | EditorEventTransient::FocusNode { .. }
            | EditorEventTransient::PressNode { .. }
            | EditorEventTransient::SetDrawerResizing { .. },
        ) => EditorEventRetentionClass::LatestState,
        EditorEvent::Viewport(
            EditorViewportEvent::LeftPressed { .. }
            | EditorViewportEvent::LeftReleased
            | EditorViewportEvent::CancelInteraction
            | EditorViewportEvent::RightPressed { .. }
            | EditorViewportEvent::RightReleased
            | EditorViewportEvent::MiddlePressed { .. }
            | EditorViewportEvent::MiddleReleased
            | EditorViewportEvent::Scrolled { .. },
        )
        | EditorEvent::Transient(
            EditorEventTransient::BeginViewDrag { .. } | EditorEventTransient::EndViewDrag,
        ) => EditorEventRetentionClass::FrameLocal,
        _ => EditorEventRetentionClass::DurableReplay,
    }
}

fn latest_state_key(event: &EditorEvent) -> Option<EditorEventLatestStateKey> {
    match event {
        EditorEvent::Viewport(EditorViewportEvent::PointerMoved { .. }) => {
            Some(EditorEventLatestStateKey::PointerPosition)
        }
        EditorEvent::Viewport(EditorViewportEvent::Resized { .. }) => {
            Some(EditorEventLatestStateKey::ViewportSize)
        }
        EditorEvent::Animation(EditorAnimationEvent::ScrubTimeline { .. }) => {
            Some(EditorEventLatestStateKey::TimelineCursor)
        }
        EditorEvent::Transient(EditorEventTransient::HoverNode { .. }) => {
            Some(EditorEventLatestStateKey::HoverNode)
        }
        EditorEvent::Transient(EditorEventTransient::FocusNode { .. }) => {
            Some(EditorEventLatestStateKey::FocusNode)
        }
        EditorEvent::Transient(EditorEventTransient::PressNode { .. }) => {
            Some(EditorEventLatestStateKey::PressNode)
        }
        EditorEvent::Transient(EditorEventTransient::SetDrawerResizing { drawer_id, .. }) => {
            Some(EditorEventLatestStateKey::DrawerResize(drawer_id.clone()))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{retention_class, EditorEvent, EditorEventRetentionClass, EditorViewportEvent};

    #[test]
    fn cancel_interaction_is_frame_local() {
        assert_eq!(
            retention_class(&EditorEvent::Viewport(
                EditorViewportEvent::CancelInteraction
            )),
            EditorEventRetentionClass::FrameLocal
        );
    }

    #[test]
    fn retention_acknowledgement_and_pages_keep_delivery_cursor_indexed() {
        let source = include_str!("retention.rs");
        let acknowledge_body = source
            .split("fn acknowledge_through")
            .nth(1)
            .and_then(|body| body.split("fn diagnostics").next())
            .expect("retention acknowledge body should remain available");
        assert!(acknowledge_body.contains("first_key_value"));
        assert!(acknowledge_body.contains("delivery_cursor"));
        assert!(!acknowledge_body.contains("retained_by_event_sequence"));
        assert!(!acknowledge_body.contains(".retain("));

        let diagnostics_body = source
            .split("fn diagnostics")
            .nth(1)
            .and_then(|body| {
                body.split("pub(crate) struct EditorEventRetentionStore")
                    .next()
            })
            .expect("retention diagnostics body should remain available");
        assert!(diagnostics_body.contains("retained_by_event_sequence"));
        assert!(diagnostics_body.contains("retained_by_age"));

        let page_body = source
            .split("fn records_page_after")
            .nth(1)
            .and_then(|body| body.split("pub(crate) fn acknowledge_through").next())
            .expect("retention page body should remain available");
        assert!(page_body.contains("after_delivery_cursor"));
        assert!(page_body.contains("next_after"));
        assert!(!page_body.contains("sort_unstable"));
        let counting_writer_name = ["Counting", "Writer"].concat();
        assert!(!source.contains(&counting_writer_name));
    }
}
