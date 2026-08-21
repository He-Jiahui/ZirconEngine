use std::collections::{BTreeSet, VecDeque};

use super::*;

fn handle(slot: usize) -> RuntimeEventMirrorSubscriptionHandle {
    RuntimeEventMirrorSubscriptionHandle::new(slot, 1)
}

fn populated_queue(count: usize) -> RuntimeEventMirrorReclaimQueue {
    let mut queue = RuntimeEventMirrorReclaimQueue::default();
    for slot in 0..count {
        let handle = handle(slot);
        queue.register_live_record(handle);
        queue.enqueue(handle);
    }
    queue
}

#[test]
fn indexed_reclaim_queue_unlinks_retired_handles_without_reordering_survivors() {
    let mut queue = populated_queue(8);
    for slot in [7, 3, 0] {
        queue.retire_live_record(handle(slot));
    }

    assert_eq!(
        queue.drain(),
        [1, 2, 4, 5, 6].map(handle).to_vec(),
        "retirement must not reorder surviving reclaim intents"
    );
    queue.enqueue(handle(1));
    assert_eq!(
        queue.drain(),
        vec![handle(1)],
        "draining must reset the indexed FIFO for reuse"
    );
}

#[derive(Default)]
struct LegacyReclaimQueue {
    pending: VecDeque<RuntimeEventMirrorSubscriptionHandle>,
    pending_handles: BTreeSet<RuntimeEventMirrorSubscriptionHandle>,
    live_handles: BTreeSet<RuntimeEventMirrorSubscriptionHandle>,
}

impl LegacyReclaimQueue {
    fn populate(count: usize) -> Self {
        let mut queue = Self::default();
        for slot in 0..count {
            let handle = handle(slot);
            queue.live_handles.insert(handle);
            queue.pending_handles.insert(handle);
            queue.pending.push_back(handle);
        }
        queue
    }

    fn retire(&mut self, handle: RuntimeEventMirrorSubscriptionHandle) {
        assert!(self.live_handles.remove(&handle));
        if self.pending_handles.remove(&handle) {
            self.pending.retain(|pending| *pending != handle);
        }
    }

    fn drain(mut self) -> Vec<RuntimeEventMirrorSubscriptionHandle> {
        self.pending.drain(..).collect()
    }
}

#[test]
#[ignore = "release-mode reclaim queue evidence; run explicitly"]
fn runtime_event_mirror_indexed_reclaim_queue_release_benchmark() {
    const SAMPLE_PAIRS: usize = 21;
    const PENDING_HANDLES: usize = 4_096;
    const RETIRED_HANDLES: usize = 2_048;

    fn retired_handles() -> impl Iterator<Item = RuntimeEventMirrorSubscriptionHandle> {
        (0..RETIRED_HANDLES).rev().map(|index| handle(index * 2))
    }

    fn run_legacy() -> (u128, Vec<RuntimeEventMirrorSubscriptionHandle>) {
        let mut queue = LegacyReclaimQueue::populate(PENDING_HANDLES);
        let started = Instant::now();
        for handle in retired_handles() {
            queue.retire(handle);
        }
        (
            started.elapsed().as_nanos(),
            std::hint::black_box(queue.drain()),
        )
    }

    fn run_indexed() -> (u128, Vec<RuntimeEventMirrorSubscriptionHandle>) {
        let mut queue = populated_queue(PENDING_HANDLES);
        let started = Instant::now();
        for handle in retired_handles() {
            queue.retire_live_record(handle);
        }
        (
            started.elapsed().as_nanos(),
            std::hint::black_box(queue.drain()),
        )
    }

    fn nearest_rank(samples: &mut [u128], percentile: usize) -> u128 {
        samples.sort_unstable();
        let rank = samples.len().saturating_mul(percentile).saturating_add(99) / 100;
        samples[rank.saturating_sub(1).min(samples.len().saturating_sub(1))]
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut expected_survivors = None;
    for pair in 0..SAMPLE_PAIRS {
        let legacy_first = pair % 2 == 0;
        let first = if legacy_first {
            run_legacy()
        } else {
            run_indexed()
        };
        let second = if legacy_first {
            run_indexed()
        } else {
            run_legacy()
        };
        let (legacy, indexed) = if legacy_first {
            (first, second)
        } else {
            (second, first)
        };
        assert_eq!(legacy.1, indexed.1);
        if let Some(expected) = &expected_survivors {
            assert_eq!(&legacy.1, expected);
        } else {
            expected_survivors = Some(legacy.1.clone());
        }
        legacy_samples.push(legacy.0);
        indexed_samples.push(indexed.0);
    }

    let legacy_p50_ns = nearest_rank(&mut legacy_samples.clone(), 50);
    let legacy_p95_ns = nearest_rank(&mut legacy_samples.clone(), 95);
    let indexed_p50_ns = nearest_rank(&mut indexed_samples.clone(), 50);
    let indexed_p95_ns = nearest_rank(&mut indexed_samples.clone(), 95);
    let legacy_ns = legacy_samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let indexed_ns = indexed_samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let legacy_retire_inspections = RETIRED_HANDLES.saturating_mul(
        PENDING_HANDLES
            .saturating_mul(2)
            .saturating_sub(RETIRED_HANDLES)
            .saturating_add(1),
    ) / 2;
    let indexed_unlinks = RETIRED_HANDLES;
    let retire_work_reduction_basis_points = legacy_retire_inspections
        .saturating_sub(indexed_unlinks)
        .saturating_mul(10_000)
        / legacy_retire_inspections;

    assert!(
        indexed_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "indexed reclaim P95 must be at most 25% of legacy retain P95"
    );
    println!(
        "PERF-MVP-SEMR-P2-006 sample_pairs={SAMPLE_PAIRS} sample_order=alternating \
percentile_method=nearest_rank pending_handles={PENDING_HANDLES} retired_handles={RETIRED_HANDLES} \
legacy_ns={legacy_ns} indexed_ns={indexed_ns} legacy_p50_ns={legacy_p50_ns} \
legacy_p95_ns={legacy_p95_ns} indexed_p50_ns={indexed_p50_ns} indexed_p95_ns={indexed_p95_ns} \
legacy_retire_inspections={legacy_retire_inspections} indexed_unlinks={indexed_unlinks} \
retire_work_reduction_basis_points={retire_work_reduction_basis_points}"
    );
}
