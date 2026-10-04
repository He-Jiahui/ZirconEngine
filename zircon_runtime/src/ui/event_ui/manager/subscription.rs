use crossbeam_channel::{unbounded, Receiver};

use super::UiEventManager;
use zircon_runtime_interface::ui::event_ui::{UiNotification, UiSubscriptionId};

impl UiEventManager {
    /// 订阅者须保存并持续消费 Receiver，结束时用编号退订；通知通道无容量上限，不能只持有编号。
    pub fn subscribe(&mut self) -> (UiSubscriptionId, Receiver<UiNotification>) {
        self.next_subscription_id += 1;
        let subscription_id = UiSubscriptionId::new(self.next_subscription_id);
        let (tx, rx) = unbounded();
        self.subscriptions.insert(subscription_id, tx);
        (subscription_id, rx)
    }

    pub fn unsubscribe(&mut self, subscription_id: UiSubscriptionId) -> bool {
        self.subscriptions.remove(&subscription_id).is_some()
    }

    pub(crate) fn broadcast(&self, notification: UiNotification) {
        // 最后一位订阅者接收原载荷，其余订阅者各需独立副本；单订阅路径不复制大型反射/调用结果。
        let mut senders = self.subscriptions.values();
        let Some(final_sender) = senders.next_back() else {
            return;
        };
        for sender in senders {
            let _ = sender.send(notification.clone());
        }
        let _ = final_sender.send(notification);
    }
}

#[cfg(test)]
#[path = "subscription/tests/owned_notification_fanout_tests.rs"]
mod owned_notification_fanout_tests;
