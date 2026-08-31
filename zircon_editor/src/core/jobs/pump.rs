use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::core::editor_message::{
    EditorMessage, EditorMessagePayload, EditorTopic, SharedEditorMessageBus, TOPIC_JOB,
};

use super::event_journal::{EditorJobEventJournal, EditorJobEventJournalRecord};

pub const DEFAULT_JOB_EVENT_PUMP_BUDGET: JobEventPumpBudget =
    JobEventPumpBudget::new(64, Duration::from_millis(1));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JobEventPumpBudget {
    max_events: usize,
    max_elapsed: Duration,
}

impl JobEventPumpBudget {
    pub const fn new(max_events: usize, max_elapsed: Duration) -> Self {
        Self {
            max_events,
            max_elapsed,
        }
    }

    pub const fn max_events(self) -> usize {
        self.max_events
    }

    pub const fn max_elapsed(self) -> Duration {
        self.max_elapsed
    }
}

pub(super) struct JobEventPump {
    bus: SharedEditorMessageBus,
    topic: EditorTopic,
    queue: EditorJobEventJournal,
    consumer: Mutex<()>,
}

impl JobEventPump {
    pub(super) fn new(bus: SharedEditorMessageBus, queue: EditorJobEventJournal) -> Self {
        Self {
            bus,
            topic: EditorTopic::parse(TOPIC_JOB)
                .expect("the built-in editor job topic must remain valid"),
            queue,
            consumer: Mutex::new(()),
        }
    }

    pub(super) fn pump(&self, budget: JobEventPumpBudget) -> usize {
        let started = Instant::now();
        self.pump_with_elapsed(budget, || started.elapsed())
    }

    pub(super) fn pump_with_elapsed(
        &self,
        budget: JobEventPumpBudget,
        mut elapsed: impl FnMut() -> Duration,
    ) -> usize {
        if budget.max_events == 0 || budget.max_elapsed.is_zero() {
            return 0;
        }
        let _consumer = self
            .consumer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut count = 0;
        while count < budget.max_events && elapsed() < budget.max_elapsed {
            let Some(record) = self.queue.pop() else {
                break;
            };
            let payload = match &record {
                EditorJobEventJournalRecord::Event { event, .. } => {
                    EditorMessagePayload::Job(event.clone())
                }
                EditorJobEventJournalRecord::Gap(gap) => {
                    EditorMessagePayload::JobJournalGap(gap.clone())
                }
            };
            let report = self
                .bus
                .publish(self.topic.clone(), EditorMessage::new(payload));
            if report.error().is_some() || !report.backpressured().is_empty() {
                self.queue.restore_front(record);
                break;
            }
            count += 1;
        }
        count
    }
}
