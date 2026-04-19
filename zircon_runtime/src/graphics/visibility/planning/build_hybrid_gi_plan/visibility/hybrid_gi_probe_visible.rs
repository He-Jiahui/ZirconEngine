use crate::core::framework::render::{RenderHybridGiProbe, ViewportCameraSnapshot};

use super::sphere_visible::sphere_visible;

pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) fn hybrid_gi_probe_visible(
    probe: &RenderHybridGiProbe,
    camera: &ViewportCameraSnapshot,
) -> bool {
    sphere_visible(probe.position, probe.radius, camera)
}
