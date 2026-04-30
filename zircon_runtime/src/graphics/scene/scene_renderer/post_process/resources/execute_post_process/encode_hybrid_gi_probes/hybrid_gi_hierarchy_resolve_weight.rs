use std::collections::{BTreeMap, BTreeSet};

use crate::graphics::types::ViewportRenderFrame;

use super::hybrid_gi_budget_weight::hybrid_gi_budget_weight;
use super::hybrid_gi_probe_source::{fallback_probe_sources_by_id, HybridGiProbeSource};
use super::runtime_parent_chain::{
    gather_runtime_descendant_chain_weight, gather_runtime_parent_chain_weight,
    runtime_parent_topology_is_authoritative, runtime_probe_lineage_has_scene_truth,
    runtime_resolve_weight_support,
};

const CHILD_SPECIFICITY_BOOST: f32 = 0.3;
const RESIDENT_CHILD_ATTENUATION: f32 = 0.78;
const FARTHER_ANCESTOR_BUDGET_FALLOFF: f32 = 0.72;
const FARTHER_ANCESTOR_BUDGET_SCALE: f32 = 0.6;
const RUNTIME_RESOLVE_BLEND_MIN: f32 = 0.25;
const RUNTIME_RESOLVE_BLEND_RANGE: f32 = 2.5;

pub(super) fn hybrid_gi_hierarchy_resolve_weight<S: HybridGiProbeSource + ?Sized>(
    frame: &ViewportRenderFrame,
    source: &S,
) -> f32 {
    let source_probe_id = source.probe_id();
    let exact_runtime_weight = frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .and_then(|runtime| runtime.hierarchy_resolve_weight(source_probe_id))
        .filter(|runtime_weight| *runtime_weight > f32::EPSILON);
    let exact_runtime_weight_has_scene_truth = frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .map(|runtime| runtime_probe_has_scene_truth(runtime, source_probe_id))
        .unwrap_or(false);
    if frame.hybrid_gi_scene_prepare.is_some()
        || runtime_probe_lineage_has_scene_truth(frame, source_probe_id)
    {
        return exact_runtime_weight
            .filter(|_| exact_runtime_weight_has_scene_truth)
            .unwrap_or(1.0);
    }

    let inherited_runtime_weight = gather_runtime_parent_chain_weight(frame, source_probe_id)
        .filter(|runtime_weight| *runtime_weight > f32::EPSILON);
    let descendant_runtime_weight = gather_runtime_descendant_chain_weight(frame, source_probe_id)
        .filter(|runtime_weight| *runtime_weight > f32::EPSILON);
    if let Some(runtime_weight) = blend_runtime_lineage_resolve_weights(
        exact_runtime_weight,
        inherited_runtime_weight,
        descendant_runtime_weight,
    ) {
        return runtime_weight;
    }

    if frame.hybrid_gi_scene_prepare.is_some() {
        return 1.0;
    }
    if runtime_parent_topology_is_authoritative(frame) {
        return 1.0;
    }
    if frame.hybrid_gi_resolve_runtime.is_some() {
        return 1.0;
    }

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
    let resident_probe_ids = resident_prepare_by_id
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    if resident_probe_ids.is_empty() {
        return 1.0;
    }

    let probes_by_id =
        fallback_probe_sources_by_id(frame.extract.lighting.hybrid_global_illumination.as_ref());
    if probes_by_id.is_empty() {
        return 1.0;
    }
    let resident_child_count =
        resident_descendant_count(&probes_by_id, &resident_probe_ids, source_probe_id);
    let resident_parent_depth =
        resident_parent_depth(&probes_by_id, &resident_probe_ids, source_probe_id);
    let farther_ancestor_budget_support = farther_resident_ancestor_budget_support(
        &probes_by_id,
        &resident_prepare_by_id,
        source_probe_id,
    );

    let specificity_weight = 1.0 + resident_parent_depth as f32 * CHILD_SPECIFICITY_BOOST;
    let attenuation_weight = if resident_child_count == 0 {
        1.0
    } else {
        RESIDENT_CHILD_ATTENUATION.powi(resident_child_count as i32)
    };
    let lineage_budget_weight =
        1.0 + farther_ancestor_budget_support * FARTHER_ANCESTOR_BUDGET_SCALE;
    (specificity_weight * attenuation_weight * lineage_budget_weight).clamp(0.25, 2.5)
}

