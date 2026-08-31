use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::scene::ecs::Event;
use crate::scene::{SceneResult, World};

use super::subscription::{
    lock_runtime_event_mirror_reclaim_queue, RuntimeEventMirrorReclaimQueue,
    RuntimeEventMirrorSubscriptionHandle, RuntimeEventMirrorSubscriptionRecord,
};
use super::{RuntimeEventMirrorDrainPage, RuntimeEventMirrorError, RuntimeEventMirrorSubscription};

type ReaderCountCallback = dyn Fn(&mut World, u32) -> SceneResult<()> + Send + Sync;
const RUNTIME_EVENT_MIRROR_DESCRIPTOR_MAX_BYTES: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeEventMirrorDescriptor {
    pub event_id: String,
    pub payload_schema: String,
}

impl RuntimeEventMirrorDescriptor {
    pub fn new(event_id: impl Into<String>, payload_schema: impl Into<String>) -> Self {
        Self {
            event_id: event_id.into(),
            payload_schema: payload_schema.into(),
        }
    }
}

trait RuntimeEventMirrorFactory: Send + Sync {
    fn register_event(&self, world: &mut World);
    fn create_subscription_record(&self, event_id: String) -> RuntimeEventMirrorSubscriptionRecord;
}

struct TypedRuntimeEventMirrorFactory<E>(PhantomData<fn() -> E>);

impl<E> Default for TypedRuntimeEventMirrorFactory<E> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<E> RuntimeEventMirrorFactory for TypedRuntimeEventMirrorFactory<E>
where
    E: Event + Serialize,
{
    fn register_event(&self, world: &mut World) {
        world.register_event::<E>();
    }

    fn create_subscription_record(&self, event_id: String) -> RuntimeEventMirrorSubscriptionRecord {
        RuntimeEventMirrorSubscriptionRecord::typed::<E>(event_id)
    }
}

#[derive(Clone)]
pub struct RuntimeEventMirrorRegistration {
    descriptor: RuntimeEventMirrorDescriptor,
    factory: Arc<dyn RuntimeEventMirrorFactory>,
    reader_count_callback: Option<Arc<ReaderCountCallback>>,
}

impl RuntimeEventMirrorRegistration {
    pub fn typed<E>(event_id: impl Into<String>, payload_schema: impl Into<String>) -> Self
    where
        E: Event + Serialize,
    {
        Self {
            descriptor: RuntimeEventMirrorDescriptor::new(event_id, payload_schema),
            factory: Arc::new(TypedRuntimeEventMirrorFactory::<E>::default()),
            reader_count_callback: None,
        }
    }

    pub fn with_reader_count_callback(
        mut self,
        callback: impl Fn(&mut World, u32) -> SceneResult<()> + Send + Sync + 'static,
    ) -> Self {
        self.reader_count_callback = Some(Arc::new(callback));
        self
    }

    pub fn descriptor(&self) -> &RuntimeEventMirrorDescriptor {
        &self.descriptor
    }

    pub(crate) fn register_event(&self, world: &mut World) {
        self.factory.register_event(world);
    }

    pub(crate) fn create_subscription_record(&self) -> RuntimeEventMirrorSubscriptionRecord {
        self.factory
            .create_subscription_record(self.descriptor.event_id.clone())
    }

    pub(crate) fn notify_reader_count(
        &self,
        world: &mut World,
        reader_count: u32,
    ) -> Result<(), RuntimeEventMirrorError> {
        let Some(callback) = &self.reader_count_callback else {
            return Ok(());
        };
        callback(world, reader_count).map_err(|error| {
            RuntimeEventMirrorError::ReaderCountCallback {
                event_id: self.descriptor.event_id.clone(),
                message: error.to_string(),
            }
        })
    }
}

impl fmt::Debug for RuntimeEventMirrorRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeEventMirrorRegistration")
            .field("descriptor", &self.descriptor)
            .field(
                "has_reader_count_callback",
                &self.reader_count_callback.is_some(),
            )
            .finish_non_exhaustive()
    }
}

