use crate::core::gateway::GatewayOrigin;

use super::execution_support::{invoke_consumer_callback, unsubscribe_consumer};
use super::{
    ActiveConsumer, ActiveConsumerIdentity, EditorRuntimeEventConsumerCallbackPhase,
    EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerHost, QualifiedSubscription,
};

#[derive(Clone)]
pub(super) struct PendingRemoteCleanup {
    pub(super) subscription: QualifiedSubscription,
    pub(super) origin: GatewayOrigin,
}

impl EditorRuntimeEventConsumerHost {
    /// 终止、能力收缩和贡献撤销共用此退役路径；本地释放后仍须结清原端点订阅。
    pub(super) fn retire_active_consumer(
        &self,
        identity: &ActiveConsumerIdentity,
        play_session_id: u64,
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        let Some(consumer) = self.remove_active_consumer(identity) else {
            return Ok(());
        };
        self.release_pending_bytes(consumer.pending_retained_bytes);
        // 退订失败后保留原端点订阅，供后续清理重试。
        let remote_cleanup = unsubscribe_consumer(
            &consumer.origin,
            &identity.consumer_id,
            &identity.subscription,
        )
        .err();
        if remote_cleanup.is_some() {
            self.defer_remote_cleanup(
                &identity.consumer_id,
                identity.subscription.clone(),
                consumer.origin.clone(),
            );
        }
        let callback_cleanup = invoke_consumer_callback(
            &identity.consumer_id,
            EditorRuntimeEventConsumerCallbackPhase::EndSession,
            None,
            || consumer.registration.end_session(play_session_id),
        )
        .err();
        if callback_cleanup.is_some() {
            self.record_callback_fault(
                &identity.consumer_id,
                play_session_id,
                EditorRuntimeEventConsumerCallbackPhase::EndSession,
                None,
                None,
            );
        }
        combine_cleanup_errors(
            "retire runtime event consumer",
            remote_cleanup,
            callback_cleanup,
        )
        .map_or(Ok(()), Err)
    }

    fn remove_active_consumer(&self, identity: &ActiveConsumerIdentity) -> Option<ActiveConsumer> {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let matches = active.get(&identity.consumer_id).is_some_and(|consumer| {
            consumer.generation == identity.generation
                && consumer.subscription == identity.subscription
        });
        matches
            .then(|| active.remove(&identity.consumer_id))
            .flatten()
    }

    pub(super) fn rollback_added_consumers(
        &self,
        added: &[ActiveConsumerIdentity],
        play_session_id: u64,
    ) -> Option<EditorRuntimeEventConsumerError> {
        let mut first_error = None;
        for identity in added {
            if let Err(error) = self.retire_active_consumer(identity, play_session_id) {
                first_error.get_or_insert(error);
            }
        }
        first_error
    }

    /// 失败的远端清理保留原端点；同一消费者的不同订阅分别保留，重复登记同一订阅不会增加待办。
    pub(super) fn defer_remote_cleanup(
        &self,
        consumer_id: &str,
        subscription: QualifiedSubscription,
        origin: GatewayOrigin,
    ) {
        let mut pending = self
            .pending_remote_cleanup
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let cleanups = pending.entry(consumer_id.to_string()).or_default();
        if !cleanups
            .iter()
            .any(|cleanup| cleanup.subscription == subscription)
        {
            cleanups.push(PendingRemoteCleanup {
                subscription,
                origin,
            });
        }
    }

    pub(super) fn retry_pending_remote_cleanup(&self) {
        let pending = self
            .pending_remote_cleanup
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .flat_map(|(consumer_id, cleanups)| {
                cleanups
                    .iter()
                    .cloned()
                    .map(move |cleanup| (consumer_id.clone(), cleanup))
            })
            .collect::<Vec<_>>();
        for (consumer_id, cleanup) in pending {
            if unsubscribe_consumer(&cleanup.origin, &consumer_id, &cleanup.subscription).is_ok() {
                let mut pending = self
                    .pending_remote_cleanup
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if let Some(cleanups) = pending.get_mut(&consumer_id) {
                    cleanups.retain(|entry| entry.subscription != cleanup.subscription);
                    if cleanups.is_empty() {
                        pending.remove(&consumer_id);
                    }
                }
            }
        }
    }

    pub(super) fn flush_pending_remote_cleanup(&self) -> Option<EditorRuntimeEventConsumerError> {
        let pending = std::mem::take(
            &mut *self
                .pending_remote_cleanup
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        );
        pending
            .into_iter()
            .fold(None, |mut first_error, (consumer_id, cleanups)| {
                for cleanup in cleanups {
                    if let Err(error) =
                        unsubscribe_consumer(&cleanup.origin, &consumer_id, &cleanup.subscription)
                    {
                        self.defer_remote_cleanup(
                            &consumer_id,
                            cleanup.subscription,
                            cleanup.origin,
                        );
                        first_error.get_or_insert(error);
                    }
                }
                first_error
            })
    }
}

pub(super) fn combine_cleanup_errors(
    operation: &'static str,
    first: Option<EditorRuntimeEventConsumerError>,
    second: Option<EditorRuntimeEventConsumerError>,
) -> Option<EditorRuntimeEventConsumerError> {
    match (first, second) {
        (Some(first), Some(second)) => Some(EditorRuntimeEventConsumerError::with_cleanup(
            operation, first, second,
        )),
        (Some(error), None) | (None, Some(error)) => Some(error),
        (None, None) => None,
    }
}
