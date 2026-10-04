use std::collections::HashSet;

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
        let mut removed_ids = HashSet::with_capacity(removed.len());
        removed_ids.extend(removed.iter().map(String::as_str));
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
#[path = "tests/contribution_lifecycle.rs"]
mod tests;
