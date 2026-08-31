use std::collections::BTreeMap;

use crate::scene::event_mirror::{
    RuntimeEventMirrorDescriptor, RuntimeEventMirrorDrainPage, RuntimeEventMirrorError,
    RuntimeEventMirrorLifecycleDiagnostics, RuntimeEventMirrorReclaimReport,
    RuntimeEventMirrorRegistration, RuntimeEventMirrorSubscription,
    RuntimeEventMirrorSubscriptionHandle, RuntimeEventMirrorSubscriptionRecord,
};

use super::World;

impl World {
    pub fn register_runtime_event_mirror(
        &mut self,
        registration: RuntimeEventMirrorRegistration,
    ) -> Result<(), RuntimeEventMirrorError> {
        let apply = registration.clone();
        self.event_mirrors.insert(registration)?;
        apply.register_event(self);
        Ok(())
    }

    pub fn runtime_event_mirror_descriptor(
        &self,
        event_id: &str,
    ) -> Option<&RuntimeEventMirrorDescriptor> {
        self.event_mirrors
            .get(event_id)
            .map(RuntimeEventMirrorRegistration::descriptor)
    }

    pub fn subscribe_runtime_event_mirror(
        &mut self,
        event_id: &str,
        payload_schema: &str,
    ) -> Result<RuntimeEventMirrorSubscription, RuntimeEventMirrorError> {
        let registration = self.event_mirrors.get(event_id).cloned().ok_or_else(|| {
            RuntimeEventMirrorError::UnknownEventId {
                event_id: event_id.to_string(),
            }
        })?;
        let descriptor = registration.descriptor();
        if descriptor.payload_schema != payload_schema {
            return Err(RuntimeEventMirrorError::PayloadSchemaMismatch {
                event_id: event_id.to_string(),
                expected: descriptor.payload_schema.clone(),
                actual: payload_schema.to_string(),
            });
        }

        let mut record = registration.create_subscription_record();
        if !record.connect(self) {
            return Err(RuntimeEventMirrorError::ConnectionFailed {
                event_id: event_id.to_string(),
            });
        }
        let reader_count = match self.event_mirrors.increment_reader(event_id) {
            Ok(reader_count) => reader_count,
            Err(error) => {
                let disconnected = record.disconnect(self);
                debug_assert!(disconnected);
                return Err(error);
            }
        };
        if let Err(error) = registration.notify_reader_count(self, reader_count) {
            let disconnected = record.disconnect(self);
            debug_assert!(disconnected);
            if let Ok(rollback_count) = self.event_mirrors.decrement_reader(event_id) {
                let _ = registration.notify_reader_count(self, rollback_count);
            }
            return Err(error);
        }
        Ok(self.event_mirrors.allocate_subscription(record))
    }

    pub fn unsubscribe_runtime_event_mirror(
        &mut self,
        subscription: &mut RuntimeEventMirrorSubscription,
    ) -> Result<bool, RuntimeEventMirrorError> {
        let Some(handle) = subscription.handle() else {
            return Ok(false);
        };
        if !self.event_mirrors.owns_subscription(subscription) {
            return Ok(false);
        }
        let Some(mut record) = self.event_mirrors.take_subscription(handle) else {
            return Ok(false);
        };
        let registration = self
            .event_mirrors
            .get(record.event_id())
            .cloned()
            .expect("live runtime event mirror record retains its registration");
        let event_id = registration_event_id(&registration);
        if !record.disconnect(self) {
            self.event_mirrors.restore_subscription(handle, record);
            return Ok(false);
        }
        let reader_count = match self.event_mirrors.decrement_reader(event_id) {
            Ok(reader_count) => reader_count,
            Err(error) => {
                let reconnected = record.connect(self);
                debug_assert!(reconnected);
                self.event_mirrors.restore_subscription(handle, record);
                return Err(error);
            }
        };
        if let Err(error) = registration.notify_reader_count(self, reader_count) {
            let rollback_count = match self.event_mirrors.increment_reader(event_id) {
                Ok(rollback_count) => rollback_count,
                Err(rollback_error) => {
                    let reconnected = record.connect(self);
                    debug_assert!(reconnected);
                    self.event_mirrors.restore_subscription(handle, record);
                    return Err(rollback_error);
                }
            };
            let reconnected = record.connect(self);
            debug_assert!(reconnected);
            let _ = registration.notify_reader_count(self, rollback_count);
            self.event_mirrors.restore_subscription(handle, record);
            return Err(error);
        }
        self.event_mirrors.retire_subscription(handle);
        subscription.mark_disconnected();
        Ok(true)
    }

    pub fn drain_runtime_event_mirror(
        &self,
        subscription: &mut RuntimeEventMirrorSubscription,
    ) -> Result<Vec<serde_json::Value>, RuntimeEventMirrorError> {
        let handle = self.connected_runtime_event_mirror_handle(subscription)?;
        self.event_mirrors
            .drain_subscription(handle)
            .unwrap_or_else(|| {
                Err(RuntimeEventMirrorError::Disconnected {
                    event_id: subscription.descriptor().event_id.clone(),
                })
            })
    }

