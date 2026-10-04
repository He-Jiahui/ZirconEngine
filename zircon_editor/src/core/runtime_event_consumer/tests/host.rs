use super::{next_consumer_generation, EditorRuntimeEventConsumerHost, QualifiedSubscription};
use crate::core::gateway::EditorRuntimeGatewayHandle;
use crate::core::runtime_event_consumer::EditorRuntimeEventConsumerError;
use zircon_runtime_interface::ZrRuntimePluginEventSubscriptionHandle;

#[test]
fn consumer_generation_exhaustion_is_typed() {
    assert!(matches!(
        next_consumer_generation(u64::MAX),
        Err(EditorRuntimeEventConsumerError::ConsumerGenerationExhausted)
    ));
}

#[test]
fn shutdown_flushes_pending_remote_cleanup_without_an_active_play_session() {
    let gateway = EditorRuntimeGatewayHandle::detached();
    let host = EditorRuntimeEventConsumerHost::new(gateway.clone());
    let origin = gateway.current_lease().origin();
    host.defer_remote_cleanup(
        "deferred.consumer",
        QualifiedSubscription::new(
            ZrRuntimePluginEventSubscriptionHandle::new(11),
            origin.identity().clone(),
        ),
        origin,
    );

    assert_eq!(host.pending_remote_cleanup_count(), 1);
    assert!(host.shutdown().is_err());
    assert_eq!(host.pending_remote_cleanup_count(), 0);
}
