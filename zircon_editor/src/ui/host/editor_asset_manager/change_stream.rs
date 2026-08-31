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
mod tests {
    use std::collections::VecDeque;
    use std::hint::black_box;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Instant;

    use super::{
        move_change_key_to_tail, EditorAssetChangeHub, EditorAssetChangeKey, EditorAssetChangeKind,
        EditorAssetChangeRecord, MAX_PENDING_EDITOR_ASSET_CHANGES,
    };

    fn legacy_move_change_key_to_tail(
        order: &mut VecDeque<EditorAssetChangeKey>,
        key: EditorAssetChangeKey,
    ) {
        order.retain(|pending_key| pending_key != &key);
        order.push_back(key);
    }

    fn benchmark_key(index: usize) -> EditorAssetChangeKey {
        EditorAssetChangeKey::Asset {
            kind: EditorAssetChangeKind::PreviewChanged,
            uuid: Some(format!("asset-{index:04}")),
            locator: Some(format!("res://asset-{index:04}.asset")),
        }
    }

    #[test]
    fn same_asset_preview_storm_coalesces_to_latest_revision() {
        let hub = EditorAssetChangeHub::default();
        let subscription = hub.subscribe();
        for revision in 0..10_000 {
            hub.publish(change(
                EditorAssetChangeKind::PreviewChanged,
                revision,
                Some("asset-a"),
            ));
        }

        assert_eq!(subscription.pending_len(), 1);
        let delivery = subscription.try_recv().expect("latest preview change");
        assert_eq!(delivery.change.catalog_revision, 9_999);
        assert!(subscription.try_recv().is_none());
    }

    #[test]
    fn overflow_collapses_to_latest_catalog_generation() {
        let hub = EditorAssetChangeHub::default();
        let subscription = hub.subscribe();
        for revision in 0..=MAX_PENDING_EDITOR_ASSET_CHANGES as u64 {
            hub.publish(change(
                EditorAssetChangeKind::ReferenceChanged,
                revision,
                Some(&format!("asset-{revision}")),
            ));
        }

        assert_eq!(subscription.pending_len(), 1);
        let delivery = subscription.try_recv().expect("overflow fallback");
        assert_eq!(delivery.change.kind, EditorAssetChangeKind::CatalogChanged);
        assert_eq!(
            delivery.change.catalog_revision,
            MAX_PENDING_EDITOR_ASSET_CHANGES as u64
        );
    }

    #[test]
    fn fanout_shares_one_immutable_change_payload() {
        let hub = EditorAssetChangeHub::default();
        let left = hub.subscribe();
        let right = hub.subscribe();
        hub.publish(change(
            EditorAssetChangeKind::PreviewChanged,
            7,
            Some("asset-a"),
        ));

        let left = left.try_recv().expect("left delivery");
        let right = right.try_recv().expect("right delivery");
        assert!(Arc::ptr_eq(&left.change, &right.change));
    }

    #[test]
    fn wake_subscription_notifies_after_a_change_enters_its_mailbox() {
        let hub = EditorAssetChangeHub::default();
        let wake_count = Arc::new(AtomicUsize::new(0));
        let wake_count_for_callback = Arc::clone(&wake_count);
        let subscription = hub.subscribe_with_wake(Arc::new(move || {
            wake_count_for_callback.fetch_add(1, Ordering::Relaxed);
        }));

        hub.publish(change(
            EditorAssetChangeKind::PreviewChanged,
            7,
            Some("asset-a"),
        ));

        assert_eq!(wake_count.load(Ordering::Relaxed), 1);
        assert_eq!(subscription.pending_len(), 1);
    }

    #[test]
    fn coalesced_key_moves_to_tail_without_revision_regression() {
        let hub = EditorAssetChangeHub::default();
        let subscription = hub.subscribe();
        hub.publish(change(
            EditorAssetChangeKind::PreviewChanged,
            1,
            Some("asset-a"),
        ));
        hub.publish(change(
            EditorAssetChangeKind::PreviewChanged,
            2,
            Some("asset-b"),
        ));
        hub.publish(change(
            EditorAssetChangeKind::PreviewChanged,
            3,
            Some("asset-a"),
        ));

        let first = subscription.try_recv().expect("asset-b");
        let second = subscription.try_recv().expect("newer asset-a");
        assert_eq!(first.change.uuid.as_deref(), Some("asset-b"));
        assert_eq!(first.change.catalog_revision, 2);
        assert_eq!(second.change.uuid.as_deref(), Some("asset-a"));
        assert_eq!(second.change.catalog_revision, 3);
    }

