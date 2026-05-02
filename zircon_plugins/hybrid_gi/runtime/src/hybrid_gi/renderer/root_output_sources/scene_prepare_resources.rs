use crate::hybrid_gi::renderer::{HybridGiGpuReadback, HybridGiScenePrepareResourcesSnapshot};

pub(in crate::hybrid_gi::renderer) fn hybrid_gi_scene_prepare_resources(
    readback: Option<&HybridGiGpuReadback>,
) -> Option<HybridGiScenePrepareResourcesSnapshot> {
    readback.and_then(HybridGiGpuReadback::scene_prepare_resources)
}
