use serde_json::Value;

use crate::core::{EngineEvent, EventBus};

use super::super::super::*;

#[test]
fn event_bus_and_config_store_roundtrip() {
    let runtime = CoreRuntime::new();
    let events = runtime.handle().subscribe_events("editor.selection");
    runtime.publish_event("editor.selection", serde_json::json!({ "node": 7 }));
    let event = events.recv().unwrap();
    assert_eq!(event.payload["node"], 7);

    runtime
        .handle()
        .store_config("editor.theme", &serde_json::json!({ "name": "TokyoNight" }))
        .unwrap();
    let theme: Value = runtime.load_config("editor.theme").unwrap();
    assert_eq!(theme["name"], "TokyoNight");
}

#[test]
fn event_bus_prunes_closed_subscribers_after_snapshot_publish() {
    let bus = EventBus::default();
    let closed_events = bus.subscribe("runtime.tick");
    let live_events = bus.subscribe("runtime.tick");
    drop(closed_events);

    bus.publish(EngineEvent {
        topic: "runtime.tick".to_string(),
        payload: serde_json::json!({ "frame": 1 }),
    });
    let event = live_events.recv().unwrap();
    assert_eq!(event.payload["frame"], 1);

    bus.publish(EngineEvent {
        topic: "runtime.tick".to_string(),
        payload: serde_json::json!({ "frame": 2 }),
    });
    let event = live_events.recv().unwrap();
    assert_eq!(event.payload["frame"], 2);
}
