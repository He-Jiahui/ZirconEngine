use super::*;
use crate::hybrid_gi::HybridGiProbeUpdateRequest;

#[test]
fn cached_pending_update_sort_preserves_priority_order() {
    let mut runtime = HybridGiRuntimeState::default();
    runtime.push_pending_update_request(HybridGiProbeUpdateRequest::new(30, 3, 2));
    runtime.push_pending_update_request(HybridGiProbeUpdateRequest::new(20, 2, 1));
    runtime.push_pending_update_request(HybridGiProbeUpdateRequest::new(10, 1, 1));

    let pending_updates = collect_pending_updates(&runtime);
    let projected = pending_updates
        .iter()
        .map(|update| (update.probe_id, update.ray_budget, update.generation))
        .collect::<Vec<_>>();

    assert_eq!(projected, vec![(10, 1, 1), (20, 2, 1), (30, 3, 2)]);
}
