use crossbeam_channel::{unbounded, Receiver};

use super::UiEventManager;
use zircon_runtime_interface::ui::event_ui::{UiNotification, UiSubscriptionId};

impl UiEventManager {
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
#[path = "subscription/owned_notification_fanout_tests.rs"]
mod owned_notification_fanout_tests;
