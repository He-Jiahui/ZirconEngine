use zircon_scene::{RenderHybridGiProbe, ViewportCameraSnapshot};

use super::sphere_visible::sphere_visible;

pub(super) fn hybrid_gi_probe_visible(
    probe: &RenderHybridGiProbe,
    camera: &ViewportCameraSnapshot,
) -> bool {
    sphere_visible(probe.position, probe.radius, camera)
}