    #[test]
    fn optimization_batch_eq_tail_coalescing_preserves_existing_queue_order() {
        let seed = (0..MAX_PENDING_EDITOR_ASSET_CHANGES)
            .map(benchmark_key)
            .collect::<VecDeque<_>>();
        for target_index in [
            0,
            MAX_PENDING_EDITOR_ASSET_CHANGES / 2,
            MAX_PENDING_EDITOR_ASSET_CHANGES - 1,
        ] {
            let key = benchmark_key(target_index);
            let mut legacy = seed.clone();
            let mut optimized = seed.clone();

            legacy_move_change_key_to_tail(&mut legacy, key.clone());
            move_change_key_to_tail(&mut optimized, key);

            assert_eq!(optimized, legacy);
        }

        let source = include_str!("change_stream.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("asset change stream production implementation");
        let movement = production
            .split("fn move_change_key_to_tail(")
            .nth(1)
            .expect("coalesced key movement");
        assert!(movement.contains("order.back() == Some(&key)"));
    }

    #[test]
    #[ignore = "release-only tail coalescing fast-path benchmark"]
    fn optimization_batch_eq_tail_coalescing_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const MOVES_PER_SAMPLE: usize = 8_192;

        fn measure_legacy(
            seed: &VecDeque<EditorAssetChangeKey>,
            key: &EditorAssetChangeKey,
        ) -> u128 {
            let mut order = seed.clone();
            let started = Instant::now();
            for _ in 0..MOVES_PER_SAMPLE {
                legacy_move_change_key_to_tail(&mut order, black_box(key.clone()));
            }
            black_box(order.len());
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(
            seed: &VecDeque<EditorAssetChangeKey>,
            key: &EditorAssetChangeKey,
        ) -> u128 {
            let mut order = seed.clone();
            let started = Instant::now();
            for _ in 0..MOVES_PER_SAMPLE {
                move_change_key_to_tail(&mut order, black_box(key.clone()));
            }
            black_box(order.len());
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let seed = (0..MAX_PENDING_EDITOR_ASSET_CHANGES)
            .map(benchmark_key)
            .collect::<VecDeque<_>>();
        let key = seed.back().expect("full benchmark mailbox").clone();
        for _ in 0..4 {
            black_box(measure_legacy(&seed, &key));
            black_box(measure_optimized(&seed, &key));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy(&seed, &key));
                optimized_samples.push(measure_optimized(&seed, &key));
            } else {
                optimized_samples.push(measure_optimized(&seed, &key));
                legacy_samples.push(measure_legacy(&seed, &key));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "EDITOR379_TAIL_COALESCING_FAST_PATH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             moves_per_sample={MOVES_PER_SAMPLE} mailbox_size={MAX_PENDING_EDITOR_ASSET_CHANGES} \
             pair_order=alternating_legacy_even legacy_queue_scans_per_move=1 \
             optimized_queue_scans_per_tail_move=0 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(20),
            "tail coalescing must reduce P95 by at least 80%: \
             legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    #[test]
    fn concurrent_publishers_converge_all_subscribers_to_same_latest_payload() {
        let hub = EditorAssetChangeHub::default();
        let left = hub.subscribe();
        let right = hub.subscribe();
        let publishers = (0..4)
            .map(|_| {
                let hub = hub.clone();
                std::thread::spawn(move || {
                    for _ in 0..1_000 {
                        hub.publish(change(
                            EditorAssetChangeKind::PreviewChanged,
                            7,
                            Some("asset-a"),
                        ));
                    }
                })
            })
            .collect::<Vec<_>>();
        for publisher in publishers {
            publisher.join().expect("publisher");
        }

        let left_delivery = left.try_recv().expect("left latest");
        let right_delivery = right.try_recv().expect("right latest");
        assert!(Arc::ptr_eq(&left_delivery.change, &right_delivery.change));
        assert!(left.try_recv().is_none());
        assert!(right.try_recv().is_none());
    }

    #[test]
    fn discarded_or_completed_delivery_is_not_implicitly_requeued() {
        let hub = EditorAssetChangeHub::default();
        let subscription = hub.subscribe();
        hub.publish(change(
            EditorAssetChangeKind::PreviewAdmissionAvailable,
            3,
            Some("asset-a"),
        ));

        assert_eq!(subscription.discard_pending(), 1);
        assert_eq!(subscription.pending_len(), 0);
        assert!(subscription.try_recv().is_none());
    }

    #[test]
    fn silent_subscribe_drop_churn_prunes_dead_owners() {
        let hub = EditorAssetChangeHub::default();
        for _ in 0..10_000 {
            drop(hub.subscribe());
        }

        let live = hub.subscribe();
        assert_eq!(hub.lock_subscribers().len(), 1);
        drop(live);
    }

    fn change(
        kind: EditorAssetChangeKind,
        catalog_revision: u64,
        uuid: Option<&str>,
    ) -> EditorAssetChangeRecord {
        EditorAssetChangeRecord {
            kind,
            catalog_revision,
            uuid: uuid.map(str::to_string),
            locator: uuid.map(|uuid| format!("res://{uuid}.asset")),
        }
    }
}
