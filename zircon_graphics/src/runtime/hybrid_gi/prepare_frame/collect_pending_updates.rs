use crate::types::HybridGiPrepareUpdateRequest;

use super::super::HybridGiRuntimeState;

pub(super) fn collect_pending_updates(
    runtime: &HybridGiRuntimeState,
) -> Vec<HybridGiPrepareUpdateRequest> {
    runtime
        .pending_updates
        .iter()
        .map(|update| HybridGiPrepareUpdateRequest {
            probe_id: update.probe_id,
            ray_budget: update.ray_budget,
            generation: update.generation,
        })
        .collect()
}
