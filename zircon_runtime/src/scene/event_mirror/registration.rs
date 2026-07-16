use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use serde::Serialize;

use crate::scene::ecs::Event;
use crate::scene::{SceneResult, World};

use super::{RuntimeEventMirrorError, RuntimeEventMirrorSubscription};

type ReaderCountCallback = dyn Fn(&mut World, u32) -> SceneResult<()> + Send + Sync;

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
    fn create_subscription(&self, world: &mut World) -> RuntimeEventMirrorSubscription;
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

    fn create_subscription(&self, world: &mut World) -> RuntimeEventMirrorSubscription {
        RuntimeEventMirrorSubscription::typed(world.register_dormant_event_subscription::<E>())
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

    pub(crate) fn create_subscription(&self, world: &mut World) -> RuntimeEventMirrorSubscription {
        let mut subscription = self.factory.create_subscription(world);
        subscription.attach_registration(self.clone());
        subscription
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

#[derive(Clone, Default)]
pub(crate) struct RuntimeEventMirrorRegistry {
    registrations: BTreeMap<String, RuntimeEventMirrorRegistration>,
    reader_counts: BTreeMap<String, u32>,
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
}

impl fmt::Debug for RuntimeEventMirrorRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeEventMirrorRegistry")
            .field("event_ids", &self.registrations.keys().collect::<Vec<_>>())
            .field("reader_counts", &self.reader_counts)
            .finish()
    }
}

impl PartialEq for RuntimeEventMirrorRegistry {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
