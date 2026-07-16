use crate::scene::event_mirror::{
    RuntimeEventMirrorDescriptor, RuntimeEventMirrorError, RuntimeEventMirrorRegistration,
    RuntimeEventMirrorSubscription,
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

        let mut subscription = registration.create_subscription(self);
        if !subscription.connect(self) {
            return Err(RuntimeEventMirrorError::ConnectionFailed {
                event_id: event_id.to_string(),
            });
        }
        let reader_count = match self.event_mirrors.increment_reader(event_id) {
            Ok(reader_count) => reader_count,
            Err(error) => {
                subscription.disconnect(self);
                return Err(error);
            }
        };
        if let Err(error) = registration.notify_reader_count(self, reader_count) {
            subscription.disconnect(self);
            if let Ok(rollback_count) = self.event_mirrors.decrement_reader(event_id) {
                let _ = registration.notify_reader_count(self, rollback_count);
            }
            return Err(error);
        }
        Ok(subscription)
    }

    pub fn unsubscribe_runtime_event_mirror(
        &mut self,
        subscription: &mut RuntimeEventMirrorSubscription,
    ) -> Result<bool, RuntimeEventMirrorError> {
        let registration = subscription.registration().clone();
        if !subscription.disconnect(self) {
            return Ok(false);
        }
        let event_id = registration.descriptor().event_id.as_str();
        let reader_count = match self.event_mirrors.decrement_reader(event_id) {
            Ok(reader_count) => reader_count,
            Err(error) => {
                subscription.connect(self);
                return Err(error);
            }
        };
        if let Err(error) = registration.notify_reader_count(self, reader_count) {
            if let Ok(rollback_count) = self.event_mirrors.increment_reader(event_id) {
                subscription.connect(self);
                let _ = registration.notify_reader_count(self, rollback_count);
            }
            return Err(error);
        }
        Ok(true)
    }

    pub fn drain_runtime_event_mirror(
        &self,
        subscription: &mut RuntimeEventMirrorSubscription,
    ) -> Result<Vec<serde_json::Value>, RuntimeEventMirrorError> {
        subscription.drain(self)
    }
}
