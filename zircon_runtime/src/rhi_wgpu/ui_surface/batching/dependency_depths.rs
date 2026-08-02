use std::cmp::Ordering;
use std::ops::Range;

use crate::rhi::UiSurfaceRect;

use super::DrawItem;

pub(super) fn dependency_depths(items: &[DrawItem]) -> (Vec<usize>, usize, usize, usize) {
    if items.is_empty() {
        return (Vec::new(), 0, 0, 0);
    }

    let axis = preferred_sweep_axis(items);
    let index = IntervalIndex::new(items, axis);
    let mut depths = vec![0; items.len()];
    let mut dependency_count = 0;
    let mut overlap_candidate_count = 0;

    // Items are already in painter order. Querying only earlier indices lets us
    // update the longest dependency depth without retaining every overlap edge.
    for later_index in 0..items.len() {
        let later_rect = items[later_index].rect();
        index.query(later_rect, |earlier_index| {
            if earlier_index >= later_index {
                return;
            }
            overlap_candidate_count += 1;
            if !rects_intersect(items[earlier_index].rect(), later_rect) {
                return;
            }
            dependency_count += 1;
            depths[later_index] = depths[later_index].max(depths[earlier_index] + 1);
        });
    }

    let layer_count = depths
        .iter()
        .copied()
        .max()
        .map(|depth| depth + 1)
        .unwrap_or(0);
    (
        depths,
        layer_count,
        dependency_count,
        overlap_candidate_count,
    )
}

#[derive(Clone, Copy)]
enum Axis {
    X,
    Y,
}

fn preferred_sweep_axis(items: &[DrawItem]) -> Axis {
    let (total_width, total_height) = items.iter().fold((0.0_f64, 0.0_f64), |sum, item| {
        let rect = item.rect();
        (
            sum.0 + f64::from(rect.width.max(0.0)),
            sum.1 + f64::from(rect.height.max(0.0)),
        )
    });
    if total_height < total_width {
        Axis::Y
    } else {
        Axis::X
    }
}

struct IntervalIndex<'a> {
    items: &'a [DrawItem],
    axis: Axis,
    nodes: Vec<IntervalNode>,
    crossing_by_start: Vec<usize>,
    crossing_by_end: Vec<usize>,
    root: Option<usize>,
}

impl<'a> IntervalIndex<'a> {
    fn new(items: &'a [DrawItem], axis: Axis) -> Self {
        let mut indices = (0..items.len()).collect::<Vec<_>>();
        let mut nodes = Vec::with_capacity(items.len());
        let mut crossing_by_start = Vec::with_capacity(items.len());
        let mut crossing_by_end = Vec::with_capacity(items.len());
        let root = IntervalNode::build(
            &mut indices,
            items,
            axis,
            &mut nodes,
            &mut crossing_by_start,
            &mut crossing_by_end,
        );
        Self {
            items,
            axis,
            nodes,
            crossing_by_start,
            crossing_by_end,
            root,
        }
    }

    fn query(&self, rect: UiSurfaceRect, mut visit: impl FnMut(usize)) {
        if let Some(root) = self.root {
            self.query_node(root, rect, &mut visit);
        }
    }

    fn query_node(&self, node_index: usize, rect: UiSurfaceRect, visit: &mut impl FnMut(usize)) {
        let node = &self.nodes[node_index];
        let (start, end) = interval(rect, self.axis);
        match (end.total_cmp(&node.center), start.total_cmp(&node.center)) {
            (Ordering::Less | Ordering::Equal, _) => {
                for index in &self.crossing_by_start[node.crossing_by_start.clone()] {
                    if interval(self.items[*index].rect(), self.axis).0 >= end {
                        break;
                    }
                    visit(*index);
                }
                if let Some(left) = node.left {
                    self.query_node(left, rect, visit);
                }
            }
            (_, Ordering::Greater | Ordering::Equal) => {
                for index in &self.crossing_by_end[node.crossing_by_end.clone()] {
                    if interval(self.items[*index].rect(), self.axis).1 <= start {
                        break;
                    }
                    visit(*index);
                }
                if let Some(right) = node.right {
                    self.query_node(right, rect, visit);
                }
            }
            _ => {
                self.crossing_by_start[node.crossing_by_start.clone()]
                    .iter()
                    .copied()
                    .for_each(&mut *visit);
                if let Some(left) = node.left {
                    self.query_node(left, rect, visit);
                }
                if let Some(right) = node.right {
                    self.query_node(right, rect, visit);
                }
            }
        }
    }
}

