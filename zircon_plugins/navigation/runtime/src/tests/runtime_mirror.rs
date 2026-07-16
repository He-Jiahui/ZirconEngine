use zircon_runtime::core::framework::navigation::{NavAgentTickReport, NavigationDebugCapture};
use zircon_runtime::scene::World;

use crate::plugin_registration;

const EVENT_ID: &str = "navigation.events.agent_tick_completed";
const PAYLOAD_SCHEMA: &str = "navigation.events.nav_agent_tick_report.v1";

#[test]
fn navigation_tick_mirror_controls_debug_capture_and_delivers_ecs_payload() {
    let mut report = plugin_registration();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    let mut world = World::empty();
    report.extensions.apply_to_world(&mut world).unwrap();
    assert!(!world.resource::<NavigationDebugCapture>().enabled);

    let mut subscription = world
        .subscribe_runtime_event_mirror(EVENT_ID, PAYLOAD_SCHEMA)
        .unwrap();
    assert!(world.resource::<NavigationDebugCapture>().enabled);

    let tick = NavAgentTickReport {
        scanned_agents: 4,
        moved_agents: 3,
        ..NavAgentTickReport::default()
    };
    world.send_event(tick);
    world.update_events::<NavAgentTickReport>();
    let payloads = world.drain_runtime_event_mirror(&mut subscription).unwrap();
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0]["scanned_agents"], 4);
    assert_eq!(payloads[0]["moved_agents"], 3);

    assert!(world
        .unsubscribe_runtime_event_mirror(&mut subscription)
        .unwrap());
    assert!(!world.resource::<NavigationDebugCapture>().enabled);
}