struct RuntimeEventMirrorSubscriptionSlot {
    generation: u64,
    record: Option<RuntimeEventMirrorSubscriptionRecord>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimeEventMirrorLifecycleDiagnostics {
    pub live_subscriptions: usize,
    pub pending_reclaims: usize,
    pub reclaim_budget: usize,
    pub reader_count: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimeEventMirrorReclaimReport {
    pub attempted: usize,
    pub reclaimed: usize,
    pub retry_pending: usize,
    pub callback_failures: usize,
}

pub(crate) struct RuntimeEventMirrorRegistry {
    registrations: BTreeMap<String, RuntimeEventMirrorRegistration>,
    reader_counts: HashMap<String, u32>,
    subscription_slots: Vec<RuntimeEventMirrorSubscriptionSlot>,
    free_subscription_slots: Vec<usize>,
    reclaim_queue: Arc<Mutex<RuntimeEventMirrorReclaimQueue>>,
}

impl Default for RuntimeEventMirrorRegistry {
    fn default() -> Self {
        Self {
            registrations: BTreeMap::new(),
            reader_counts: HashMap::new(),
            subscription_slots: Vec::new(),
            free_subscription_slots: Vec::new(),
            reclaim_queue: Arc::new(Mutex::new(RuntimeEventMirrorReclaimQueue::default())),
        }
    }
}

impl Clone for RuntimeEventMirrorRegistry {
    fn clone(&self) -> Self {
        Self {
            registrations: self.registrations.clone(),
            reader_counts: self
                .registrations
                .keys()
                .map(|event_id| (event_id.clone(), 0))
                .collect(),
            ..Self::default()
        }
    }
}

impl RuntimeEventMirrorRegistry {
    pub(crate) fn insert(
        &mut self,
        registration: RuntimeEventMirrorRegistration,
    ) -> Result<(), RuntimeEventMirrorError> {
        let descriptor = registration.descriptor();
        if descriptor.event_id.trim().is_empty() {
            return Err(RuntimeEventMirrorError::EmptyEventId);
        }
        if descriptor.payload_schema.trim().is_empty() {
            return Err(RuntimeEventMirrorError::EmptyPayloadSchema {
                event_id: descriptor.event_id.clone(),
            });
        }
        for (field, value) in [
            ("event id", descriptor.event_id.as_str()),
            ("payload schema", descriptor.payload_schema.as_str()),
        ] {
            if value.len() > RUNTIME_EVENT_MIRROR_DESCRIPTOR_MAX_BYTES {
                return Err(RuntimeEventMirrorError::DescriptorTooLarge {
                    event_id: descriptor.event_id.clone(),
                    field,
                    actual_bytes: value.len(),
                    max_bytes: RUNTIME_EVENT_MIRROR_DESCRIPTOR_MAX_BYTES,
                });
            }
        }
        if self.registrations.contains_key(&descriptor.event_id) {
            return Err(RuntimeEventMirrorError::DuplicateEventId {
                event_id: descriptor.event_id.clone(),
            });
        }
        let event_id = descriptor.event_id.clone();
        self.registrations.insert(event_id.clone(), registration);
        self.reader_counts.insert(event_id, 0);
        Ok(())
    }

    pub(crate) fn get(&self, event_id: &str) -> Option<&RuntimeEventMirrorRegistration> {
        self.registrations.get(event_id)
    }

    pub(crate) fn increment_reader(
        &mut self,
        event_id: &str,
    ) -> Result<u32, RuntimeEventMirrorError> {
        let count = self.reader_counts.get_mut(event_id).ok_or_else(|| {
            RuntimeEventMirrorError::UnknownEventId {
                event_id: event_id.to_string(),
            }
        })?;
        *count =
            count
                .checked_add(1)
                .ok_or_else(|| RuntimeEventMirrorError::ReaderCountOverflow {
                    event_id: event_id.to_string(),
                })?;
        Ok(*count)
    }

    pub(crate) fn decrement_reader(
        &mut self,
        event_id: &str,
    ) -> Result<u32, RuntimeEventMirrorError> {
        let count = self.reader_counts.get_mut(event_id).ok_or_else(|| {
            RuntimeEventMirrorError::UnknownEventId {
                event_id: event_id.to_string(),
            }
        })?;
        *count =
            count
                .checked_sub(1)
                .ok_or_else(|| RuntimeEventMirrorError::ReaderCountUnderflow {
                    event_id: event_id.to_string(),
                })?;
        Ok(*count)
    }

    pub(crate) fn allocate_subscription(
        &mut self,
        record: RuntimeEventMirrorSubscriptionRecord,
    ) -> RuntimeEventMirrorSubscription {
        let mut record = Some(record);
        let handle = loop {
            let Some(slot_index) = self.free_subscription_slots.pop() else {
                let slot_index = self.subscription_slots.len();
                self.subscription_slots
                    .push(RuntimeEventMirrorSubscriptionSlot {
                        generation: 1,
                        record: record.take(),
                    });
                break RuntimeEventMirrorSubscriptionHandle::new(slot_index, 1);
            };
            let slot = &mut self.subscription_slots[slot_index];
            let Some(generation) = slot.generation.checked_add(1) else {
                continue;
            };
            slot.generation = generation;
            slot.record = record.take();
            break RuntimeEventMirrorSubscriptionHandle::new(slot_index, generation);
        };
        lock_runtime_event_mirror_reclaim_queue(&self.reclaim_queue).register_live_record(handle);
        let event_id = self
            .subscription_record(handle)
            .expect("allocated runtime event mirror record")
            .event_id()
            .to_string();
        let descriptor = self
            .registrations
            .get(&event_id)
            .expect("subscription event registration remains available")
            .descriptor()
            .clone();
        RuntimeEventMirrorSubscription::new(descriptor, handle, Arc::clone(&self.reclaim_queue))
    }

    pub(crate) fn owns_subscription(&self, subscription: &RuntimeEventMirrorSubscription) -> bool {
        subscription.belongs_to(&self.reclaim_queue)
    }

