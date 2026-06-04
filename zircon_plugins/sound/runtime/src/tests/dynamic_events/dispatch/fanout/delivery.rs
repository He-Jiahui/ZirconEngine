use super::super::super::*;

use super::support::fanout_fixture;

#[test]
fn dynamic_event_dispatch_clones_invocation_to_each_fanout_delivery() {
    let fixture = fanout_fixture();

    let deliveries = fixture.sound.dispatch_dynamic_events().unwrap();

    assert_eq!(deliveries.len(), 3);
    assert!(deliveries
        .iter()
        .all(|delivery| delivery.invocation == fixture.invocation));
}