fn runtime_probe_has_scene_truth(
    runtime: &crate::graphics::types::HybridGiResolveRuntime,
    probe_id: u32,
) -> bool {
    let has_supported_irradiance = runtime.hierarchy_irradiance_includes_scene_truth(probe_id)
        && runtime
            .hierarchy_irradiance(probe_id)
            .map(|source| source[3] > f32::EPSILON)
            .unwrap_or(false);
    let has_supported_rt_lighting = runtime.hierarchy_rt_lighting_includes_scene_truth(probe_id)
        && (runtime
            .hierarchy_rt_lighting(probe_id)
            .map(|source| source[3] > f32::EPSILON)
            .unwrap_or(false)
            || (runtime.has_probe_rt_lighting(probe_id)
                && runtime_resolve_weight_support(runtime.hierarchy_resolve_weight(probe_id))
                    > f32::EPSILON));
    has_supported_irradiance || has_supported_rt_lighting
}

fn blend_runtime_lineage_resolve_weights(
    exact: Option<f32>,
    inherited: Option<f32>,
    descendant: Option<f32>,
) -> Option<f32> {
    let mut weighted_weight = 0.0_f32;
    let mut total_support = 0.0_f32;
    for weight in [exact, inherited, descendant].into_iter().flatten() {
        let support = runtime_resolve_blend_support(weight);
        if support <= f32::EPSILON {
            continue;
        }
        weighted_weight += weight * support;
        total_support += support;
    }

    if total_support <= f32::EPSILON {
        return None;
    }

    Some((weighted_weight / total_support).clamp(0.25, 2.75))
}

fn runtime_resolve_blend_support(weight: f32) -> f32 {
    ((weight - RUNTIME_RESOLVE_BLEND_MIN) / RUNTIME_RESOLVE_BLEND_RANGE).clamp(0.05, 1.0)
}

fn resident_descendant_count(
    probes_by_id: &BTreeMap<u32, impl HybridGiProbeSource>,
    resident_probe_ids: &BTreeSet<u32>,
    probe_id: u32,
) -> usize {
    let mut count = 0usize;
    let mut stack = probes_by_id
        .values()
        .filter(|probe| probe.parent_probe_id() == Some(probe_id))
        .map(|probe| probe.probe_id())
        .collect::<Vec<_>>();
    let mut visited_probe_ids = BTreeSet::new();

    while let Some(candidate_probe_id) = stack.pop() {
        if !visited_probe_ids.insert(candidate_probe_id) {
            continue;
        }
        if resident_probe_ids.contains(&candidate_probe_id) {
            count += 1;
        }
        stack.extend(
            probes_by_id
                .values()
                .filter(|probe| probe.parent_probe_id() == Some(candidate_probe_id))
                .map(|probe| probe.probe_id()),
        );
    }

    count
}

fn resident_parent_depth(
    probes_by_id: &BTreeMap<u32, impl HybridGiProbeSource>,
    resident_probe_ids: &BTreeSet<u32>,
    probe_id: u32,
) -> usize {
    let mut depth = 0usize;
    let mut current_probe_id = probe_id;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);

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
        if resident_probe_ids.contains(&parent_probe_id) {
            depth += 1;
        }
        current_probe_id = parent_probe_id;
    }

    depth
}

