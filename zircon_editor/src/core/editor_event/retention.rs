use std::collections::VecDeque;
use std::io::{self, Write};
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

#[derive(Clone, Debug, PartialEq, Eq)]
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
        let encoded_bytes = encoded_size(&record);
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
}

#[derive(Debug)]
struct RetainedEditorEvent {
    payload: Arc<SharedEditorEventRecord>,
    retained_at: Instant,
}

#[derive(Debug, Default)]
struct RetentionQueue {
    entries: VecDeque<RetainedEditorEvent>,
    retained_bytes: usize,
    dropped_records: u64,
    coalesced_records: u64,
    first_dropped_sequence: Option<u64>,
    last_dropped_sequence: Option<u64>,
}

impl RetentionQueue {
    fn push(
        &mut self,
        payload: Arc<SharedEditorEventRecord>,
        budget: &EditorEventRetentionBudget,
        now: Instant,
    ) {
        self.prune_expired(budget, now);
        if payload.class == EditorEventRetentionClass::LatestState {
            if let Some(key) = payload.latest_state_key.as_ref() {
                if let Some(index) = self
                    .entries
                    .iter()
                    .position(|entry| entry.payload.latest_state_key.as_ref() == Some(key))
                {
                    self.coalesced_records = self.coalesced_records.saturating_add(1);
                    if self.entries[index].payload.record.sequence.0 >= payload.record.sequence.0 {
                        return;
                    }
                    self.remove_at(index, false);
                }
            }
        }

        self.retained_bytes = self.retained_bytes.saturating_add(payload.encoded_bytes);
        self.entries.push_back(RetainedEditorEvent {
            payload,
            retained_at: now,
        });
        while self.entries.len() > budget.max_records || self.retained_bytes > budget.max_bytes {
            self.drop_front();
        }
    }

    fn prune_expired(&mut self, budget: &EditorEventRetentionBudget, now: Instant) {
        while self
            .entries
            .front()
            .is_some_and(|entry| now.saturating_duration_since(entry.retained_at) > budget.max_age)
        {
            self.drop_front();
        }
    }

    fn drop_front(&mut self) {
        self.remove_at(0, true);
    }

    fn remove_at(&mut self, index: usize, counts_as_drop: bool) {
        let Some(entry) = self.entries.remove(index) else {
            return;
        };
        self.retained_bytes = self
            .retained_bytes
            .saturating_sub(entry.payload.encoded_bytes);
        if counts_as_drop {
            self.dropped_records = self.dropped_records.saturating_add(1);
            let sequence = entry.payload.record.sequence.0;
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

    fn acknowledge_through(&mut self, sequence: u64) -> usize {
        let before = self.entries.len();
        let mut removed_bytes = 0usize;
        self.entries.retain(|entry| {
            let retained = entry.payload.record.sequence.0 > sequence;
            if !retained {
                removed_bytes = removed_bytes.saturating_add(entry.payload.encoded_bytes);
            }
            retained
        });
        self.retained_bytes = self.retained_bytes.saturating_sub(removed_bytes);
        before - self.entries.len()
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
                .entries
                .front()
                .map(|entry| entry.payload.record.sequence.0),
            last_retained_sequence: self
                .entries
                .back()
                .map(|entry| entry.payload.record.sequence.0),
            oldest_retained_age_millis: self
                .entries
                .front()
                .map(|entry| duration_millis(now.saturating_duration_since(entry.retained_at))),
        }
    }
}

#[derive(Debug)]
pub(crate) struct EditorEventRetentionStore {
    budgets: EditorEventRetentionBudgets,
    durable_replay: RetentionQueue,
    frame_local: RetentionQueue,
    latest_state: RetentionQueue,
}

impl EditorEventRetentionStore {
    pub(crate) fn new(budgets: EditorEventRetentionBudgets) -> Self {
        Self {
            budgets,
            durable_replay: RetentionQueue::default(),
            frame_local: RetentionQueue::default(),
            latest_state: RetentionQueue::default(),
        }
    }

    pub(crate) fn push(&mut self, payload: Arc<SharedEditorEventRecord>) {
        let now = Instant::now();
        let class = payload.class;
        let budget = self.budgets.budget_for(class).clone();
        self.queue_mut(class).push(payload, &budget, now);
    }

    pub(crate) fn records(&mut self) -> Vec<Arc<SharedEditorEventRecord>> {
        let now = Instant::now();
        self.prune_expired(now);
        let mut records = Vec::with_capacity(
            self.durable_replay.entries.len()
                + self.frame_local.entries.len()
                + self.latest_state.entries.len(),
        );
        for queue in [&self.durable_replay, &self.frame_local, &self.latest_state] {
            records.extend(queue.entries.iter().map(|entry| Arc::clone(&entry.payload)));
        }
        records.sort_unstable_by_key(|payload| payload.record.sequence.0);
        records
    }

    pub(crate) fn acknowledge_through(&mut self, sequence: u64) -> usize {
        self.durable_replay.acknowledge_through(sequence)
            + self.frame_local.acknowledge_through(sequence)
            + self.latest_state.acknowledge_through(sequence)
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

fn encoded_size(record: &EditorEventRecord) -> usize {
    let mut counter = CountingWriter::default();
    serde_json::to_writer(&mut counter, record)
        .map(|()| counter.bytes)
        .unwrap_or(std::mem::size_of_val(record))
}

#[derive(Default)]
struct CountingWriter {
    bytes: usize,
}

impl Write for CountingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes = self.bytes.saturating_add(bytes.len());
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn retention_status_and_ack_do_not_materialize_or_rescan_remaining_records() {
        let source = include_str!("retention.rs");
        let acknowledge_body = source
            .split("fn acknowledge_through")
            .nth(1)
            .and_then(|body| body.split("fn diagnostics").next())
            .expect("retention acknowledge body should remain available");
        assert!(acknowledge_body.contains("removed_bytes"));
        assert!(!acknowledge_body.contains(".map(|entry| entry.payload.encoded_bytes)"));

        let diagnostics_body = source
            .split("fn diagnostics")
            .nth(1)
            .and_then(|body| {
                body.split("pub(crate) struct EditorEventRetentionStore")
                    .next()
            })
            .expect("retention diagnostics body should remain available");
        assert!(diagnostics_body.contains(".front()"));
        assert!(diagnostics_body.contains(".back()"));
    }
}
