use super::fixture::{assert_absent, assert_contains, assert_ordered, EventBusSources};

// EventBusSources 从生产文件快照核对公开边界：EventBus::subscribe 接收调用方策略，并返回框架订阅 trait object。
// 主题注册与队列创建留在 EventBusState，旧 receiver/channel 形态不得出现在 facade。
#[test]
fn event_bus_subscribe_binds_an_explicit_delivery_policy_to_state() {
    let sources = EventBusSources::load();

    assert_ordered(
        sources.subscribe,
        &[
            "use crate::core::framework::events::{",
            "EngineEventDeliveryPolicy,",
            "EngineEventSubscription",
            "impl EventBus",
            "pub fn subscribe(",
            "policy: EngineEventDeliveryPolicy",
            "Box<dyn EngineEventSubscription>",
            ".subscribe(topic.into(), policy)",
        ],
    );
    assert_contains(sources.topic, "pub(super) fn subscribe(");
    assert_contains(sources.topic, "EventSubscriber::new(");
    assert_ordered(
        sources.topic,
        &[
            // 先在主题注册表写锁内取得 topic 并增加 pending reservation，随后释放全局锁。
            // 测试钩子可在 reservation 存活时与 prune 交错；添加 subscriber 后释放 reservation，Drop 负责扣回占位计数。
            "let (topic, reservation) = {",
            "let mut topics = self.write_topics();",
            "let reservation = topic.reserve_subscription();",
            "(topic, reservation)",
            "after_reservation();",
            "topic.add_subscriber",
            "drop(reservation);",
            "EventSubscription::new",
        ],
    );
    assert_contains(sources.topic, "pending_subscriptions: AtomicUsize");
    assert_contains(sources.topic, "struct PendingSubscription");
    assert_contains(sources.topic, "impl Drop for PendingSubscription");
    assert_contains(sources.topic, "topic.is_removable()");
    assert_absent(sources.subscribe, "ChannelReceiver<EngineEvent>");
    assert_absent(sources.subscribe, "Entry::Occupied");
}
