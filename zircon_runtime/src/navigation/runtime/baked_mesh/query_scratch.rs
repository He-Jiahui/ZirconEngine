use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::core::math::Real;

#[derive(Debug, Default)]
pub(super) struct FallbackQueryScratch {
    pub(super) open: BinaryHeap<PathOpenNode>,
    pub(super) best_cost: Vec<Real>,
    pub(super) previous: Vec<Option<usize>>,
    pub(super) traversal: Vec<usize>,
    pub(super) path: Vec<usize>,
    pub(super) query_count: u64,
    query_epoch: u64,
    traversal_epoch: u64,
    best_cost_epochs: Vec<u64>,
    previous_epochs: Vec<u64>,
    visited_epochs: Vec<u64>,
}

impl FallbackQueryScratch {
    pub(super) fn prepare(&mut self, polygon_count: usize) {
        self.open.clear();
        self.path.clear();
        self.traversal.clear();
        self.best_cost.resize(polygon_count, Real::INFINITY);
        self.best_cost_epochs.resize(polygon_count, 0);
        self.previous.resize(polygon_count, None);
        self.previous_epochs.resize(polygon_count, 0);
        self.visited_epochs.resize(polygon_count, 0);
        self.advance_query_epoch();
        self.query_count = self.query_count.saturating_add(1);
    }

    pub(super) fn reset_traversal(&mut self) {
        self.traversal.clear();
        self.advance_traversal_epoch();
    }

    pub(super) fn best_cost(&self, polygon: usize) -> Real {
        if self.best_cost_epochs[polygon] == self.query_epoch {
            self.best_cost[polygon]
        } else {
            Real::INFINITY
        }
    }

    pub(super) fn set_best_cost(&mut self, polygon: usize, cost: Real) {
        self.best_cost[polygon] = cost;
        self.best_cost_epochs[polygon] = self.query_epoch;
    }

    pub(super) fn previous(&self, polygon: usize) -> Option<usize> {
        (self.previous_epochs[polygon] == self.query_epoch)
            .then_some(self.previous[polygon])
            .flatten()
    }

    pub(super) fn set_previous(&mut self, polygon: usize, previous: usize) {
        self.previous[polygon] = Some(previous);
        self.previous_epochs[polygon] = self.query_epoch;
    }

    pub(super) fn mark_visited(&mut self, polygon: usize) -> bool {
        if self.visited_epochs[polygon] == self.traversal_epoch {
            return false;
        }
        self.visited_epochs[polygon] = self.traversal_epoch;
        true
    }

    pub(super) fn is_visited(&self, polygon: usize) -> bool {
        self.visited_epochs[polygon] == self.traversal_epoch
    }

    fn advance_query_epoch(&mut self) {
        self.query_epoch = self.query_epoch.wrapping_add(1);
        if self.query_epoch != 0 {
            return;
        }
        // Stamps make untouched slots logically default; only epoch wrap needs a full reset.
        self.best_cost_epochs.fill(0);
        self.previous_epochs.fill(0);
        self.query_epoch = 1;
    }

    fn advance_traversal_epoch(&mut self) {
        self.traversal_epoch = self.traversal_epoch.wrapping_add(1);
        if self.traversal_epoch != 0 {
            return;
        }
        // Traversal marks have an independent lifetime from A* cost and predecessor slots.
        self.visited_epochs.fill(0);
        self.traversal_epoch = 1;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PathOpenNode {
    pub(super) polygon: usize,
    pub(super) estimated_total: Real,
}

impl Eq for PathOpenNode {}

impl Ord for PathOpenNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .estimated_total
            .total_cmp(&self.estimated_total)
            .then_with(|| other.polygon.cmp(&self.polygon))
    }
}

impl PartialOrd for PathOpenNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
