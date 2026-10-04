use zircon_runtime::core::framework::sound::{
    SoundDynamicEventCatalog, SoundDynamicEventDescriptor, SoundDynamicEventHandlerDescriptor,
};

use super::{register_dynamic_event_handler, DynamicEventHandlerRegistry};

#[test]
fn handler_registration_maintains_dispatch_order_after_insert_and_update() {
    let catalog = SoundDynamicEventCatalog {
        namespace: "benchmark".to_string(),
        version: 1,
        events: vec![SoundDynamicEventDescriptor {
            id: "weapon.fire".to_string(),
            display_name: "Weapon Fire".to_string(),
            payload_schema: "weapon/v1".to_string(),
        }],
    };
    let mut handlers = DynamicEventHandlerRegistry::default();
    register_dynamic_event_handler(&catalog, &mut handlers, handler("timeline", "marker", 10))
        .unwrap();
    register_dynamic_event_handler(&catalog, &mut handlers, handler("gameplay", "foley", 20))
        .unwrap();
    register_dynamic_event_handler(&catalog, &mut handlers, handler("analytics", "counter", 20))
        .unwrap();

    assert_eq!(
        handler_keys(handlers.handlers()),
        ["analytics/counter", "gameplay/foley", "timeline/marker"]
    );

    register_dynamic_event_handler(&catalog, &mut handlers, handler("timeline", "marker", 30))
        .unwrap();
    assert_eq!(
        handler_keys(handlers.handlers()),
        ["timeline/marker", "analytics/counter", "gameplay/foley"]
    );
    assert_eq!(
        handlers.indices_for_event("weapon.fire").unwrap(),
        [0, 1, 2]
    );
}

#[test]
fn handler_retain_rebuilds_event_indices() {
    let mut handlers = DynamicEventHandlerRegistry::from_handlers(vec![
        event_handler("weapon.fire", "audio", "foley", 20),
        event_handler("music.stop", "music", "fade", 10),
        event_handler("weapon.fire", "telemetry", "count", 10),
    ]);

    handlers.retain(|handler| handler.event_id != "music.stop");

    assert_eq!(handlers.indices_for_event("weapon.fire").unwrap(), [0, 1]);
    assert!(handlers.indices_for_event("music.stop").is_none());
    assert_eq!(
        handler_keys(handlers.handlers()),
        ["audio/foley", "telemetry/count"]
    );
}

fn handler(plugin_id: &str, handler_id: &str, priority: i32) -> SoundDynamicEventHandlerDescriptor {
    event_handler("weapon.fire", plugin_id, handler_id, priority)
}

fn event_handler(
    event_id: &str,
    plugin_id: &str,
    handler_id: &str,
    priority: i32,
) -> SoundDynamicEventHandlerDescriptor {
    SoundDynamicEventHandlerDescriptor {
        plugin_id: plugin_id.to_string(),
        handler_id: handler_id.to_string(),
        event_id: event_id.to_string(),
        display_name: handler_id.to_string(),
        priority,
    }
}

fn handler_keys(handlers: &[SoundDynamicEventHandlerDescriptor]) -> Vec<String> {
    handlers
        .iter()
        .map(|handler| format!("{}/{}", handler.plugin_id, handler.handler_id))
        .collect()
}
