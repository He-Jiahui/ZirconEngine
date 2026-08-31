use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;
use std::{panic::catch_unwind, panic::AssertUnwindSafe};

use zircon_runtime_interface::ZrRuntimePluginEventDeliveryV1;

use crate::core::gateway::GatewayOrigin;

use super::{
    ActiveConsumerSnapshot, EditorRuntimeEventConsumerCallbackPhase,
    EditorRuntimeEventConsumerError, QualifiedSubscription, EXECUTION_IDLE, EXECUTION_LIFECYCLE,
    EXECUTION_PUMP,
};

pub(super) fn invoke_consumer_callback<T>(
    consumer_id: &str,
    phase: EditorRuntimeEventConsumerCallbackPhase,
    delivery_sequence: Option<u64>,
    callback: impl FnOnce() -> T,
) -> Result<T, EditorRuntimeEventConsumerError> {
    catch_unwind(AssertUnwindSafe(callback)).map_err(|_| {
        EditorRuntimeEventConsumerError::callback_panicked(consumer_id, phase, delivery_sequence)
    })
}

pub(super) fn p95_duration(samples: &mut [Duration]) -> Duration {
    if samples.is_empty() {
        return Duration::ZERO;
    }
    samples.sort_unstable();
    let index = samples.len().saturating_mul(95).div_ceil(100) - 1;
    samples[index]
}

pub(super) struct PumpExecutionGuard<'a> {
    execution_state: &'a AtomicU8,
}

impl<'a> PumpExecutionGuard<'a> {
    pub(super) fn enter(execution_state: &'a AtomicU8) -> Option<Self> {
        execution_state
            .compare_exchange(
                EXECUTION_IDLE,
                EXECUTION_PUMP,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .ok()
            .map(|_| Self { execution_state })
    }
}

impl Drop for PumpExecutionGuard<'_> {
    fn drop(&mut self) {
        self.execution_state
            .store(EXECUTION_IDLE, Ordering::Release);
    }
}

pub(super) struct LifecycleExecutionGuard<'a> {
    execution_state: &'a AtomicU8,
}

impl<'a> LifecycleExecutionGuard<'a> {
    pub(super) fn enter(
        execution_state: &'a AtomicU8,
        operation: &'static str,
    ) -> Result<Self, EditorRuntimeEventConsumerError> {
        execution_state
            .compare_exchange(
                EXECUTION_IDLE,
                EXECUTION_LIFECYCLE,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .map(|_| Self { execution_state })
            .map_err(|_| EditorRuntimeEventConsumerError::LifecycleMutationBusy { operation })
    }
}

impl Drop for LifecycleExecutionGuard<'_> {
    fn drop(&mut self) {
        self.execution_state
            .store(EXECUTION_IDLE, Ordering::Release);
    }
}

pub(super) fn validate_delivery(
    snapshot: &ActiveConsumerSnapshot,
    runtime_session_id: u64,
    last_sequence: Option<u64>,
    delivery: &ZrRuntimePluginEventDeliveryV1,
) -> Result<(), EditorRuntimeEventConsumerError> {
    let manifest = snapshot.registration.manifest();
    if delivery.play_session_id != runtime_session_id {
        return Err(EditorRuntimeEventConsumerError::WrongSession {
            consumer_id: snapshot.consumer_id.clone(),
            expected: runtime_session_id,
            actual: delivery.play_session_id,
        });
    }
    if delivery.subscription != snapshot.subscription.raw() {
        return Err(EditorRuntimeEventConsumerError::ForeignSubscription {
            consumer_id: snapshot.consumer_id.clone(),
        });
    }
    if delivery.event_id != manifest.event_id {
        return Err(EditorRuntimeEventConsumerError::EventMismatch {
            consumer_id: snapshot.consumer_id.clone(),
            expected: manifest.event_id.clone(),
            actual: delivery.event_id.clone(),
        });
    }
    if delivery.payload_schema != manifest.payload_schema {
        return Err(EditorRuntimeEventConsumerError::SchemaMismatch {
            consumer_id: snapshot.consumer_id.clone(),
            expected: manifest.payload_schema.clone(),
            actual: delivery.payload_schema.clone(),
        });
    }
    if last_sequence.is_some_and(|sequence| delivery.sequence <= sequence) {
        return Err(EditorRuntimeEventConsumerError::StaleSequence {
            consumer_id: snapshot.consumer_id.clone(),
            sequence: delivery.sequence,
        });
    }
    Ok(())
}

pub(super) fn unsubscribe_consumer(
    origin: &GatewayOrigin,
    consumer_id: &str,
    subscription: &QualifiedSubscription,
) -> Result<(), EditorRuntimeEventConsumerError> {
    if !subscription.belongs_to(origin) {
        return Err(EditorRuntimeEventConsumerError::Gateway {
            consumer_id: consumer_id.to_string(),
            message: "plugin event subscription belongs to a different gateway identity"
                .to_string(),
        });
    }
    match origin
        .gateway()
        .unsubscribe_plugin_event(subscription.raw())
    {
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
