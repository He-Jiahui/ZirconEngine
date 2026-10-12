// 一个调用匹配多个处理器时，每份投递保留相同事件内容，不让后续处理器失去负载。
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
