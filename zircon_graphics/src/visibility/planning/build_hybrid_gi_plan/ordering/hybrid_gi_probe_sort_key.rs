use std::cmp::Ordering;

use zircon_framework::render::RenderHybridGiProbe;

pub(in crate::visibility::planning::build_hybrid_gi_plan) fn hybrid_gi_probe_sort_key(
    left: &RenderHybridGiProbe,
    right: &RenderHybridGiProbe,
) -> Ordering {
    right
        .ray_budget
        .cmp(&left.ray_budget)
        .then_with(|| left.probe_id.cmp(&right.probe_id))
}
