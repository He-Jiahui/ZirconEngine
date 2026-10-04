use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};

use zircon_runtime_interface::ui::surface::{
    UiHitTestCell, UiPersistentSequence, UiPersistentSequenceCowStats,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct UiCellMembershipPatchStats {
    pub(super) staged_cell_count: usize,
    pub(super) published_cell_count: usize,
    pub(super) source_membership_count: usize,
    pub(super) materialized_membership_count: usize,
    pub(super) replacement_buffer_count: usize,
    pub(super) staged_removal_count: usize,
    pub(super) staged_addition_count: usize,
    pub(super) cell_cow_stats: UiPersistentSequenceCowStats,
}

#[derive(Debug, Default)]
pub(super) struct UiCellMembershipPatches {
    by_cell: BTreeMap<usize, UiCellMembershipDelta>,
}

#[derive(Debug, Default)]
struct UiCellMembershipDelta {
    removals: Vec<usize>,
    additions: Vec<usize>,
}

impl UiCellMembershipPatches {
    pub(super) fn stage(
        &mut self,
        entry_index: usize,
        previous_cells: &[usize],
        next_cells: &[usize],
        reinsert_common_cells: bool,
    ) {
        let mut previous = previous_cells.iter().copied().peekable();
        let mut next = next_cells.iter().copied().peekable();
        loop {
            match (previous.peek().copied(), next.peek().copied()) {
                (Some(left), Some(right)) if left == right => {
                    if reinsert_common_cells {
                        let delta = self.by_cell.entry(left).or_default();
                        delta.removals.push(entry_index);
                        delta.additions.push(entry_index);
                    }
                    previous.next();
                    next.next();
                }
                (Some(left), Some(right)) if left < right => {
                    self.by_cell
                        .entry(left)
                        .or_default()
                        .removals
                        .push(entry_index);
                    previous.next();
                }
                (Some(_), Some(right)) => {
                    self.by_cell
                        .entry(right)
                        .or_default()
                        .additions
                        .push(entry_index);
                    next.next();
                }
                (Some(left), None) => {
                    self.by_cell
                        .entry(left)
                        .or_default()
                        .removals
                        .push(entry_index);
                    previous.next();
                }
                (None, Some(right)) => {
                    self.by_cell
                        .entry(right)
                        .or_default()
                        .additions
                        .push(entry_index);
                    next.next();
                }
                (None, None) => break,
            }
        }
    }

    /// Builds every replacement before publishing any cell, preserving patch atomicity.
    pub(super) fn apply<K: Ord>(
        self,
        cells: &mut UiPersistentSequence<UiHitTestCell>,
        mut sort_key: impl FnMut(usize) -> K,
    ) -> Result<UiCellMembershipPatchStats, ()> {
        let mut stats = UiCellMembershipPatchStats {
            staged_cell_count: self.by_cell.len(),
            ..UiCellMembershipPatchStats::default()
        };
        let mut replacements = Vec::with_capacity(self.by_cell.len());

        for (cell_index, mut delta) in self.by_cell {
            let current = cells.get(cell_index).ok_or(())?.entries.as_slice();
            delta.removals.sort_unstable();
            delta.removals.dedup();
            delta.additions.sort_unstable();
            delta.additions.dedup();
            delta
                .additions
                .sort_by(|left, right| sort_key(*left).cmp(&sort_key(*right)));

            stats.source_membership_count =
                stats.source_membership_count.saturating_add(current.len());
            stats.staged_removal_count = stats
                .staged_removal_count
                .saturating_add(delta.removals.len());
            stats.staged_addition_count = stats
                .staged_addition_count
                .saturating_add(delta.additions.len());

            let retained = retain_without_sorted_memberships(current, &delta.removals);
            stats.materialized_membership_count = stats
                .materialized_membership_count
                .saturating_add(retained.len());
            stats.replacement_buffer_count += usize::from(retained.capacity() > 0);
            let next = merge_ordered_memberships(retained, delta.additions, &mut sort_key);
            stats.materialized_membership_count = stats
                .materialized_membership_count
                .saturating_add(next.len());
            stats.replacement_buffer_count += usize::from(next.capacity() > 0);
            if next.as_slice() != current {
                replacements.push((cell_index, next));
            }
        }

        stats.published_cell_count = replacements.len();
        for (cell_index, entries) in replacements {
            let (cell, cow_stats) = cells
                .get_mut_with_stats(cell_index)
                .expect("cell membership replacement was fully preflighted");
            stats.cell_cow_stats.accumulate(cow_stats);
            cell.entries = entries.into();
        }
        Ok(stats)
    }
}

fn retain_without_sorted_memberships(current: &[usize], removals: &[usize]) -> Vec<usize> {
    let mut retained = Vec::with_capacity(current.len().saturating_sub(removals.len()));
    match removals {
        [] => retained.extend_from_slice(current),
        [removed] => retained.extend(
            current
                .iter()
                .copied()
                .filter(|entry_index| entry_index != removed),
        ),
        _ => {
            let removals = removals.iter().copied().collect::<HashSet<_>>();
            retained.extend(
                current
                    .iter()
                    .copied()
                    .filter(|entry_index| !removals.contains(entry_index)),
            );
        }
    }
    retained
}

fn merge_ordered_memberships<K: Ord>(
    retained: Vec<usize>,
    additions: Vec<usize>,
    sort_key: &mut impl FnMut(usize) -> K,
) -> Vec<usize> {
    let mut merged = Vec::with_capacity(retained.len().saturating_add(additions.len()));
    let mut retained = retained.into_iter().peekable();
    let mut additions = additions.into_iter().peekable();
    loop {
        match (retained.peek().copied(), additions.peek().copied()) {
            (Some(left), Some(right)) => {
                if sort_key(left).cmp(&sort_key(right)) != Ordering::Greater {
                    merged.push(retained.next().expect("peeked retained membership"));
                } else {
                    merged.push(additions.next().expect("peeked added membership"));
                }
            }
            (Some(_), None) => {
                merged.extend(retained);
                break;
            }
            (None, Some(_)) => {
                merged.extend(additions);
                break;
            }
            (None, None) => break,
        }
    }
    merged
}

#[cfg(test)]
#[path = "tests/cell_membership_patch.rs"]
mod tests;
