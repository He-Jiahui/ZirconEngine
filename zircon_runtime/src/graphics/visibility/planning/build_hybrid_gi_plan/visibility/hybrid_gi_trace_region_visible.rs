use crate::core::framework::render::ViewportCameraSnapshot;

use super::super::sources::HybridGiVisibilityPlanTraceRegion;
use super::sphere_visible::sphere_visible;

pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) fn hybrid_gi_trace_region_visible(
    region: &HybridGiVisibilityPlanTraceRegion,
    camera: &ViewportCameraSnapshot,
) -> bool {
    sphere_visible(region.bounds_center, region.bounds_radius, camera)
}
