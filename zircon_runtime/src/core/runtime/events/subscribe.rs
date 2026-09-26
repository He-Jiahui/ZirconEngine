use crate::core::framework::events::{EngineEventDeliveryPolicy, EngineEventSubscription};

use super::EventBus;

impl EventBus {
    /// 为一个主题建立独立接收队列；调用方须保留返回值，Drop 会取消订阅。
    /// 投递策略决定慢消费者遇到积压时保留哪些事件。
    pub fn subscribe(
        &self,
        topic: impl Into<String>,
        policy: EngineEventDeliveryPolicy,
    ) -> Box<dyn EngineEventSubscription> {
        Box::new(self.state.subscribe(topic.into(), policy))
    }

    #[cfg(test)]
    pub(crate) fn subscribe_after_reservation_for_test(
        &self,
        topic: impl Into<String>,
        policy: EngineEventDeliveryPolicy,
        after_reservation: impl FnOnce(),
    ) -> Box<dyn EngineEventSubscription> {
        Box::new(self.state.subscribe_after_reservation_for_test(
            topic.into(),
            policy,
            after_reservation,
        ))
    }

    #[cfg(test)]
    pub(crate) fn hold_topic_delivery_for_test(&self, topic: &str, while_locked: impl FnOnce()) {
        let topic = self
            .state
            .topic(topic)
            .expect("test topic must exist before its delivery lock is held");
        let _delivery = topic.lock_delivery();
        while_locked();
    }
}
