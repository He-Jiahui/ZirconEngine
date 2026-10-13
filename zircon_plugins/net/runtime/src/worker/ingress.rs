//! 将 worker 生命周期 NetEvent 送入 manager 事件队列；消息枚举为后续 ingress 扩展保留边界。
//! 数据 payload 仍通过 TCP/UDP 显式 poll 返回，不经过该通道。

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use zircon_runtime::core::framework::net::NetEvent;

pub(crate) const WORKER_INGRESS_CAPACITY: usize = 4_096;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NetIngress {
    Event(NetEvent),
}

impl NetIngress {
    pub(crate) fn into_event(self) -> NetEvent {
        match self {
            Self::Event(event) => event,
        }
    }
}

/// Bounded lifecycle-event queue shared by the worker producer and manager poller.
/// Cancellation wakes blocked producers during worker shutdown; already queued events remain drainable.
#[derive(Debug)]
pub(crate) struct NetWorkerIngress {
    queue: Mutex<VecDeque<NetIngress>>,
    capacity: usize,
    cancelled: AtomicBool,
    space_available: Condvar,
}

impl NetWorkerIngress {
    pub(crate) fn new(capacity: usize) -> Arc<Self> {
        assert!(capacity > 0, "worker ingress capacity must be non-zero");
        Arc::new(Self {
            queue: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
            cancelled: AtomicBool::new(false),
            space_available: Condvar::new(),
        })
    }

    pub(crate) fn push(&self, ingress: NetIngress) -> bool {
        let mut queue = self
            .queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while queue.len() >= self.capacity && !self.cancelled.load(Ordering::Acquire) {
            queue = self
                .space_available
                .wait(queue)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        if self.cancelled.load(Ordering::Acquire) {
            return false;
        }
        queue.push_back(ingress);
        true
    }

    pub(crate) fn drain(&self, max_events: usize) -> Vec<NetEvent> {
        let mut queue = self
            .queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let count = max_events.min(queue.len());
        let events = queue.drain(..count).map(NetIngress::into_event).collect();
        drop(queue);
        if count != 0 {
            self.space_available.notify_all();
        }
        events
    }

    pub(crate) fn len(&self) -> usize {
        self.queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    pub(crate) fn cancel(&self) {
        let _queue = self
            .queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.cancelled.store(true, Ordering::Release);
        self.space_available.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::{NetIngress, NetWorkerIngress};
    use std::sync::{mpsc, Arc};
    use std::thread;
    use std::time::Duration;
    use zircon_runtime::core::framework::net::{NetEvent, NetSocketId};

    fn event(id: u64) -> NetIngress {
        NetIngress::Event(NetEvent::UdpSocketClosed {
            socket: NetSocketId::new(id),
        })
    }

    #[test]
    fn ingress_flood_backpressures_at_capacity_and_cancel_wakes_producer() {
        let ingress = NetWorkerIngress::new(2);
        assert!(ingress.push(event(1)));
        assert!(ingress.push(event(2)));
        assert_eq!(ingress.len(), 2);

        let (started_tx, started_rx) = mpsc::channel();
        let producer_ingress = Arc::clone(&ingress);
        let producer = thread::spawn(move || {
            started_tx.send(()).unwrap();
            producer_ingress.push(event(3))
        });
        started_rx.recv().unwrap();
        thread::sleep(Duration::from_millis(10));
        assert!(
            !producer.is_finished(),
            "producer must wait at the bounded capacity"
        );

        ingress.cancel();
        assert!(
            !producer.join().unwrap(),
            "cancel must wake and reject the blocked event"
        );
        assert_eq!(
            ingress.len(),
            2,
            "cancellation must preserve queued lifecycle events"
        );
        assert_eq!(ingress.drain(2).len(), 2);
        assert_eq!(ingress.len(), 0);
    }

    #[test]
    fn draining_ingress_releases_blocked_lifecycle_producer() {
        let ingress = NetWorkerIngress::new(1);
        assert!(ingress.push(event(1)));
        let (started_tx, started_rx) = mpsc::channel();
        let producer_ingress = Arc::clone(&ingress);
        let producer = thread::spawn(move || {
            started_tx.send(()).unwrap();
            producer_ingress.push(event(2))
        });
        started_rx.recv().unwrap();
        thread::sleep(Duration::from_millis(10));
        assert!(
            !producer.is_finished(),
            "producer must wait until a slot is drained"
        );
        assert_eq!(ingress.drain(1).len(), 1);
        assert!(producer.join().unwrap());
        assert_eq!(ingress.drain(1).len(), 1);
    }
}
