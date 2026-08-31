use std::collections::BTreeSet;

use crate::core::extension::{ContributionSource, ContributionTicket};

use super::execution_support::LifecycleExecutionGuard;
use super::{
    ActiveConsumerIdentity, EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerHost,
    EditorRuntimeEventConsumerRegistry,
};

#[derive(Debug)]
pub(crate) struct ContributionRetirementReport {
    pub(crate) removed: Vec<String>,
    pub(crate) cleanup_error: Option<EditorRuntimeEventConsumerError>,
}

impl EditorRuntimeEventConsumerHost {
    pub(crate) fn prepare_contribution_registration(
        &self,
        ticket: ContributionTicket,
        source: ContributionSource,
        registry: EditorRuntimeEventConsumerRegistry,
    ) -> Result<EditorRuntimeEventConsumerRegistry, EditorRuntimeEventConsumerError> {
        let mut candidate = self
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        candidate.extend_contribution(ticket, source, registry)?;
        Ok(candidate)
    }

    pub(crate) fn retire_contribution(
        &self,
        ticket: ContributionTicket,
    ) -> Result<ContributionRetirementReport, EditorRuntimeEventConsumerError> {
        let _lifecycle_guard = LifecycleExecutionGuard::enter(
            &self.execution_state,
            "retire contributed runtime event consumers",
        )?;
        let (registry_candidate, removed) = self
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .without_contribution(ticket);
        if removed.is_empty() {
            return Ok(ContributionRetirementReport {
                removed,
                cleanup_error: None,
            });
        }

        let active_identities = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .filter(|(_, consumer)| consumer.registration.contribution_ticket() == Some(ticket))
            .map(|(consumer_id, consumer)| ActiveConsumerIdentity {
                consumer_id: consumer_id.clone(),
                subscription: consumer.subscription.clone(),
                generation: consumer.generation,
            })
            .collect::<Vec<_>>();
        let play_session_id = *self
            .play_session_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut first_error = (!active_identities.is_empty() && play_session_id.is_none())
            .then_some(EditorRuntimeEventConsumerError::NoActiveSession);
        let callback_session_id = play_session_id.unwrap_or_default();
        for identity in active_identities {
            if let Err(error) = self.retire_active_consumer(&identity, callback_session_id) {
                first_error.get_or_insert(error);
            }
        }

        *self
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = registry_candidate;
        let removed_ids = removed.iter().map(String::as_str).collect::<BTreeSet<_>>();
        self.quarantined_consumers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retain(|consumer_id, _| !removed_ids.contains(consumer_id.as_str()));
        self.user_disabled_consumers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retain(|consumer_id| !removed_ids.contains(consumer_id.as_str()));
        let mut round_robin_cursor = self
            .round_robin_cursor
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if round_robin_cursor
            .as_ref()
            .is_some_and(|consumer_id| removed_ids.contains(consumer_id.as_str()))
        {
            *round_robin_cursor = None;
        }
        drop(round_robin_cursor);

        Ok(ContributionRetirementReport {
            removed,
            cleanup_error: first_error,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::convert::Infallible;
    use std::sync::{Arc, Mutex};

    use zircon_runtime::plugin::PluginEventConsumerManifest;
    use zircon_runtime_interface::ZrRuntimePluginEventSubscriptionHandle;

    use crate::core::extension::{
        ContributionBatch, ContributionSource, ContributionStore, ContributionTicket,
        PluginContributionId,
    };
    use crate::core::gateway::EditorRuntimeGatewayHandle;
    use crate::core::runtime_event_consumer::{
        EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerRegistration,
        EditorRuntimeEventConsumerRegistry, EditorRuntimeEventConsumerState,
    };

    use super::super::execution_support::LifecycleExecutionGuard;
    use super::super::health::ConsumerCallbackHealth;
    use super::super::{ActiveConsumer, QualifiedSubscription};

    #[derive(Default)]
    struct ConsumerState {
        ended_session: Option<u64>,
    }

    impl EditorRuntimeEventConsumerState for ConsumerState {
        type Payload = ();
        type Error = Infallible;

        fn begin_session(&mut self, _play_session_id: u64) {}

        fn consume(
            &mut self,
            _play_session_id: u64,
            _sequence: u64,
            _payload: Self::Payload,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn end_session(&mut self, play_session_id: u64) {
            self.ended_session = Some(play_session_id);
        }
    }

    fn registration(
        consumer_id: &str,
    ) -> (
        EditorRuntimeEventConsumerRegistration,
        Arc<Mutex<ConsumerState>>,
    ) {
        let state = Arc::new(Mutex::new(ConsumerState::default()));
        (
            EditorRuntimeEventConsumerRegistration::typed(
                PluginEventConsumerManifest::new(
                    consumer_id,
                    format!("{consumer_id}.event"),
                    format!("{consumer_id}.event.v1"),
                ),
                Arc::clone(&state),
            ),
            state,
        )
    }

    fn plugin_ticket(
        store: &mut ContributionStore,
        plugin_id: &str,
    ) -> (ContributionTicket, ContributionSource) {
        let source = ContributionSource::Plugin(
            PluginContributionId::parse(plugin_id).expect("plugin id should be valid"),
        );
        let ticket = store
            .contribute(source.clone(), ContributionBatch::default())
            .expect("empty ownership batch should allocate a ticket");
        (ticket, source)
    }

    fn registry_with(consumer_id: &str) -> EditorRuntimeEventConsumerRegistry {
        let mut registry = EditorRuntimeEventConsumerRegistry::default();
        registry.register(registration(consumer_id).0).unwrap();
        registry
    }

    #[test]
    fn host_ticket_retirement_preserves_builtin_and_other_contribution() {
        let host = super::super::EditorRuntimeEventConsumerHost::default();
        let mut store = ContributionStore::default();
        let (weather_ticket, weather_source) = plugin_ticket(&mut store, "weather");
        let (lighting_ticket, lighting_source) = plugin_ticket(&mut store, "lighting");
        host.register(registry_with("builtin.console")).unwrap();
        let candidate = host
            .prepare_contribution_registration(
                weather_ticket,
                weather_source,
                registry_with("weather.clouds"),
            )
            .unwrap();
        host.install_prepared_registration(candidate);
        let candidate = host
            .prepare_contribution_registration(
                lighting_ticket,
                lighting_source,
                registry_with("lighting.exposure"),
            )
            .unwrap();
        host.install_prepared_registration(candidate);

        let removed = host.retire_contribution(weather_ticket).unwrap();

        assert_eq!(removed.removed, ["weather.clouds"]);
        let registry = host
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(registry.registration("weather.clouds").is_none());
        assert!(registry.registration("builtin.console").is_some());
        assert!(registry.registration("lighting.exposure").is_some());
    }

    #[test]
    fn cleanup_error_still_retires_active_generation_and_registry_owner() {
        let gateway = EditorRuntimeGatewayHandle::detached();
        let host = super::super::EditorRuntimeEventConsumerHost::new(gateway.clone());
        let mut store = ContributionStore::default();
        let (ticket, source) = plugin_ticket(&mut store, "weather");
        let (registration, state) = registration("weather.clouds");
        let mut contributed = EditorRuntimeEventConsumerRegistry::default();
        contributed.register(registration).unwrap();
        let candidate = host
            .prepare_contribution_registration(ticket, source, contributed)
            .unwrap();
        let owned_registration = candidate
            .registration("weather.clouds")
            .expect("candidate should retain its registration")
            .clone();
        host.install_prepared_registration(candidate);
        let origin = gateway.current_lease().origin();
        let subscription = QualifiedSubscription::new(
            ZrRuntimePluginEventSubscriptionHandle::new(17),
            origin.identity().clone(),
        );
        host.active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(
                "weather.clouds".to_string(),
                ActiveConsumer {
                    registration: owned_registration,
                    origin,
                    health: ConsumerCallbackHealth::default(),
                    subscription,
                    generation: 1,
                    last_sequence: None,
                    pending: VecDeque::new(),
                    pending_retained_bytes: 0,
                    last_observed_runtime_remaining_deliveries: None,
                    last_observed_runtime_oldest_pending_age_millis: None,
                    runtime_backlog_observed_at: None,
                },
            );
        *host
            .play_session_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(41);

        let report = host
            .retire_contribution(ticket)
            .expect("cleanup errors are reported after local publication");
        assert!(report.cleanup_error.is_some());

        assert_eq!(host.active_consumer_count(), 0);
        assert!(host
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .registration("weather.clouds")
            .is_none());
        assert_eq!(
            state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .ended_session,
            Some(41)
        );
    }

    #[test]
    fn busy_lifecycle_rejects_before_registry_publication() {
        let host = super::super::EditorRuntimeEventConsumerHost::default();
        let mut store = ContributionStore::default();
        let (ticket, source) = plugin_ticket(&mut store, "weather");
        let candidate = host
            .prepare_contribution_registration(ticket, source, registry_with("weather.clouds"))
            .unwrap();
        host.install_prepared_registration(candidate);
        let _busy = LifecycleExecutionGuard::enter(&host.execution_state, "test owner").unwrap();

        let error = host
            .retire_contribution(ticket)
            .expect_err("busy lifecycle must reject before local publication");

        assert!(matches!(
            error,
            EditorRuntimeEventConsumerError::LifecycleMutationBusy { .. }
        ));
        assert!(host
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .registration("weather.clouds")
            .is_some());
    }
}
