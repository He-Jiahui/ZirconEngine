use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Mutex, MutexGuard};

use crate::core::editor_message::{
    EditorMessage, EditorMessagePayload, EditorTopic, SharedEditorMessageBus, TOPIC_JOB,
};

use super::JobEvent;

pub(super) struct JobEventPump {
    bus: SharedEditorMessageBus,
    receiver: Mutex<Receiver<JobEvent>>,
}

impl JobEventPump {
    pub(super) fn new(bus: SharedEditorMessageBus, receiver: Receiver<JobEvent>) -> Self {
        Self {
            bus,
            receiver: Mutex::new(receiver),
        }
    }

    pub(super) fn pump(&self) -> usize {
        let topic = match EditorTopic::parse(TOPIC_JOB) {
            Ok(topic) => topic,
            Err(_) => return 0,
        };
        let receiver = self.lock_receiver();
        let mut count = 0;
        loop {
            match receiver.try_recv() {
                Ok(event) => {
                    self.bus.publish(
                        topic.clone(),
                        EditorMessage::new(EditorMessagePayload::Job(event)),
                    );
                    count += 1;
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => return count,
            }
        }
    }

    fn lock_receiver(&self) -> MutexGuard<'_, Receiver<JobEvent>> {
        self.receiver
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
