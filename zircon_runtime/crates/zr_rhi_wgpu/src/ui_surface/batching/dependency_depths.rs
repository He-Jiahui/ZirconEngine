use super::{bounds_index::BoundsIndex, DrawItem};

pub(super) fn dependency_depths(items: &[DrawItem]) -> (Vec<usize>, usize, usize, usize) {
    if items.is_empty() {
        return (Vec::new(), 0, 0, 0);
    }

    let index = BoundsIndex::new(
        items
            .iter()
            .enumerate()
            .map(|(item_index, item)| (item_index, item.rect())),
    );
    let mut depths = vec![0; items.len()];
    let mut dependency_count = 0;
    let mut overlap_candidate_count = 0;

    // Items are already in painter order. Querying only earlier indices lets us
    // update the longest dependency depth without retaining every overlap edge.
    for later_index in 0..items.len() {
        let later_rect = items[later_index].rect();
        index.query_interval_candidates(later_rect, |earlier_index| {
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

fn rects_intersect(left: zr_rhi::UiSurfaceRect, right: zr_rhi::UiSurfaceRect) -> bool {
    let left_right = left.x + left.width;
    let left_bottom = left.y + left.height;
    let right_right = right.x + right.width;
    let right_bottom = right.y + right.height;
    left.x < right_right && right.x < left_right && left.y < right_bottom && right.y < left_bottom
}

#[cfg(test)]
pub(super) fn interval_index_storage_counts(items: &[DrawItem]) -> (usize, usize, usize) {
    BoundsIndex::new(
        items
            .iter()
            .enumerate()
            .map(|(item_index, item)| (item_index, item.rect())),
    )
    .storage_counts()
}
