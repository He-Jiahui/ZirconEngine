use std::cmp::Ordering;

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
    root: Option<Box<IntervalNode>>,
}

impl<'a> IntervalIndex<'a> {
    fn new(items: &'a [DrawItem], axis: Axis) -> Self {
        let indices = (0..items.len()).collect();
        Self {
            items,
            axis,
            root: IntervalNode::build(indices, items, axis),
        }
    }

    fn query(&self, rect: UiSurfaceRect, mut visit: impl FnMut(usize)) {
        if let Some(root) = &self.root {
            root.query(rect, self.items, self.axis, &mut visit);
        }
    }
}

struct IntervalNode {
    center: f32,
    crossing_by_start: Vec<usize>,
    crossing_by_end: Vec<usize>,
    left: Option<Box<IntervalNode>>,
    right: Option<Box<IntervalNode>>,
}

impl IntervalNode {
    fn build(mut indices: Vec<usize>, items: &[DrawItem], axis: Axis) -> Option<Box<Self>> {
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
        let mut left_indices = Vec::new();
        let mut right_indices = Vec::new();
        let mut crossing = Vec::new();

        for index in indices {
            let (start, end) = interval(items[index].rect(), axis);
            if end <= center {
                left_indices.push(index);
            } else if start >= center {
                right_indices.push(index);
            } else {
                crossing.push(index);
            }
        }

        let mut crossing_by_start = crossing;
        crossing_by_start.sort_unstable_by(|left, right| {
            interval(items[*left].rect(), axis)
                .0
                .total_cmp(&interval(items[*right].rect(), axis).0)
                .then_with(|| left.cmp(right))
        });
        let mut crossing_by_end = crossing_by_start.clone();
        crossing_by_end.sort_unstable_by(|left, right| {
            interval(items[*right].rect(), axis)
                .1
                .total_cmp(&interval(items[*left].rect(), axis).1)
                .then_with(|| left.cmp(right))
        });

        Some(Box::new(Self {
            center,
            crossing_by_start,
            crossing_by_end,
            left: Self::build(left_indices, items, axis),
            right: Self::build(right_indices, items, axis),
        }))
    }

    fn query(
        &self,
        rect: UiSurfaceRect,
        items: &[DrawItem],
        axis: Axis,
        visit: &mut impl FnMut(usize),
    ) {
        let (start, end) = interval(rect, axis);
        match (end.total_cmp(&self.center), start.total_cmp(&self.center)) {
            (Ordering::Less | Ordering::Equal, _) => {
                for index in &self.crossing_by_start {
                    if interval(items[*index].rect(), axis).0 >= end {
                        break;
                    }
                    visit(*index);
                }
                if let Some(left) = &self.left {
                    left.query(rect, items, axis, visit);
                }
            }
            (_, Ordering::Greater | Ordering::Equal) => {
                for index in &self.crossing_by_end {
                    if interval(items[*index].rect(), axis).1 <= start {
                        break;
                    }
                    visit(*index);
                }
                if let Some(right) = &self.right {
                    right.query(rect, items, axis, visit);
                }
            }
            _ => {
                self.crossing_by_start.iter().copied().for_each(&mut *visit);
                if let Some(left) = &self.left {
                    left.query(rect, items, axis, visit);
                }
                if let Some(right) = &self.right {
                    right.query(rect, items, axis, visit);
                }
            }
        }
    }
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