    pub(crate) fn drain_runtime_event_mirror_payloads(
        &self,
        subscription: &mut RuntimeEventMirrorSubscription,
        max_deliveries: usize,
    ) -> Result<RuntimeEventMirrorDrainPage, RuntimeEventMirrorError> {
        let handle = self.connected_runtime_event_mirror_handle(subscription)?;
        self.event_mirrors
            .drain_subscription_payloads(handle, max_deliveries)
            .unwrap_or_else(|| {
                Err(RuntimeEventMirrorError::Disconnected {
                    event_id: subscription.descriptor().event_id.clone(),
                })
            })
    }

    pub(crate) fn runtime_event_mirror_lifecycle_diagnostics(
        &self,
        event_id: &str,
    ) -> Option<RuntimeEventMirrorLifecycleDiagnostics> {
        self.event_mirrors.lifecycle_diagnostics(event_id)
    }

    pub(crate) fn reclaim_dropped_runtime_event_mirrors(
        &mut self,
    ) -> RuntimeEventMirrorReclaimReport {
        let handles = self.event_mirrors.drain_reclaim_intents();
        let mut report = RuntimeEventMirrorReclaimReport {
            attempted: handles.len(),
            ..RuntimeEventMirrorReclaimReport::default()
        };
        let mut disconnected_by_event = BTreeMap::new();

        for handle in handles {
            let Some(mut record) = self.event_mirrors.take_subscription(handle) else {
                continue;
            };
            if !record.disconnect(self) {
                self.event_mirrors.restore_subscription(handle, record);
                self.event_mirrors.requeue_reclaim(handle);
                report.retry_pending += 1;
                continue;
            }
            disconnected_by_event
                .entry(record.event_id().to_string())
                .or_insert_with(Vec::new)
                .push((handle, record));
        }

        for (event_id, records) in disconnected_by_event {
            let registration = self
                .event_mirrors
                .get(&event_id)
                .cloned()
                .expect("live runtime event mirror record retains its registration");
            let record_count = records.len();
            let mut decremented = 0_usize;
            let mut reader_count = 0_u32;
            let mut decrement_failed = false;
            for _ in 0..record_count {
                match self.event_mirrors.decrement_reader(&event_id) {
                    Ok(count) => {
                        decremented += 1;
                        reader_count = count;
                    }
                    Err(_) => {
                        decrement_failed = true;
                        break;
                    }
                }
            }

            if decrement_failed {
                for _ in 0..decremented {
                    let _ = self.event_mirrors.increment_reader(&event_id);
                }
                self.restore_runtime_event_mirror_reclaim_records(records);
                report.retry_pending += record_count;
                continue;
            }

            if registration
                .notify_reader_count(self, reader_count)
                .is_err()
            {
                report.callback_failures += 1;
                let mut rollback_count = reader_count;
                for _ in 0..record_count {
                    rollback_count = self
                        .event_mirrors
                        .increment_reader(&event_id)
                        .expect("runtime event mirror reclaim rollback cannot overflow");
                }
                self.restore_runtime_event_mirror_reclaim_records(records);
                let _ = registration.notify_reader_count(self, rollback_count);
                report.retry_pending += record_count;
                continue;
            }

            for (handle, _) in records {
                self.event_mirrors.retire_subscription(handle);
                report.reclaimed += 1;
            }
        }

        report
    }

    pub(crate) fn shutdown_runtime_event_mirrors(&mut self) -> RuntimeEventMirrorReclaimReport {
        for handle in self.event_mirrors.live_subscription_handles() {
            self.event_mirrors.requeue_reclaim(handle);
        }
        self.reclaim_dropped_runtime_event_mirrors()
    }

    fn connected_runtime_event_mirror_handle(
        &self,
        subscription: &RuntimeEventMirrorSubscription,
    ) -> Result<RuntimeEventMirrorSubscriptionHandle, RuntimeEventMirrorError> {
        if !self.event_mirrors.owns_subscription(subscription) {
            return Err(RuntimeEventMirrorError::Disconnected {
                event_id: subscription.descriptor().event_id.clone(),
            });
        }
        subscription
            .handle()
            .ok_or_else(|| RuntimeEventMirrorError::Disconnected {
                event_id: subscription.descriptor().event_id.clone(),
            })
    }

    fn restore_runtime_event_mirror_reclaim_records(
        &mut self,
        records: Vec<(
            RuntimeEventMirrorSubscriptionHandle,
            RuntimeEventMirrorSubscriptionRecord,
        )>,
    ) {
        for (handle, mut record) in records {
            let reconnected = record.connect(self);
            debug_assert!(reconnected);
            self.event_mirrors.restore_subscription(handle, record);
            self.event_mirrors.requeue_reclaim(handle);
        }
    }
}

fn registration_event_id(registration: &RuntimeEventMirrorRegistration) -> &str {
    registration.descriptor().event_id.as_str()
}

#[cfg(test)]
#[path = "event_mirror/borrowed_unsubscribe_id_tests.rs"]
mod borrowed_unsubscribe_id_tests;

impl Drop for World {
    fn drop(&mut self) {
        let _ = self.shutdown_runtime_event_mirrors();
    }
}