fn farther_resident_ancestor_budget_support<'a>(
    probes_by_id: &BTreeMap<u32, impl HybridGiProbeSource>,
    resident_prepare_by_id: &BTreeMap<u32, &'a crate::graphics::types::HybridGiPrepareProbe>,
    probe_id: u32,
) -> f32 {
    let mut current_probe_id = probe_id;
    let mut resident_ancestor_count = 0usize;
    let mut total_support = 0.0_f32;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);

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
            if resident_ancestor_count > 1 {
                let farther_ancestor_depth = resident_ancestor_count - 2;
                total_support += FARTHER_ANCESTOR_BUDGET_FALLOFF
                    .powi(farther_ancestor_depth as i32)
                    * hybrid_gi_budget_weight(ancestor_prepare.ray_budget);
            }
        }

        current_probe_id = parent_probe_id;
    }

    total_support.clamp(0.0, 1.5)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::hybrid_gi_hierarchy_resolve_weight;
    use crate::core::framework::render::{
        FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderHybridGiExtract,
        RenderHybridGiProbe, RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
        RenderWorldSnapshotHandle, ViewportCameraSnapshot,
    };
    use crate::core::math::{UVec2, Vec4};
    use crate::graphics::types::{
        HybridGiPrepareFrame, HybridGiPrepareProbe, HybridGiResolveRuntime,
        HybridGiScenePrepareFrame, ViewportRenderFrame,
    };

    #[test]
    fn exact_runtime_resolve_weight_keeps_blending_with_descendant_continuation() {
        let strong = hierarchy_resolve_weight_with_descendant(2.4);
        let weak = hierarchy_resolve_weight_with_descendant(0.6);

        assert!(
            strong > weak + 0.25,
            "expected exact runtime resolve weight to keep blending with descendant continuation instead of returning the parent value unchanged; strong={strong:.3}, weak={weak:.3}"
        );
    }

    #[test]
    fn scene_driven_frame_uses_neutral_resolve_weight_without_runtime_authority() {
        let child_weight = scene_driven_hierarchy_resolve_weight_without_runtime_authority();

        assert!(
            (child_weight - 1.0).abs() < 0.05,
            "expected scene-driven frames to stop using authored probe hierarchy as resolve-weight authority once current scene truth is present, instead of keeping child-specific fallback weighting; child_weight={child_weight:.3}"
        );
    }

    #[test]
    fn scene_driven_frame_ignores_resolve_weight_with_stale_scene_truth_flag_without_supported_source(
    ) {
        let stale_flag_weight = scene_driven_hierarchy_resolve_weight_with_stale_scene_truth_flag();

        assert!(
            (stale_flag_weight - 1.0).abs() < 0.05,
            "expected scene-driven frames to reject exact runtime resolve weight when the only scene-truth evidence is a stale flag without supported irradiance/RT source data; stale_flag_weight={stale_flag_weight:.3}"
        );
    }

    #[test]
    fn legacy_resident_parent_depth_breaks_probe_parent_cycles() {
        let probes_by_id = BTreeMap::from([
            (
                100,
                RenderHybridGiProbe {
                    probe_id: 100,
                    parent_probe_id: Some(200),
                    ..Default::default()
                },
            ),
            (
                200,
                RenderHybridGiProbe {
                    probe_id: 200,
                    parent_probe_id: Some(100),
                    ..Default::default()
                },
            ),
        ]);

        assert_eq!(
            super::resident_parent_depth(&probes_by_id, &BTreeSet::from([100, 200]), 100),
            1
        );
    }

    #[test]
    fn legacy_resolve_weight_uses_first_probe_payload_for_duplicate_probe_ids() {
        let weight = hierarchy_resolve_weight_with_duplicate_middle_probe_payloads();

        assert!(
            weight < 1.45,
            "expected legacy hierarchy resolve weight to keep the first RenderHybridGiProbe payload for a duplicate probe id instead of applying a stale grandparent boost; weight={weight:.3}"
        );
    }

    #[test]
    fn flat_runtime_blocks_legacy_resolve_weight_parent_depth_fallback() {
        let weight = hierarchy_resolve_weight_with_flat_runtime_and_legacy_parent_depth();

        assert!(
            (weight - 1.0).abs() < 0.05,
            "expected flat runtime topology to stop legacy RenderHybridGiProbe parent-depth resolve-weight fallback; weight={weight:.3}"
        );
    }

    #[test]
    fn scene_representation_budget_blocks_legacy_resolve_weight_parent_depth_fallback() {
        let weight =
            hierarchy_resolve_weight_with_budgeted_scene_representation_and_legacy_parent_depth();

        assert!(
            (weight - 1.0).abs() < 0.05,
            "expected budgeted scene-representation extracts to stop legacy RenderHybridGiProbe parent-depth resolve-weight fallback; weight={weight:.3}"
        );
    }

    fn hierarchy_resolve_weight_with_descendant(descendant_weight: f32) -> f32 {
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
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            probes: vec![parent_probe, child_probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_parent_probes(BTreeMap::from([(
                        child_probe.probe_id,
                        parent_probe.probe_id,
                    )]))
                    .with_probe_hierarchy_resolve_weight_q8(BTreeMap::from([
                        (
                            parent_probe.probe_id,
                            HybridGiResolveRuntime::pack_resolve_weight_q8(0.6),
                        ),
                        (
                            child_probe.probe_id,
                            HybridGiResolveRuntime::pack_resolve_weight_q8(descendant_weight),
                        ),
                    ]))
                    .build(),
            ));

        hybrid_gi_hierarchy_resolve_weight(&frame, &parent_probe)
    }

    fn hierarchy_resolve_weight_with_duplicate_middle_probe_payloads() -> f32 {
        let grandparent_probe = RenderHybridGiProbe {
            probe_id: 100,
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let middle_probe = RenderHybridGiProbe {
            probe_id: 200,
            parent_probe_id: None,
            resident: true,
            ray_budget: 96,
            ..Default::default()
        };
        let stale_middle_probe = RenderHybridGiProbe {
            parent_probe_id: Some(grandparent_probe.probe_id),
            ..middle_probe
        };
        let child_probe = RenderHybridGiProbe {
            probe_id: 300,
            parent_probe_id: Some(middle_probe.probe_id),
            resident: true,
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
            probe_budget: 3,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            probes: vec![
                grandparent_probe,
                middle_probe,
                stale_middle_probe,
                child_probe,
            ],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: middle_probe.probe_id,
                        slot: 0,
                        ray_budget: middle_probe.ray_budget,
                        irradiance_rgb: [112, 112, 112],
                    },
                    HybridGiPrepareProbe {
                        probe_id: grandparent_probe.probe_id,
                        slot: 1,
                        ray_budget: grandparent_probe.ray_budget,
                        irradiance_rgb: [240, 96, 48],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }));

        hybrid_gi_hierarchy_resolve_weight(&frame, &child_probe)
    }

    fn hierarchy_resolve_weight_with_budgeted_scene_representation_and_legacy_parent_depth() -> f32
    {
        let grandparent_probe = RenderHybridGiProbe {
            probe_id: 100,
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let parent_probe = RenderHybridGiProbe {
            probe_id: 200,
            parent_probe_id: Some(grandparent_probe.probe_id),
            resident: true,
            ray_budget: 96,
            ..Default::default()
        };
        let child_probe = RenderHybridGiProbe {
            probe_id: 300,
            parent_probe_id: Some(parent_probe.probe_id),
            resident: true,
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
            probe_budget: 3,
            trace_budget: 1,
            card_budget: 1,
            voxel_budget: 1,
            probes: vec![grandparent_probe, parent_probe, child_probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: parent_probe.probe_id,
                        slot: 0,
                        ray_budget: parent_probe.ray_budget,
                        irradiance_rgb: [112, 112, 112],
                    },
                    HybridGiPrepareProbe {
                        probe_id: grandparent_probe.probe_id,
                        slot: 1,
                        ray_budget: grandparent_probe.ray_budget,
                        irradiance_rgb: [240, 96, 48],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }));

        hybrid_gi_hierarchy_resolve_weight(&frame, &child_probe)
    }

    fn hierarchy_resolve_weight_with_flat_runtime_and_legacy_parent_depth() -> f32 {
        let grandparent_probe = RenderHybridGiProbe {
            probe_id: 100,
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let parent_probe = RenderHybridGiProbe {
            probe_id: 200,
            parent_probe_id: Some(grandparent_probe.probe_id),
            resident: true,
            ray_budget: 96,
            ..Default::default()
        };
        let child_probe = RenderHybridGiProbe {
            probe_id: 300,
            parent_probe_id: Some(parent_probe.probe_id),
            resident: true,
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
            probe_budget: 3,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            probes: vec![grandparent_probe, parent_probe, child_probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: parent_probe.probe_id,
                        slot: 0,
                        ray_budget: parent_probe.ray_budget,
                        irradiance_rgb: [112, 112, 112],
                    },
                    HybridGiPrepareProbe {
                        probe_id: grandparent_probe.probe_id,
                        slot: 1,
                        ray_budget: grandparent_probe.ray_budget,
                        irradiance_rgb: [240, 96, 48],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime::default()));

        hybrid_gi_hierarchy_resolve_weight(&frame, &child_probe)
    }

    fn scene_driven_hierarchy_resolve_weight_without_runtime_authority() -> f32 {
        let parent_probe = RenderHybridGiProbe {
            probe_id: 100,
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let child_probe = RenderHybridGiProbe {
            probe_id: 200,
            parent_probe_id: Some(parent_probe.probe_id),
            resident: true,
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
            trace_budget: 0,
            card_budget: 2,
            voxel_budget: 1,
            probes: vec![parent_probe, child_probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: parent_probe.probe_id,
                        slot: 0,
                        ray_budget: parent_probe.ray_budget,
                        irradiance_rgb: [160, 160, 160],
                    },
                    HybridGiPrepareProbe {
                        probe_id: child_probe.probe_id,
                        slot: 1,
                        ray_budget: child_probe.ray_budget,
                        irradiance_rgb: [160, 160, 160],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: Vec::new(),
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            }));

        hybrid_gi_hierarchy_resolve_weight(&frame, &child_probe)
    }

    fn scene_driven_hierarchy_resolve_weight_with_stale_scene_truth_flag() -> f32 {
        let probe = RenderHybridGiProbe {
            probe_id: 100,
            resident: true,
            ray_budget: 128,
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
            probe_budget: 1,
            trace_budget: 0,
            card_budget: 2,
            voxel_budget: 1,
            probes: vec![probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: probe.probe_id,
                    slot: 0,
                    ray_budget: probe.ray_budget,
                    irradiance_rgb: [160, 160, 160],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: Vec::new(),
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_resolve_weight_q8(BTreeMap::from([(
                        probe.probe_id,
                        HybridGiResolveRuntime::pack_resolve_weight_q8(2.4),
                    )]))
                    .with_probe_scene_driven_hierarchy_irradiance_ids(BTreeSet::from([
                        probe.probe_id
                    ]))
                    .build(),
            ));

        hybrid_gi_hierarchy_resolve_weight(&frame, &probe)
    }
}
