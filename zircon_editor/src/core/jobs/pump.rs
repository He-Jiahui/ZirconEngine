use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::core::editor_message::{
    EditorMessage, EditorMessagePayload, EditorTopic, SharedEditorMessageBus, TOPIC_JOB,
};

use super::{JobEvent, JobEventKind, JobId};

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

#[derive(Clone, Debug, Default)]
pub(super) struct JobEventQueue {
    inner: Arc<Mutex<JobEventQueueState>>,
}

#[derive(Debug, Default)]
struct JobEventQueueState {
    order: VecDeque<QueuedJobEvent>,
    latest_progress: BTreeMap<JobId, JobEvent>,
}

#[derive(Debug)]
enum QueuedJobEvent {
    Lifecycle(JobEvent),
    Progress(JobId),
}

impl JobEventQueue {
    pub(super) fn push(&self, event: JobEvent) {
        self.lock().push(event);
    }

    fn pop(&self) -> Option<JobEvent> {
        self.lock().pop()
    }

    fn lock(&self) -> MutexGuard<'_, JobEventQueueState> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl JobEventQueueState {
    fn push(&mut self, event: JobEvent) {
        let id = event.id();
        if matches!(event.kind(), JobEventKind::Progress { .. }) {
            if self.latest_progress.insert(id, event).is_none() {
                self.order.push_back(QueuedJobEvent::Progress(id));
            }
            return;
        }
        self.order.push_back(QueuedJobEvent::Lifecycle(event));
    }

    fn pop(&mut self) -> Option<JobEvent> {
        match self.order.pop_front()? {
            QueuedJobEvent::Lifecycle(event) => Some(event),
            QueuedJobEvent::Progress(id) => self.latest_progress.remove(&id),
        }
    }
}

pub(super) struct JobEventPump {
    bus: SharedEditorMessageBus,
    topic: EditorTopic,
    queue: JobEventQueue,
    consumer: Mutex<()>,
}

impl JobEventPump {
    pub(super) fn new(bus: SharedEditorMessageBus, queue: JobEventQueue) -> Self {
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
            let Some(event) = self.queue.pop() else {
                break;
            };
            self.bus.publish(
                self.topic.clone(),
                EditorMessage::new(EditorMessagePayload::Job(event)),
            );
            count += 1;
        }
        count
    }
}
