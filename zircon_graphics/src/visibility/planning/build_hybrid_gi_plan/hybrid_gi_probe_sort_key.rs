use std::cmp::Ordering;

use zircon_scene::RenderHybridGiProbe;

pub(super) fn hybrid_gi_probe_sort_key(
    left: &RenderHybridGiProbe,
    right: &RenderHybridGiProbe,
) -> Ordering {
    right
        .ray_budget
        .cmp(&left.ray_budget)
        .then_with(|| left.probe_id.cmp(&right.probe_id))
}
