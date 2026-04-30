use std::collections::{BTreeMap, BTreeSet};

use crate::graphics::hybrid_gi_extract_sources::{
    enabled_hybrid_gi_extract, hybrid_gi_extract_uses_scene_representation_budget,
};
use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
use crate::graphics::types::ViewportRenderFrame;

use super::super::hybrid_gi_budget_weight::hybrid_gi_budget_weight;
use super::super::hybrid_gi_probe_source::{fallback_probe_sources_by_id, HybridGiProbeSource};
use super::super::runtime_parent_chain::runtime_parent_topology_is_authoritative;
use super::scene_prepare_irradiance_fallback::scene_prepare_surface_cache_irradiance_fallback;

const FARTHER_ANCESTOR_IRRADIANCE_INHERITANCE_FALLOFF: f32 = 0.72;
const IRRADIANCE_INHERITANCE_WEIGHT_SCALE: f32 = 0.5;

pub(super) fn ancestor_prepare_irradiance_fallback<S: HybridGiProbeSource + ?Sized>(
    frame: &ViewportRenderFrame,
    source: &S,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> [f32; 4] {
    if runtime_parent_topology_is_authoritative(frame) {
        return scene_prepare_surface_cache_irradiance_fallback(
            frame,
            source,
            scene_prepare_resources,
        )
        .unwrap_or([0.0; 4]);
    }
    if frame.hybrid_gi_resolve_runtime.is_some() {
        return scene_prepare_surface_cache_irradiance_fallback(
            frame,
            source,
            scene_prepare_resources,
        )
        .unwrap_or([0.0; 4]);
    }

    let Some(extract) =
        enabled_hybrid_gi_extract(frame.extract.lighting.hybrid_global_illumination.as_ref())
    else {
        return [0.0; 4];
    };
    if hybrid_gi_extract_uses_scene_representation_budget(extract) {
        return scene_prepare_surface_cache_irradiance_fallback(
            frame,
            source,
            scene_prepare_resources,
        )
        .unwrap_or([0.0; 4]);
    }

    let probes_by_id = fallback_probe_sources_by_id(Some(extract));
    let resident_prepare_by_id = frame
        .hybrid_gi_prepare
        .as_ref()
        .map(|prepare| {
            prepare
                .resident_probes
                .iter()
                .map(|probe| (probe.probe_id, probe))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    let mut current_probe_id = source.probe_id();
    let mut visited_probe_ids = BTreeSet::from([source.probe_id()]);
    let mut resident_ancestor_count = 0usize;

    loop {
        let Some(parent_probe_id) = probes_by_id
            .get(&current_probe_id)
            .and_then(|probe| probe.parent_probe_id())
        else {
            break;
        };
        if !visited_probe_ids.insert(parent_probe_id) {
            break;
        }
        if let Some(ancestor_prepare) = resident_prepare_by_id.get(&parent_probe_id) {
            resident_ancestor_count += 1;
            // Keep the existing parent/child resolve behavior intact and only use
            // irradiance continuation for farther resident ancestors beyond the first one.
            if resident_ancestor_count > 1 {
                let farther_ancestor_depth = resident_ancestor_count - 2;
                let hierarchy_weight = FARTHER_ANCESTOR_IRRADIANCE_INHERITANCE_FALLOFF
                    .powi(farther_ancestor_depth as i32);
                let support =
                    hierarchy_weight * hybrid_gi_budget_weight(ancestor_prepare.ray_budget);
                if support > 0.0 {
                    weighted_rgb[0] +=
                        (ancestor_prepare.irradiance_rgb[0] as f32 / 255.0) * support;
                    weighted_rgb[1] +=
                        (ancestor_prepare.irradiance_rgb[1] as f32 / 255.0) * support;
                    weighted_rgb[2] +=
                        (ancestor_prepare.irradiance_rgb[2] as f32 / 255.0) * support;
                    total_support += support;
                }
            }
        }

        current_probe_id = parent_probe_id;
    }

    if total_support <= f32::EPSILON {
        return scene_prepare_surface_cache_irradiance_fallback(
            frame,
            source,
            scene_prepare_resources,
        )
        .unwrap_or([0.0; 4]);
    }

    let inherited_weight = (total_support * IRRADIANCE_INHERITANCE_WEIGHT_SCALE).clamp(0.0, 0.75);
    [
        weighted_rgb[0] / total_support,
        weighted_rgb[1] / total_support,
        weighted_rgb[2] / total_support,
        inherited_weight,
    ]
}
