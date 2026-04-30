use std::cmp::Ordering;

use super::super::sources::HybridGiVisibilityPlanProbe;

pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) fn hybrid_gi_probe_sort_key(
    left: &HybridGiVisibilityPlanProbe,
    right: &HybridGiVisibilityPlanProbe,
) -> Ordering {
    right
        .ray_budget
        .cmp(&left.ray_budget)
        .then_with(|| left.probe_id.cmp(&right.probe_id))
}
