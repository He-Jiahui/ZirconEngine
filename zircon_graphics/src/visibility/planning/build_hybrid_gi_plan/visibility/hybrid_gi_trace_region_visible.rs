use zircon_framework::render::{RenderHybridGiTraceRegion, ViewportCameraSnapshot};

use super::sphere_visible::sphere_visible;

pub(in crate::visibility::planning::build_hybrid_gi_plan) fn hybrid_gi_trace_region_visible(
    region: &RenderHybridGiTraceRegion,
    camera: &ViewportCameraSnapshot,
) -> bool {
    sphere_visible(region.bounds_center, region.bounds_radius, camera)
}
