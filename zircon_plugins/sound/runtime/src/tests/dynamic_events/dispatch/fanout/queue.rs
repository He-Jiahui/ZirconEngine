use super::super::super::*;

use super::support::fanout_fixture;

#[test]
fn dynamic_event_dispatch_drains_fanout_queue_after_delivery() {
    let fixture = fanout_fixture();

    assert_eq!(fixture.sound.dispatch_dynamic_events().unwrap().len(), 3);
    assert!(fixture.sound.dispatch_dynamic_events().unwrap().is_empty());
    assert!(fixture.sound.drain_dynamic_events().unwrap().is_empty());
}
