use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::RenderHybridGiProbe;

use crate::graphics::types::ViewportRenderFrame;

use super::hybrid_gi_budget_weight::hybrid_gi_budget_weight;
use super::runtime_parent_chain::{
    blend_runtime_rgb_lineage_sources, gather_runtime_descendant_chain_rgb,
    gather_runtime_parent_chain_rgb,
};

const FARTHER_ANCESTOR_IRRADIANCE_INHERITANCE_FALLOFF: f32 = 0.72;
const IRRADIANCE_INHERITANCE_WEIGHT_SCALE: f32 = 0.5;

pub(super) fn hybrid_gi_hierarchy_irradiance(
    frame: &ViewportRenderFrame,
    source: &RenderHybridGiProbe,
) -> [f32; 4] {
    let exact_runtime_irradiance = frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .and_then(|runtime| runtime.hierarchy_irradiance(source.probe_id))
        .filter(|runtime_irradiance| runtime_irradiance[3] > f32::EPSILON);
    let inherited_runtime_irradiance =
        gather_runtime_parent_chain_rgb(frame, source.probe_id, |runtime, ancestor_probe_id| {
            runtime
                .hierarchy_irradiance(ancestor_probe_id)
                .map(|hierarchy_irradiance| {
                    (
                        [
                            hierarchy_irradiance[0],
                            hierarchy_irradiance[1],
                            hierarchy_irradiance[2],
                        ],
                        hierarchy_irradiance[3],
                    )
                })
        })
        .filter(|runtime_irradiance| runtime_irradiance[3] > f32::EPSILON);
    let descendant_runtime_irradiance = gather_runtime_descendant_chain_rgb(
        frame,
        source.probe_id,
        |runtime, descendant_probe_id| {
            runtime
                .hierarchy_irradiance(descendant_probe_id)
                .map(|hierarchy_irradiance| {
                    (
                        [
                            hierarchy_irradiance[0],
                            hierarchy_irradiance[1],
                            hierarchy_irradiance[2],
                        ],
                        hierarchy_irradiance[3],
                    )
                })
        },
    )
    .filter(|runtime_irradiance| runtime_irradiance[3] > f32::EPSILON);
    if let Some(runtime_irradiance) = blend_runtime_rgb_lineage_sources(
        exact_runtime_irradiance,
        inherited_runtime_irradiance,
        descendant_runtime_irradiance,
    ) {
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::hybrid_gi_hierarchy_irradiance;
    use crate::core::framework::render::{
        FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderHybridGiExtract,
        RenderHybridGiProbe, RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
        RenderWorldSnapshotHandle, ViewportCameraSnapshot,
    };
    use crate::core::math::{UVec2, Vec4};
    use crate::graphics::types::{HybridGiResolveRuntime, ViewportRenderFrame};

    #[test]
    fn exact_runtime_irradiance_keeps_blending_with_descendant_continuation() {
        let warm = hierarchy_irradiance_with_descendant(
            HybridGiResolveRuntime::pack_rgb_and_weight([0.95, 0.28, 0.12], 0.68),
        );
        let cool = hierarchy_irradiance_with_descendant(
            HybridGiResolveRuntime::pack_rgb_and_weight([0.12, 0.28, 0.95], 0.68),
        );

        assert!(
            warm[0] > cool[0] + 0.2,
            "expected exact runtime irradiance to keep blending with descendant continuation instead of returning the parent entry unchanged; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            cool[2] > warm[2] + 0.2,
            "expected descendant continuation to affect the blue channel when the child runtime turns cool; warm={warm:?}, cool={cool:?}"
        );
    }

    fn hierarchy_irradiance_with_descendant(descendant_runtime: [u8; 4]) -> [f32; 4] {
        let parent_probe = RenderHybridGiProbe {
            probe_id: 100,
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let child_probe = RenderHybridGiProbe {
            probe_id: 200,
            parent_probe_id: Some(parent_probe.probe_id),
            ray_budget: 88,
            ..Default::default()
        };

        let snapshot = RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        };
        let mut extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            probe_budget: 2,
            trace_budget: 2,
            card_budget: 2,
            voxel_budget: 1,
            probes: vec![parent_probe, child_probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                probe_hierarchy_irradiance_rgb_and_weight: BTreeMap::from([
                    (
                        parent_probe.probe_id,
                        HybridGiResolveRuntime::pack_rgb_and_weight([0.5, 0.5, 0.5], 0.12),
                    ),
                    (child_probe.probe_id, descendant_runtime),
                ]),
                ..Default::default()
            }));

        hybrid_gi_hierarchy_irradiance(&frame, &parent_probe)
    }
}