struct IntervalNode {
    center: f32,
    crossing_by_start: Range<usize>,
    crossing_by_end: Range<usize>,
    left: Option<usize>,
    right: Option<usize>,
}

impl IntervalNode {
    fn build(
        indices: &mut [usize],
        items: &[DrawItem],
        axis: Axis,
        nodes: &mut Vec<Self>,
        pooled_by_start: &mut Vec<usize>,
        pooled_by_end: &mut Vec<usize>,
    ) -> Option<usize> {
        if indices.is_empty() {
            return None;
        }

        let median = indices.len() / 2;
        indices.select_nth_unstable_by(median, |left, right| {
            interval_midpoint(items[*left].rect(), axis)
                .total_cmp(&interval_midpoint(items[*right].rect(), axis))
                .then_with(|| left.cmp(right))
        });
        let center = interval_midpoint(items[indices[median]].rect(), axis);
        let (left_end, crossing_end) = partition_intervals(indices, items, axis, center);
        let (left_indices, remaining) = indices.split_at_mut(left_end);
        let (crossing, right_indices) = remaining.split_at_mut(crossing_end - left_end);

        crossing.sort_unstable_by(|left, right| {
            interval(items[*left].rect(), axis)
                .0
                .total_cmp(&interval(items[*right].rect(), axis).0)
                .then_with(|| left.cmp(right))
        });
        let start_offset = pooled_by_start.len();
        pooled_by_start.extend_from_slice(crossing);
        let crossing_by_start = start_offset..pooled_by_start.len();

        crossing.sort_unstable_by(|left, right| {
            interval(items[*right].rect(), axis)
                .1
                .total_cmp(&interval(items[*left].rect(), axis).1)
                .then_with(|| left.cmp(right))
        });
        let end_offset = pooled_by_end.len();
        pooled_by_end.extend_from_slice(crossing);
        let crossing_by_end = end_offset..pooled_by_end.len();

        let node_index = nodes.len();
        nodes.push(Self {
            center,
            crossing_by_start,
            crossing_by_end,
            left: None,
            right: None,
        });
        let left = Self::build(
            left_indices,
            items,
            axis,
            nodes,
            pooled_by_start,
            pooled_by_end,
        );
        let right = Self::build(
            right_indices,
            items,
            axis,
            nodes,
            pooled_by_start,
            pooled_by_end,
        );
        nodes[node_index].left = left;
        nodes[node_index].right = right;
        Some(node_index)
    }
}

fn partition_intervals(
    indices: &mut [usize],
    items: &[DrawItem],
    axis: Axis,
    center: f32,
) -> (usize, usize) {
    let mut left_end = 0;
    let mut cursor = 0;
    let mut right_start = indices.len();
    while cursor < right_start {
        let (start, end) = interval(items[indices[cursor]].rect(), axis);
        if end <= center {
            indices.swap(left_end, cursor);
            left_end += 1;
            cursor += 1;
        } else if start >= center {
            right_start -= 1;
            indices.swap(cursor, right_start);
        } else {
            cursor += 1;
        }
    }
    (left_end, right_start)
}

#[cfg(test)]
pub(super) fn interval_index_storage_counts(items: &[DrawItem]) -> (usize, usize, usize) {
    let index = IntervalIndex::new(items, preferred_sweep_axis(items));
    (
        index.nodes.len(),
        index.crossing_by_start.len(),
        index.crossing_by_end.len(),
    )
}

fn interval(rect: UiSurfaceRect, axis: Axis) -> (f32, f32) {
    match axis {
        Axis::X => (rect.x, rect.x + rect.width),
        Axis::Y => (rect.y, rect.y + rect.height),
    }
}

fn interval_midpoint(rect: UiSurfaceRect, axis: Axis) -> f32 {
    let (start, end) = interval(rect, axis);
    start + (end - start) * 0.5
}

fn rects_intersect(left: UiSurfaceRect, right: UiSurfaceRect) -> bool {
    let left_right = left.x + left.width;
    let left_bottom = left.y + left.height;
    let right_right = right.x + right.width;
    let right_bottom = right.y + right.height;
    left.x < right_right && right.x < left_right && left.y < right_bottom && right.y < left_bottom
}
