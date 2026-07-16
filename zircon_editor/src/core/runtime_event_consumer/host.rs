use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

use zircon_runtime_interface::ZrRuntimePluginEventSubscriptionHandle;

use crate::core::gateway::{EditorRuntimeGateway, EditorRuntimeGatewayHandle};

use super::{
    EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerRegistration,
    EditorRuntimeEventConsumerRegistry,
};

struct ActiveConsumer {
    registration: EditorRuntimeEventConsumerRegistration,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
    last_sequence: Option<u64>,
}

pub struct EditorRuntimeEventConsumerHost {
    gateway: EditorRuntimeGatewayHandle,
    registry: Mutex<EditorRuntimeEventConsumerRegistry>,
    active: Mutex<BTreeMap<String, ActiveConsumer>>,
    play_session_id: Mutex<Option<u64>>,
}

impl Default for EditorRuntimeEventConsumerHost {
    fn default() -> Self {
        Self::new(EditorRuntimeGatewayHandle::detached())
    }
}

impl EditorRuntimeEventConsumerHost {
    pub fn new(gateway: EditorRuntimeGatewayHandle) -> Self {
        Self {
            gateway,
            registry: Mutex::new(EditorRuntimeEventConsumerRegistry::default()),
            active: Mutex::new(BTreeMap::new()),
            play_session_id: Mutex::new(None),
        }
    }

    pub fn runtime_session_id(&self) -> u64 {
        self.gateway.session_handle().raw()
    }

    pub fn register(
        &self,
        registry: EditorRuntimeEventConsumerRegistry,
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        self.registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .extend(registry)
    }

    pub fn begin_play_session(
        &self,
        play_session_id: u64,
        enabled_capabilities: &[String],
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        let mut session = self
            .play_session_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(active) = *session {
            return Err(EditorRuntimeEventConsumerError::SessionAlreadyActive {
                play_session_id: active,
            });
        }
        *session = Some(play_session_id);
        drop(session);

        if let Err(error) = self.reconcile_enabled_capabilities(enabled_capabilities) {
            return Err(match self.end_play_session(play_session_id) {
                Ok(()) => error,
                Err(cleanup) => EditorRuntimeEventConsumerError::with_cleanup(
                    "begin runtime event consumer session",
                    error,
                    cleanup,
                ),
            });
        }
        Ok(())
    }

