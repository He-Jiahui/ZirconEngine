use zircon_runtime::core::framework::navigation::{NavAgentTickReport, NavigationDebugCapture};
use zircon_runtime::scene::World;

use crate::{
    plugin_registration, NavigationOverlayFrame, NAVIGATION_OVERLAY_FRAME_EVENT_ID,
    NAVIGATION_OVERLAY_FRAME_PAYLOAD_SCHEMA,
};

#[test]
fn navigation_overlay_mirror_controls_debug_capture_and_delivers_frame() {
    let mut report = plugin_registration();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    let mut world = World::empty();
    report.extensions.apply_to_world(&mut world).unwrap();
    assert!(!world.resource::<NavigationDebugCapture>().enabled);

    let mut subscription = world
        .subscribe_runtime_event_mirror(
            NAVIGATION_OVERLAY_FRAME_EVENT_ID,
            NAVIGATION_OVERLAY_FRAME_PAYLOAD_SCHEMA,
        )
        .unwrap();
    assert!(world.resource::<NavigationDebugCapture>().enabled);

    world.send_event(NavigationOverlayFrame {
        owner_generation: 7,
        tick_report: NavAgentTickReport {
            scanned_agents: 4,
            moved_agents: 3,
            ..NavAgentTickReport::default()
        },
        ..NavigationOverlayFrame::default()
    });
    world.update_events::<NavigationOverlayFrame>();
    let payloads = world.drain_runtime_event_mirror(&mut subscription).unwrap();
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0]["owner_generation"], 7);
    assert_eq!(payloads[0]["tick_report"]["moved_agents"], 3);

    assert!(world
        .unsubscribe_runtime_event_mirror(&mut subscription)
        .unwrap());
    assert!(!world.resource::<NavigationDebugCapture>().enabled);
}
