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

// Manifest 与 block loader 共用的待派发队列。多个票据可等待同一键；
// 派发优先级取当前最高需求，票据撤销后须重新排序而非沿用旧优先级。
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
#[path = "tests/io_frontier.rs"]
mod tests;
