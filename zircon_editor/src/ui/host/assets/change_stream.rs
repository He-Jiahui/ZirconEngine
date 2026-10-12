use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};
use zircon_runtime::core::framework::channel::ChannelWakeCallback;

use super::{EditorAssetChangeKind, EditorAssetChangeRecord};

// Overflow converges to one catalog refresh, which preserves the latest committed
// generation without allowing a paused consumer to retain unbounded asset keys.
const MAX_PENDING_EDITOR_ASSET_CHANGES: usize = 512;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum EditorAssetChangeKey {
    Catalog,
    Asset {
        kind: EditorAssetChangeKind,
        uuid: Option<String>,
        locator: Option<String>,
    },
}

impl EditorAssetChangeKey {
    fn from_change(change: &EditorAssetChangeRecord) -> Self {
        if change.kind == EditorAssetChangeKind::CatalogChanged {
            Self::Catalog
        } else {
            Self::Asset {
                kind: change.kind,
                uuid: change.uuid.clone(),
                locator: change.locator.clone(),
            }
        }
    }
}

fn move_change_key_to_tail(order: &mut VecDeque<EditorAssetChangeKey>, key: EditorAssetChangeKey) {
    if order.back() == Some(&key) {
        return;
    }
    order.retain(|pending_key| pending_key != &key);
    order.push_back(key);
}

struct PendingEditorAssetChange {
    change: Arc<EditorAssetChangeRecord>,
    publish_sequence: u64,
    queued_at: Instant,
}

#[derive(Default)]
struct EditorAssetChangeMailbox {
    order: VecDeque<EditorAssetChangeKey>,
    pending: HashMap<EditorAssetChangeKey, PendingEditorAssetChange>,
    wake: Option<ChannelWakeCallback>,
}

impl EditorAssetChangeMailbox {
    fn with_wake(wake: ChannelWakeCallback) -> Self {
        Self {
            wake: Some(wake),
            ..Default::default()
        }
    }

    fn push(&mut self, change: Arc<EditorAssetChangeRecord>, publish_sequence: u64) -> bool {
        let key = EditorAssetChangeKey::from_change(&change);
        if let Some(current) = self.pending.get_mut(&key) {
            if change.catalog_revision < current.change.catalog_revision
                || (change.catalog_revision == current.change.catalog_revision
                    && publish_sequence <= current.publish_sequence)
            {
                return false;
            }
            current.change = change;
            current.publish_sequence = publish_sequence;
            current.queued_at = Instant::now();
            move_change_key_to_tail(&mut self.order, key);
            return true;
        }

        if self.pending.len() >= MAX_PENDING_EDITOR_ASSET_CHANGES {
            self.collapse_to_latest_catalog_generation(change, publish_sequence);
            return true;
        }

        self.order.push_back(key.clone());
        self.pending.insert(
            key,
            PendingEditorAssetChange {
                change,
                publish_sequence,
                queued_at: Instant::now(),
            },
        );
        true
    }

    fn collapse_to_latest_catalog_generation(
        &mut self,
        incoming: Arc<EditorAssetChangeRecord>,
        publish_sequence: u64,
    ) {
        let catalog_revision = self
            .pending
            .values()
            .map(|pending| pending.change.catalog_revision)
            .chain(std::iter::once(incoming.catalog_revision))
            .max()
            .unwrap_or_default();
        self.order.clear();
        self.pending.clear();

        let key = EditorAssetChangeKey::Catalog;
        self.order.push_back(key.clone());
        self.pending.insert(
            key,
            PendingEditorAssetChange {
                change: Arc::new(EditorAssetChangeRecord {
                    kind: EditorAssetChangeKind::CatalogChanged,
                    catalog_revision,
                    uuid: None,
                    locator: None,
                }),
                publish_sequence,
                queued_at: Instant::now(),
            },
        );
    }

