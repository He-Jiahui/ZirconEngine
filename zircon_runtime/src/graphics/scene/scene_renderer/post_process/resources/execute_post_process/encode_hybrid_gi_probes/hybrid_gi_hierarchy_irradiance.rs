use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::RenderHybridGiProbe;

use crate::graphics::types::EditorOrRuntimeFrame;

use super::hybrid_gi_budget_weight::hybrid_gi_budget_weight;

const FARTHER_ANCESTOR_IRRADIANCE_INHERITANCE_FALLOFF: f32 = 0.72;
const IRRADIANCE_INHERITANCE_WEIGHT_SCALE: f32 = 0.5;

pub(super) fn hybrid_gi_hierarchy_irradiance(
    frame: &EditorOrRuntimeFrame,
    source: &RenderHybridGiProbe,
) -> [f32; 4] {
    if let Some(runtime_irradiance) = frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .and_then(|runtime| runtime.hierarchy_irradiance(source.probe_id))
    {
        return runtime_irradiance;
    }

    let Some(prepare) = frame.hybrid_gi_prepare.as_ref() else {
        return [0.0; 4];
    };
    let Some(extract) = frame.extract.lighting.hybrid_global_illumination.as_ref() else {
        return [0.0; 4];
    };

    let probes_by_id = extract
        .probes
        .iter()
        .copied()
        .map(|probe| (probe.probe_id, probe))
        .collect::<BTreeMap<_, _>>();
    let resident_prepare_by_id = prepare
        .resident_probes
        .iter()
        .map(|probe| (probe.probe_id, probe))
        .collect::<BTreeMap<_, _>>();

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    let mut current_probe_id = source.probe_id;
    let mut visited_probe_ids = BTreeSet::from([source.probe_id]);
    let mut resident_ancestor_count = 0usize;

    loop {
        let Some(parent_probe_id) = probes_by_id
            .get(&current_probe_id)
            .and_then(|probe| probe.parent_probe_id)
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
        return [0.0; 4];
    }

    let inherited_weight = (total_support * IRRADIANCE_INHERITANCE_WEIGHT_SCALE).clamp(0.0, 0.75);
    [
        weighted_rgb[0] / total_support,
        weighted_rgb[1] / total_support,
        weighted_rgb[2] / total_support,
        inherited_weight,
    ]
}
