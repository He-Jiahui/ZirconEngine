use std::collections::HashMap;

use crate::graphics::types::VirtualGeometryPrepareClusterState;

#[derive(Default)]
pub(super) struct SeedBackedFrontierRanking {
    unresolved_page_rank_by_page: HashMap<u32, u32>,
    next_unresolved_page_rank: u32,
}

pub(super) fn seed_backed_frontier_rank_for_cluster(
    page_id: u32,
    state: VirtualGeometryPrepareClusterState,
    frontier_ranking: &mut SeedBackedFrontierRanking,
) -> u32 {
    if matches!(state, VirtualGeometryPrepareClusterState::Resident) {
        return 0;
    }

    *frontier_ranking
        .unresolved_page_rank_by_page
        .entry(page_id)
        .or_insert_with(|| {
            let rank = frontier_ranking.next_unresolved_page_rank;
            frontier_ranking.next_unresolved_page_rank =
                frontier_ranking.next_unresolved_page_rank.saturating_add(1);
            rank
        })
}
