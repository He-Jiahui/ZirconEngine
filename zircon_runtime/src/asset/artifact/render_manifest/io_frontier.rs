use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hash::Hash;
use std::time::Instant;

use super::RenderArtifactIoPriority;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct RenderArtifactIoDemandKey {
    priority: u8,
    deadline: Option<Reverse<Instant>>,
    ticket_id: Reverse<u64>,
}

impl RenderArtifactIoDemandKey {
    pub(super) fn new(
        priority: RenderArtifactIoPriority,
        deadline: Option<Instant>,
        ticket_id: u64,
    ) -> Self {
        Self {
            priority: priority.raw(),
            deadline: deadline.map(Reverse),
            ticket_id: Reverse(ticket_id),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct RenderArtifactIoFrontierKey {
    priority: u8,
    deadline: Option<Reverse<Instant>>,
    sequence: Reverse<u64>,
}

impl RenderArtifactIoFrontierKey {
    fn from_demand(demand: RenderArtifactIoDemandKey, sequence: u64) -> Self {
        Self {
            priority: demand.priority,
            deadline: demand.deadline,
            sequence: Reverse(sequence),
        }
    }
}

pub(super) struct RenderArtifactIoFrontier<K> {
    ordered: BTreeMap<RenderArtifactIoFrontierKey, K>,
    queued: HashMap<K, RenderArtifactIoFrontierKey>,
    waiters: HashMap<K, BTreeSet<RenderArtifactIoDemandKey>>,
}

impl<K> RenderArtifactIoFrontier<K>
where
    K: Clone + Eq + Hash,
{
    pub(super) fn new() -> Self {
        Self {
            ordered: BTreeMap::new(),
            queued: HashMap::new(),
            waiters: HashMap::new(),
        }
    }

    pub(super) fn queued_len(&self) -> usize {
        self.ordered.len()
    }

    pub(super) fn add_waiter(&mut self, key: K, demand: RenderArtifactIoDemandKey) {
        self.waiters.entry(key.clone()).or_default().insert(demand);
        self.refresh(&key);
    }

    pub(super) fn remove_waiter(&mut self, key: &K, demand: RenderArtifactIoDemandKey) {
        let remove_set = self.waiters.get_mut(key).is_some_and(|waiters| {
            waiters.remove(&demand);
            waiters.is_empty()
        });
        if remove_set {
            self.waiters.remove(key);
        }
        self.refresh(key);
    }

    pub(super) fn enqueue(&mut self, key: K, sequence: u64) {
        if self.queued.contains_key(&key) {
            self.refresh(&key);
            return;
        }
        let Some(demand) = self.effective_demand(&key) else {
            return;
        };
        let frontier_key = RenderArtifactIoFrontierKey::from_demand(demand, sequence);
        self.ordered.insert(frontier_key, key.clone());
        self.queued.insert(key, frontier_key);
    }

    pub(super) fn remove_entry(&mut self, key: &K) {
        if let Some(frontier_key) = self.queued.remove(key) {
            self.ordered.remove(&frontier_key);
        }
        self.waiters.remove(key);
    }

    pub(super) fn pop_highest(&mut self) -> Option<(RenderArtifactIoFrontierKey, K)> {
        let (frontier_key, key) = self.ordered.pop_last()?;
        self.queued.remove(&key);
        Some((frontier_key, key))
    }

    pub(super) fn restore(&mut self, frontier_key: RenderArtifactIoFrontierKey, key: K) {
        self.ordered.insert(frontier_key, key.clone());
        self.queued.insert(key, frontier_key);
    }

    pub(super) fn clear(&mut self) {
        self.ordered.clear();
        self.queued.clear();
        self.waiters.clear();
    }

    fn refresh(&mut self, key: &K) {
        let Some(current) = self.queued.get(key).copied() else {
            return;
        };
        let Some(demand) = self.effective_demand(key) else {
            self.queued.remove(key);
            self.ordered.remove(&current);
            return;
        };
        let next = RenderArtifactIoFrontierKey::from_demand(demand, current.sequence.0);
        if current == next {
            return;
        }
        self.ordered.remove(&current);
        self.ordered.insert(next, key.clone());
        self.queued.insert(key.clone(), next);
    }

    fn effective_demand(&self, key: &K) -> Option<RenderArtifactIoDemandKey> {
        self.waiters
            .get(key)
            .and_then(|waiters| waiters.iter().next_back().copied())
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Duration;

    use super::*;

    #[test]
    fn render_io_frontier_recalculates_priority_when_a_waiter_leaves() {
        let mut frontier = RenderArtifactIoFrontier::<u8>::new();
        let low = RenderArtifactIoDemandKey::new(RenderArtifactIoPriority::LOW, None, 1);
        let critical = RenderArtifactIoDemandKey::new(RenderArtifactIoPriority::CRITICAL, None, 2);
        let normal = RenderArtifactIoDemandKey::new(RenderArtifactIoPriority::NORMAL, None, 3);
        frontier.add_waiter(1, low);
        frontier.enqueue(1, 1);
        frontier.add_waiter(2, normal);
        frontier.enqueue(2, 2);
        frontier.add_waiter(1, critical);

        frontier.remove_waiter(&1, critical);

        assert_eq!(frontier.pop_highest().map(|(_, key)| key), Some(2));
        assert_eq!(frontier.pop_highest().map(|(_, key)| key), Some(1));
    }

    #[test]
    fn render_io_frontier_orders_equal_priority_by_deadline_then_fifo() {
        let mut frontier = RenderArtifactIoFrontier::<u8>::new();
        let now = Instant::now();
        for (key, deadline, sequence) in [
            (1, Some(now + Duration::from_secs(2)), 1),
            (2, Some(now + Duration::from_secs(1)), 2),
            (3, None, 3),
            (4, None, 4),
        ] {
            frontier.add_waiter(
                key,
                RenderArtifactIoDemandKey::new(
                    RenderArtifactIoPriority::NORMAL,
                    deadline,
                    u64::from(key),
                ),
            );
            frontier.enqueue(key, sequence);
        }

        assert_eq!(frontier.pop_highest().map(|(_, key)| key), Some(2));
        assert_eq!(frontier.pop_highest().map(|(_, key)| key), Some(1));
        assert_eq!(frontier.pop_highest().map(|(_, key)| key), Some(3));
        assert_eq!(frontier.pop_highest().map(|(_, key)| key), Some(4));
    }

    #[test]
    fn optimization_batch_hb_runtime583_frontier_pop_moves_owned_key() {
        let key = "resource/".repeat(256);
        let mut frontier = RenderArtifactIoFrontier::<String>::new();
        frontier.add_waiter(
            key.clone(),
            RenderArtifactIoDemandKey::new(RenderArtifactIoPriority::NORMAL, None, 1),
        );
        frontier.enqueue(key.clone(), 1);

        assert_eq!(frontier.pop_highest().map(|(_, key)| key), Some(key));
        assert_eq!(frontier.queued_len(), 0);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_hb_runtime583_frontier_owned_pop_p95() {
        const SAMPLE_PAIRS: usize = 21;
        const ENTRIES: usize = 512;
        const KEY_SEGMENTS: usize = 512;
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            let legacy_frontier = benchmark_frontier(ENTRIES, KEY_SEGMENTS);
            let optimized_frontier = benchmark_frontier(ENTRIES, KEY_SEGMENTS);
            if pair % 2 == 0 {
                legacy.push(measure(legacy_frontier, false));
                optimized.push(measure(optimized_frontier, true));
            } else {
                optimized.push(measure(optimized_frontier, true));
                legacy.push(measure(legacy_frontier, false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME583_FRONTIER_OWNED_POP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} entries={ENTRIES} \
key_segments={KEY_SEGMENTS} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(75),
            "owned frontier pop must improve long-key P95 by at least 25%"
        );
    }

    fn benchmark_frontier(entries: usize, key_segments: usize) -> RenderArtifactIoFrontier<String> {
        let mut frontier = RenderArtifactIoFrontier::new();
        let prefix = "resource-segment/".repeat(key_segments);
        for index in 0..entries {
            let key = format!("{prefix}{index}");
            frontier.add_waiter(
                key.clone(),
                RenderArtifactIoDemandKey::new(
                    RenderArtifactIoPriority::NORMAL,
                    None,
                    index as u64,
                ),
            );
            frontier.enqueue(key, index as u64);
        }
        frontier
    }

    fn measure(mut frontier: RenderArtifactIoFrontier<String>, optimized: bool) -> u128 {
        let started = Instant::now();
        let mut bytes = 0_usize;
        while !frontier.ordered.is_empty() {
            let popped = if optimized {
                frontier.pop_highest()
            } else {
                pop_highest_legacy(&mut frontier)
            }
            .expect("fixture frontier should remain populated");
            bytes ^= black_box(popped.1.len());
        }
        black_box(bytes);
        started.elapsed().as_nanos().max(1)
    }

    fn pop_highest_legacy(
        frontier: &mut RenderArtifactIoFrontier<String>,
    ) -> Option<(RenderArtifactIoFrontierKey, String)> {
        let (&frontier_key, key) = frontier.ordered.iter().next_back()?;
        let key = key.clone();
        frontier.ordered.remove(&frontier_key);
        frontier.queued.remove(&key);
        Some((frontier_key, key))
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
