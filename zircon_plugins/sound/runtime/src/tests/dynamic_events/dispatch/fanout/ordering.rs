// 固定一次事件扇出时处理器按优先级与身份排序，供后续执行报告保持可预测顺序。
use super::super::super::*;

use super::support::fanout_fixture;

#[test]
fn dynamic_event_dispatch_orders_fanout_handlers_deterministically() {
    let fixture = fanout_fixture();

    let deliveries = fixture.sound.dispatch_dynamic_events().unwrap();

    assert_eq!(deliveries.len(), 3);
    assert_eq!(deliveries[0].handler.plugin_id, "analytics");
    assert_eq!(deliveries[0].handler.handler_id, "combat-counter");
    assert_eq!(deliveries[1].handler.plugin_id, "gameplay_audio");
    assert_eq!(deliveries[1].handler.handler_id, "weapon-foley");
    assert_eq!(deliveries[2].handler.plugin_id, "timeline_sequence");
    assert_eq!(deliveries[2].handler.handler_id, "timeline-marker");
}
