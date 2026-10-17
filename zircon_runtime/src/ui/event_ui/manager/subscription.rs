use crossbeam_channel::{unbounded, Receiver};

use super::UiEventManager;
use zircon_runtime_interface::ui::event_ui::{UiNotification, UiSubscriptionId};

impl UiEventManager {
    /// 订阅者须保存并持续消费 Receiver，结束时用编号退订；通知通道无容量上限，不能只持有编号。
    ///
    /// # 返回与生命周期
    /// 返回新编号及拥有的接收端；管理器只保存发送端，退订按编号移除该发送端。
    /// 每次广播携带反射差异或调用结果，接收端是否仍连接由调用者维护。
    pub fn subscribe(&mut self) -> (UiSubscriptionId, Receiver<UiNotification>) {
        self.next_subscription_id += 1;
        let subscription_id = UiSubscriptionId::new(self.next_subscription_id);
        let (tx, rx) = unbounded();
        self.subscriptions.insert(subscription_id, tx);
        (subscription_id, rx)
    }

    /// 按订阅编号移除发送端，返回该编号此前是否仍在表中。
    ///
    /// 退订不消费调用者的接收端；其持有及释放仍由调用者负责。
    pub fn unsubscribe(&mut self, subscription_id: UiSubscriptionId) -> bool {
        self.subscriptions.remove(&subscription_id).is_some()
    }

    /// 将拥有的通知交给当前订阅发送端；无订阅时直接释放载荷。
    ///
    /// 发送失败在此忽略，不清理断开的订阅；多订阅的克隆与末次移交顺序见下方循环。
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
#[path = "tests/owned_notification_fanout.rs"]
mod owned_notification_fanout_tests;