    pub(crate) fn take_subscription(
        &mut self,
        handle: RuntimeEventMirrorSubscriptionHandle,
    ) -> Option<RuntimeEventMirrorSubscriptionRecord> {
        let slot = self.subscription_slots.get_mut(handle.slot())?;
        (slot.generation == handle.generation())
            .then(|| slot.record.take())
            .flatten()
    }

    pub(crate) fn restore_subscription(
        &mut self,
        handle: RuntimeEventMirrorSubscriptionHandle,
        record: RuntimeEventMirrorSubscriptionRecord,
    ) {
        let slot = self
            .subscription_slots
            .get_mut(handle.slot())
            .expect("runtime event mirror subscription slot remains allocated");
        assert_eq!(slot.generation, handle.generation());
        assert!(slot.record.replace(record).is_none());
    }

    pub(crate) fn retire_subscription(&mut self, handle: RuntimeEventMirrorSubscriptionHandle) {
        let slot = self
            .subscription_slots
            .get_mut(handle.slot())
            .expect("runtime event mirror subscription slot remains allocated");
        assert_eq!(slot.generation, handle.generation());
        assert!(slot.record.is_none());
        self.free_subscription_slots.push(handle.slot());
        lock_runtime_event_mirror_reclaim_queue(&self.reclaim_queue).retire_live_record(handle);
    }

    pub(crate) fn drain_subscription(
        &self,
        handle: RuntimeEventMirrorSubscriptionHandle,
    ) -> Option<Result<Vec<serde_json::Value>, RuntimeEventMirrorError>> {
        self.subscription_record(handle)
            .map(RuntimeEventMirrorSubscriptionRecord::drain)
    }

    pub(crate) fn drain_subscription_payloads(
        &self,
        handle: RuntimeEventMirrorSubscriptionHandle,
        max_deliveries: usize,
    ) -> Option<Result<RuntimeEventMirrorDrainPage, RuntimeEventMirrorError>> {
        self.subscription_record(handle)
            .map(|record| record.drain_payloads_up_to(max_deliveries))
    }

    pub(crate) fn drain_reclaim_intents(&self) -> Vec<RuntimeEventMirrorSubscriptionHandle> {
        lock_runtime_event_mirror_reclaim_queue(&self.reclaim_queue).drain()
    }

    pub(crate) fn requeue_reclaim(&self, handle: RuntimeEventMirrorSubscriptionHandle) {
        lock_runtime_event_mirror_reclaim_queue(&self.reclaim_queue).enqueue(handle);
    }

    pub(crate) fn live_subscription_handles(&self) -> Vec<RuntimeEventMirrorSubscriptionHandle> {
        self.subscription_slots
            .iter()
            .enumerate()
            .filter_map(|(slot, record)| {
                record
                    .record
                    .as_ref()
                    .map(|_| RuntimeEventMirrorSubscriptionHandle::new(slot, record.generation))
            })
            .collect()
    }

    pub(crate) fn lifecycle_diagnostics(
        &self,
        event_id: &str,
    ) -> Option<RuntimeEventMirrorLifecycleDiagnostics> {
        let reader_count = self.reader_counts.get(event_id).copied()?;
        let reclaim = lock_runtime_event_mirror_reclaim_queue(&self.reclaim_queue);
        let pending_reclaims = reclaim
            .pending_handles()
            .filter(|handle| {
                self.subscription_record(**handle)
                    .is_some_and(|record| record.event_id() == event_id)
            })
            .count();
        Some(RuntimeEventMirrorLifecycleDiagnostics {
            live_subscriptions: self
                .subscription_slots
                .iter()
                .filter_map(|slot| slot.record.as_ref())
                .filter(|record| record.event_id() == event_id)
                .count(),
            pending_reclaims,
            reclaim_budget: reclaim.live_record_budget(),
            reader_count,
        })
    }

    fn subscription_record(
        &self,
        handle: RuntimeEventMirrorSubscriptionHandle,
    ) -> Option<&RuntimeEventMirrorSubscriptionRecord> {
        let slot = self.subscription_slots.get(handle.slot())?;
        (slot.generation == handle.generation())
            .then_some(slot.record.as_ref())
            .flatten()
    }
}

impl fmt::Debug for RuntimeEventMirrorRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reader_counts = self
            .registrations
            .keys()
            .filter_map(|event_id| {
                self.reader_counts
                    .get(event_id)
                    .map(|count| (event_id, count))
            })
            .collect::<BTreeMap<_, _>>();
        formatter
            .debug_struct("RuntimeEventMirrorRegistry")
            .field("event_ids", &self.registrations.keys().collect::<Vec<_>>())
            .field("reader_counts", &reader_counts)
            .field(
                "live_subscription_count",
                &lock_runtime_event_mirror_reclaim_queue(&self.reclaim_queue).live_record_budget(),
            )
            .finish()
    }
}

impl PartialEq for RuntimeEventMirrorRegistry {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[cfg(test)]
#[path = "registration/hash_index_tests.rs"]
mod hash_index_tests;
