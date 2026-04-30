use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
use crate::graphics::types::ViewportRenderFrame;

use super::super::hybrid_gi_probe_source::HybridGiProbeSource;
use super::super::scene_prepare_surface_cache_samples::scene_prepare_surface_cache_fallback_rgb_and_support;

const SCENE_PREPARE_SURFACE_CACHE_IRRADIANCE_WEIGHT_SCALE: f32 = 0.58;

pub(super) fn scene_prepare_surface_cache_irradiance_fallback<S: HybridGiProbeSource + ?Sized>(
    frame: &ViewportRenderFrame,
    source: &S,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> Option<[f32; 4]> {
    let scene_prepare = frame.hybrid_gi_scene_prepare.as_ref()?;
    let (rgb, support) = scene_prepare_surface_cache_fallback_rgb_and_support(
        scene_prepare,
        source.position(),
        source.radius(),
        scene_prepare_resources,
    )?;
    Some([
        rgb[0],
        rgb[1],
        rgb[2],
        (support * SCENE_PREPARE_SURFACE_CACHE_IRRADIANCE_WEIGHT_SCALE).clamp(0.18, 0.62),
    ])
}