    fn pop(&mut self) -> Option<EditorAssetChangeDelivery> {
        while let Some(key) = self.order.pop_front() {
            let Some(pending) = self.pending.remove(&key) else {
                continue;
            };
            return Some(EditorAssetChangeDelivery {
                change: pending.change,
                queue_age: pending.queued_at.elapsed(),
            });
        }
        None
    }

    fn clear(&mut self) -> usize {
        let discarded = self.pending.len();
        self.order.clear();
        self.pending.clear();
        discarded
    }
}

#[derive(Clone, Debug)]
pub struct EditorAssetChangeDelivery {
    pub change: Arc<EditorAssetChangeRecord>,
    pub queue_age: Duration,
}

#[derive(Clone)]
pub struct EditorAssetChangeSubscription {
    mailbox: Arc<Mutex<EditorAssetChangeMailbox>>,
}

impl EditorAssetChangeSubscription {
    pub fn try_recv(&self) -> Option<EditorAssetChangeDelivery> {
        self.lock_mailbox().pop()
    }

    pub fn pending_len(&self) -> usize {
        self.lock_mailbox().pending.len()
    }

    pub fn discard_pending(&self) -> usize {
        self.lock_mailbox().clear()
    }

    fn lock_mailbox(&self) -> std::sync::MutexGuard<'_, EditorAssetChangeMailbox> {
        self.mailbox
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[derive(Clone)]
pub(crate) struct EditorAssetChangeHub {
    subscribers: Arc<Mutex<Vec<Weak<Mutex<EditorAssetChangeMailbox>>>>>,
    publish_order: Arc<Mutex<()>>,
    next_publish_sequence: Arc<AtomicU64>,
}

impl Default for EditorAssetChangeHub {
    fn default() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(Vec::new())),
            publish_order: Arc::new(Mutex::new(())),
            next_publish_sequence: Arc::new(AtomicU64::new(1)),
        }
    }
}

impl EditorAssetChangeHub {
    pub(crate) fn subscribe(&self) -> EditorAssetChangeSubscription {
        self.subscribe_internal(EditorAssetChangeMailbox::default())
    }

    pub(crate) fn subscribe_with_wake(
        &self,
        wake: ChannelWakeCallback,
    ) -> EditorAssetChangeSubscription {
        self.subscribe_internal(EditorAssetChangeMailbox::with_wake(wake))
    }

    fn subscribe_internal(
        &self,
        mailbox: EditorAssetChangeMailbox,
    ) -> EditorAssetChangeSubscription {
        let mailbox = Arc::new(Mutex::new(mailbox));
        let mut subscribers = self.lock_subscribers();
        subscribers.retain(|subscriber| subscriber.strong_count() > 0);
        subscribers.push(Arc::downgrade(&mailbox));
        drop(subscribers);
        EditorAssetChangeSubscription { mailbox }
    }

    pub(crate) fn publish(&self, change: EditorAssetChangeRecord) {
        let _publish_guard = self
            .publish_order
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let publish_sequence = self.next_publish_sequence.fetch_add(1, Ordering::Relaxed);
        let change = Arc::new(change);
        // The owner lock protects only weak subscription membership. Mailbox
        // fanout happens after it is released and shares this immutable payload.
        let targets = {
            let mut subscribers = self.lock_subscribers();
            let mut targets = Vec::with_capacity(subscribers.len());
            subscribers.retain(|subscriber| {
                let Some(mailbox) = subscriber.upgrade() else {
                    return false;
                };
                targets.push(mailbox);
                true
            });
            targets
        };

        for mailbox in targets {
            let wake = {
                let mut mailbox = mailbox
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                mailbox
                    .push(Arc::clone(&change), publish_sequence)
                    .then(|| mailbox.wake.clone())
                    .flatten()
            };
            if let Some(wake) = wake {
                wake();
            }
        }
    }

    fn lock_subscribers(
        &self,
    ) -> std::sync::MutexGuard<'_, Vec<Weak<Mutex<EditorAssetChangeMailbox>>>> {
        self.subscribers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
#[path = "tests/change_stream.rs"]
mod tests;
