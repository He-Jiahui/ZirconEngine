use crate::core::framework::events::*;
use crate::core::runtime::EventBus;

#[test]
fn optimization_batch_jm_runtime652_fanout_admission_keeps_snapshot_bound() {
    let bus = EventBus::default();
    let subscribers: Vec<_> = (0..256)
        .map(|_| {
            bus.subscribe(
                "runtime.fanout",
                EngineEventDeliveryPolicy::Reliable {
                    limits: EventRetentionLimits::default(),
                },
            )
            .unwrap()
        })
        .collect();
    let receipt = bus
        .try_publish(EngineEvent {
            topic: "runtime.fanout".into(),
            payload: serde_json::json!({"value":1}),
        })
        .unwrap();
    assert_eq!(receipt.subscribers, subscribers.len());
    assert_eq!(bus.retention_snapshot().retained_events, subscribers.len());
    for sub in &subscribers {
        assert_eq!(sub.recv().unwrap().decode_payload().unwrap()["value"], 1);
    }
    assert_eq!(bus.retention_snapshot().retained_events, 0);
}
