use zircon_scene::{RenderHybridGiTraceRegion, ViewportCameraSnapshot};

use super::sphere_visible::sphere_visible;

pub(super) fn hybrid_gi_trace_region_visible(
    region: &RenderHybridGiTraceRegion,
    camera: &ViewportCameraSnapshot,
) -> bool {
    sphere_visible(region.bounds_center, region.bounds_radius, camera)
}
