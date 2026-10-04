use crate::core::framework::events::{
    EngineEvent, EngineEventDeliveryPolicy, EventBusDiagnosticsMode,
};

use super::super::EventBus;

#[test]
fn unsubscribe_preserves_queue_diagnostics_after_lock_scope_split() {
    let bus = EventBus::new(EventBusDiagnosticsMode::Enabled);
    let subscription = bus
        .subscribe(
            "runtime.unsubscribe-lock-scope",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
    for sequence in 0..3 {
        bus.try_publish(EngineEvent {
            topic: "runtime.unsubscribe-lock-scope".to_string(),
            payload: serde_json::json!({ "sequence": sequence }),
        })
        .expect("event admission");
    }
    assert_eq!(bus.diagnostic_report().queued, 3);

    drop(subscription);

    let report = bus.diagnostic_report();
    assert_eq!(report.queued, 0);
    assert_eq!(report.disconnected, 1);
    assert_eq!(report.queue_age_samples, 3);
}

#[test]
fn unsubscribe_drains_diagnostics_after_releasing_delivery_lock() {
    let source = include_str!("../prune.rs");
    let lock_scope = source
        .split_once("pub(super) fn unsubscribe")
        .and_then(|(_, body)| body.split_once("#[cfg(test)]"))
        .map(|(body, _)| body)
        .expect("unsubscribe implementation");
    let release = lock_scope
        .find("drop(_delivery)")
        .expect("delivery lock release");
    let accounting = lock_scope
        .find("record_deactivated_queue(queued)")
        .expect("deactivated queue accounting");
    assert!(release < accounting);
}
