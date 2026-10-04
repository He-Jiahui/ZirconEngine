use super::topic::EventBusState;
use crate::core::framework::events::EventBusCloseReceipt;

impl EventBusState {
    pub(super) fn close_admission(&self) -> EventBusCloseReceipt {
        {
            self.admission.lock().closed = true;
        }
        let topics = self.snapshot_topics();
        let mut drained = Vec::new();
        for topic in topics {
            let _delivery = topic.lock_delivery();
            for subscriber in topic.snapshot_subscribers().iter() {
                if let Some(queue) = subscriber.deactivate() {
                    drained.push((subscriber.clone(), queue));
                }
            }
        }
        let drained_events = drained.iter().map(|(_, queue)| queue.len()).sum();
        for (subscriber, queue) in drained {
            subscriber.record_deactivated_queue(queue);
        }
        EventBusCloseReceipt {
            drained_events,
            retention: self.admission.snapshot(),
        }
    }
}
