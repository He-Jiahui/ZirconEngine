use crossbeam_channel::{unbounded, Receiver};

use super::super::{UiNotification, UiSubscriptionId};
use super::UiEventManager;

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
        for sender in self.subscriptions.values() {
            let _ = sender.send(notification.clone());
        }
    }
}
