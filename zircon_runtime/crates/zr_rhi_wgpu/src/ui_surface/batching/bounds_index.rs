use std::cmp::Ordering;
use std::ops::Range;

use zr_rhi::UiSurfaceRect;

#[derive(Clone, Copy, Debug)]
struct IndexedBounds {
    source_index: usize,
    rect: UiSurfaceRect,
}

#[derive(Clone, Copy, Debug)]
enum Axis {
    X,
    Y,
}

/// Immutable interval tree used by generation-owned UI projections.
///
/// Each source row is retained once in each crossing-order pool. Queries return
/// interval candidates; callers apply the exact two-axis visibility predicate.
#[derive(Clone, Debug, Default)]
pub(super) struct BoundsIndex {
    bounds: Vec<IndexedBounds>,
    axis: Option<Axis>,
    nodes: Vec<IntervalNode>,
    crossing_by_start: Vec<usize>,
    crossing_by_end: Vec<usize>,
    root: Option<usize>,
}

impl BoundsIndex {
    pub(super) fn new(bounds: impl IntoIterator<Item = (usize, UiSurfaceRect)>) -> Self {
        let bounds = bounds
            .into_iter()
            .filter(|(_, rect)| rect.has_finite_positive_area())
            .map(|(source_index, rect)| IndexedBounds { source_index, rect })
            .collect::<Vec<_>>();
        if bounds.is_empty() {
            return Self::default();
        }

        let axis = preferred_axis(&bounds);
        let mut indices = (0..bounds.len()).collect::<Vec<_>>();
        let mut nodes = Vec::with_capacity(bounds.len());
        let mut crossing_by_start = Vec::with_capacity(bounds.len());
        let mut crossing_by_end = Vec::with_capacity(bounds.len());
        let root = IntervalNode::build(
            &mut indices,
            &bounds,
            axis,
            &mut nodes,
            &mut crossing_by_start,
            &mut crossing_by_end,
        );
        Self {
            bounds,
            axis: Some(axis),
            nodes,
            crossing_by_start,
            crossing_by_end,
            root,
        }
    }

    pub(super) fn query_sorted_into(&self, rect: UiSurfaceRect, candidates: &mut Vec<usize>) {
        candidates.clear();
        self.query(rect, |source_index| candidates.push(source_index));
        candidates.sort_unstable();
    }

    pub(super) fn query(&self, rect: UiSurfaceRect, mut visit: impl FnMut(usize)) {
        self.query_indexed_bounds(rect, |bounds| {
            if rects_intersect(bounds.rect, rect) {
                visit(bounds.source_index);
            }
        });
    }

    pub(super) fn query_interval_candidates(
        &self,
        rect: UiSurfaceRect,
        mut visit: impl FnMut(usize),
    ) {
        self.query_indexed_bounds(rect, |bounds| visit(bounds.source_index));
    }

    fn query_indexed_bounds(&self, rect: UiSurfaceRect, mut visit: impl FnMut(IndexedBounds)) {
        if !rect.has_finite_positive_area() {
            return;
        }
        let Some(axis) = self.axis else {
            return;
        };
        let Some(root) = self.root else {
            return;
        };

        self.query_node(root, rect, axis, &mut |bounds_index| {
            visit(self.bounds[bounds_index]);
        });
    }

    fn query_node(
        &self,
        node_index: usize,
        rect: UiSurfaceRect,
        axis: Axis,
        visit: &mut impl FnMut(usize),
    ) {
        let node = &self.nodes[node_index];
        let (start, end) = interval(rect, axis);
        match (end.total_cmp(&node.center), start.total_cmp(&node.center)) {
            (Ordering::Less | Ordering::Equal, _) => {
                for index in &self.crossing_by_start[node.crossing_by_start.clone()] {
                    if interval(self.bounds[*index].rect, axis).0 >= end {
                        break;
                    }
                    visit(*index);
                }
                if let Some(left) = node.left {
                    self.query_node(left, rect, axis, visit);
                }
            }
            (_, Ordering::Greater | Ordering::Equal) => {
                for index in &self.crossing_by_end[node.crossing_by_end.clone()] {
                    if interval(self.bounds[*index].rect, axis).1 <= start {
                        break;
                    }
                    visit(*index);
                }
                if let Some(right) = node.right {
                    self.query_node(right, rect, axis, visit);
                }
            }
            _ => {
                self.crossing_by_start[node.crossing_by_start.clone()]
                    .iter()
                    .copied()
                    .for_each(&mut *visit);
                if let Some(left) = node.left {
                    self.query_node(left, rect, axis, visit);
                }
                if let Some(right) = node.right {
                    self.query_node(right, rect, axis, visit);
                }
            }
        }
    }

    #[cfg(test)]
    pub(super) fn storage_counts(&self) -> (usize, usize, usize) {
        (
            self.nodes.len(),
            self.crossing_by_start.len(),
            self.crossing_by_end.len(),
        )
    }
}

fn preferred_axis(bounds: &[IndexedBounds]) -> Axis {
    let (total_width, total_height) = bounds.iter().fold((0.0_f64, 0.0_f64), |sum, bounds| {
        (
            sum.0 + f64::from(bounds.rect.width.max(0.0)),
            sum.1 + f64::from(bounds.rect.height.max(0.0)),
        )
    });
    if total_height < total_width {
        Axis::Y
    } else {
        Axis::X
    }
}

#[derive(Clone, Debug)]
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
        bounds: &[IndexedBounds],
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
            interval_midpoint(bounds[*left].rect, axis)
                .total_cmp(&interval_midpoint(bounds[*right].rect, axis))
                .then_with(|| left.cmp(right))
        });
        let center = interval_midpoint(bounds[indices[median]].rect, axis);
        let (left_end, crossing_end) = partition_intervals(indices, bounds, axis, center);
        let (left_indices, remaining) = indices.split_at_mut(left_end);
        let (crossing, right_indices) = remaining.split_at_mut(crossing_end - left_end);

        crossing.sort_unstable_by(|left, right| {
            interval(bounds[*left].rect, axis)
                .0
                .total_cmp(&interval(bounds[*right].rect, axis).0)
                .then_with(|| left.cmp(right))
        });
        let start_offset = pooled_by_start.len();
        pooled_by_start.extend_from_slice(crossing);
        let crossing_by_start = start_offset..pooled_by_start.len();

        crossing.sort_unstable_by(|left, right| {
            interval(bounds[*right].rect, axis)
                .1
                .total_cmp(&interval(bounds[*left].rect, axis).1)
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
            bounds,
            axis,
            nodes,
            pooled_by_start,
            pooled_by_end,
        );
        let right = Self::build(
            right_indices,
            bounds,
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
    bounds: &[IndexedBounds],
    axis: Axis,
    center: f32,
) -> (usize, usize) {
    let mut left_end = 0;
    let mut cursor = 0;
    let mut right_start = indices.len();
    while cursor < right_start {
        let (start, end) = interval(bounds[indices[cursor]].rect, axis);
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