    pub fn reconcile_enabled_capabilities(
        &self,
        enabled_capabilities: &[String],
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        let play_session_id = self
            .play_session_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .ok_or(EditorRuntimeEventConsumerError::NoActiveSession)?;
        let registrations = self
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .registrations()
            .cloned()
            .collect::<Vec<_>>();
        let desired = registrations
            .into_iter()
            .filter(|registration| {
                let required = &registration.manifest().required_capability;
                required.is_empty()
                    || enabled_capabilities
                        .iter()
                        .any(|capability| capability == required)
            })
            .map(|registration| (registration.manifest().consumer_id.clone(), registration))
            .collect::<BTreeMap<_, _>>();
        let gateway = self.gateway.clone();
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let removed = active
            .keys()
            .filter(|consumer_id| !desired.contains_key(*consumer_id))
            .cloned()
            .collect::<Vec<_>>();
        let mut first_error = None;
        for consumer_id in removed {
            let Some(subscription) = active
                .get(&consumer_id)
                .map(|consumer| consumer.subscription)
            else {
                continue;
            };
            match unsubscribe_consumer(&gateway, &consumer_id, subscription) {
                Ok(()) => {
                    if let Some(consumer) = active.remove(&consumer_id) {
                        consumer.registration.end_session(play_session_id);
                    }
                }
                Err(error) => {
                    first_error.get_or_insert(error);
                }
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }

        let existing = active.keys().cloned().collect::<BTreeSet<_>>();
        let mut added = Vec::new();
        for (consumer_id, registration) in desired {
            if existing.contains(&consumer_id) {
                continue;
            }
            let manifest = registration.manifest().clone();
            let subscription = match gateway
                .subscribe_plugin_event(&manifest.event_id, &manifest.payload_schema)
            {
                Ok(Some(subscription)) => subscription,
                Ok(None) => {
                    let error = EditorRuntimeEventConsumerError::Unsupported {
                        consumer_id: manifest.consumer_id.clone(),
                    };
                    return Err(
                        match rollback_added_consumers(
                            &gateway,
                            &mut active,
                            &added,
                            play_session_id,
                        ) {
                            Some(cleanup) => EditorRuntimeEventConsumerError::with_cleanup(
                                "reconcile runtime event consumers",
                                error,
                                cleanup,
                            ),
                            None => error,
                        },
                    );
                }
                Err(message) => {
                    let error = EditorRuntimeEventConsumerError::Gateway {
                        consumer_id: manifest.consumer_id.clone(),
                        message: message.to_string(),
                    };
                    return Err(
                        match rollback_added_consumers(
                            &gateway,
                            &mut active,
                            &added,
                            play_session_id,
                        ) {
                            Some(cleanup) => EditorRuntimeEventConsumerError::with_cleanup(
                                "reconcile runtime event consumers",
                                error,
                                cleanup,
                            ),
                            None => error,
                        },
                    );
                }
            };
            registration.begin_session(play_session_id);
            active.insert(
                manifest.consumer_id.clone(),
                ActiveConsumer {
                    registration,
                    subscription,
                    last_sequence: None,
                },
            );
            added.push(manifest.consumer_id);
        }
        Ok(())
    }

    pub fn pump(&self) -> Result<usize, EditorRuntimeEventConsumerError> {
        let play_session_id = self
            .play_session_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .unwrap_or_default();
        if play_session_id == 0 {
            return Ok(0);
        }
        let gateway = self.gateway.clone();
        let runtime_session_id = gateway.session_handle().raw();
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut applied = 0;
        for consumer in active.values_mut() {
            let manifest = consumer.registration.manifest();
            let deliveries =
                gateway
                    .drain_plugin_events(consumer.subscription)
                    .map_err(|message| EditorRuntimeEventConsumerError::Gateway {
                        consumer_id: manifest.consumer_id.clone(),
                        message: message.to_string(),
                    })?;
            for delivery in deliveries {
                if delivery.play_session_id != runtime_session_id {
                    return Err(EditorRuntimeEventConsumerError::WrongSession {
                        consumer_id: manifest.consumer_id.clone(),
                        expected: runtime_session_id,
                        actual: delivery.play_session_id,
                    });
                }
                if delivery.subscription != consumer.subscription {
                    return Err(EditorRuntimeEventConsumerError::ForeignSubscription {
                        consumer_id: manifest.consumer_id.clone(),
                    });
                }
                if delivery.event_id != manifest.event_id {
                    return Err(EditorRuntimeEventConsumerError::EventMismatch {
                        consumer_id: manifest.consumer_id.clone(),
                        expected: manifest.event_id.clone(),
                        actual: delivery.event_id,
                    });
                }
                if delivery.payload_schema != manifest.payload_schema {
                    return Err(EditorRuntimeEventConsumerError::SchemaMismatch {
                        consumer_id: manifest.consumer_id.clone(),
                        expected: manifest.payload_schema.clone(),
                        actual: delivery.payload_schema,
                    });
                }
                if consumer
                    .last_sequence
                    .is_some_and(|sequence| delivery.sequence <= sequence)
                {
                    return Err(EditorRuntimeEventConsumerError::StaleSequence {
                        consumer_id: manifest.consumer_id.clone(),
                        sequence: delivery.sequence,
                    });
                }
                consumer
                    .registration
                    .consume(play_session_id, delivery.sequence, delivery.payload)
                    .map_err(|source| EditorRuntimeEventConsumerError::Payload {
                        consumer_id: manifest.consumer_id.clone(),
                        source,
                    })?;
                consumer.last_sequence = Some(delivery.sequence);
                applied += 1;
            }
        }
        Ok(applied)
    }

    pub fn end_play_session(
        &self,
        play_session_id: u64,
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        let mut session = self
            .play_session_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *session != Some(play_session_id) {
            return Err(EditorRuntimeEventConsumerError::RuntimeSessionMismatch {
                expected: (*session).unwrap_or_default(),
                actual: play_session_id,
            });
        }
        let gateway = self.gateway.clone();
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let consumer_ids = active.keys().cloned().collect::<Vec<_>>();
        let mut first_error = None;
        for consumer_id in consumer_ids {
            let Some(subscription) = active
                .get(&consumer_id)
                .map(|consumer| consumer.subscription)
            else {
                continue;
            };
            match unsubscribe_consumer(&gateway, &consumer_id, subscription) {
                Ok(()) => {
                    if let Some(consumer) = active.remove(&consumer_id) {
                        consumer.registration.end_session(play_session_id);
                    }
                }
                Err(error) => {
                    first_error.get_or_insert(error);
                }
            }
        }
        if active.is_empty() {
            *session = None;
        }
        first_error.map_or(Ok(()), Err)
    }

    pub fn active_consumer_count(&self) -> usize {
        self.active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    pub fn active_play_session_id(&self) -> Option<u64> {
        *self
            .play_session_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn rollback_added_consumers(
    gateway: &dyn EditorRuntimeGateway,
    active: &mut BTreeMap<String, ActiveConsumer>,
    added: &[String],
    play_session_id: u64,
) -> Option<EditorRuntimeEventConsumerError> {
    let mut first_error = None;
    for consumer_id in added {
        let Some(subscription) = active
            .get(consumer_id)
            .map(|consumer| consumer.subscription)
        else {
            continue;
        };
        match unsubscribe_consumer(gateway, consumer_id, subscription) {
            Ok(()) => {
                if let Some(consumer) = active.remove(consumer_id) {
                    consumer.registration.end_session(play_session_id);
                }
            }
            Err(error) => {
                first_error.get_or_insert(error);
            }
        }
    }
    first_error
}

fn unsubscribe_consumer(
    gateway: &dyn EditorRuntimeGateway,
    consumer_id: &str,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
) -> Result<(), EditorRuntimeEventConsumerError> {
    match gateway.unsubscribe_plugin_event(subscription) {
        Ok(true) => Ok(()),
        Ok(false) => Err(EditorRuntimeEventConsumerError::Gateway {
            consumer_id: consumer_id.to_string(),
            message: "runtime did not remove the plugin event subscription".to_string(),
        }),
        Err(message) => Err(EditorRuntimeEventConsumerError::Gateway {
            consumer_id: consumer_id.to_string(),
            message: message.to_string(),
        }),
    }
}
