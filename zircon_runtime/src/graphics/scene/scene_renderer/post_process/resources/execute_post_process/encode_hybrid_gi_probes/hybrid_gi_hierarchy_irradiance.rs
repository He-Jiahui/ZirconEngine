use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::RenderHybridGiProbe;

use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
use crate::graphics::types::ViewportRenderFrame;

use super::hybrid_gi_budget_weight::hybrid_gi_budget_weight;
use super::runtime_parent_chain::{
    blend_runtime_rgb_lineage_sources, frame_has_scheduled_trace_region_payload,
    gather_runtime_descendant_chain_rgb, gather_runtime_descendant_chain_rgb_without_depth_falloff,
    gather_runtime_parent_chain_rgb, gather_runtime_parent_chain_rgb_without_depth_falloff,
    runtime_irradiance_lineage_has_scene_truth, runtime_parent_topology_is_authoritative,
    runtime_probe_lineage_has_scene_truth,
};
use super::scene_prepare_surface_cache_samples::scene_prepare_surface_cache_fallback_rgb_and_support;

const FARTHER_ANCESTOR_IRRADIANCE_INHERITANCE_FALLOFF: f32 = 0.72;
const IRRADIANCE_INHERITANCE_WEIGHT_SCALE: f32 = 0.5;
const SCENE_PREPARE_SURFACE_CACHE_IRRADIANCE_WEIGHT_SCALE: f32 = 0.58;

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn hybrid_gi_hierarchy_irradiance(
    frame: &ViewportRenderFrame,
    source: &RenderHybridGiProbe,
) -> [f32; 4] {
    hybrid_gi_hierarchy_irradiance_with_scene_prepare_resources(frame, source, None)
}

pub(crate) fn hybrid_gi_hierarchy_irradiance_with_scene_prepare_resources(
    frame: &ViewportRenderFrame,
    source: &RenderHybridGiProbe,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> [f32; 4] {
    let has_scene_prepare = frame.hybrid_gi_scene_prepare.is_some();
    let runtime_probe_scene_truth = runtime_probe_lineage_has_scene_truth(frame, source.probe_id);
    let stripped_runtime_probe_scene_truth = !has_scene_prepare && runtime_probe_scene_truth;
    let scene_driven_frame = has_scene_prepare
        || stripped_runtime_probe_scene_truth
        || runtime_irradiance_lineage_has_scene_truth(frame, source.probe_id);
    let current_trace_schedule_is_empty = !frame_has_scheduled_trace_region_payload(frame);
    let exact_runtime_irradiance = frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .and_then(|runtime| runtime.hierarchy_irradiance(source.probe_id))
        .filter(|runtime_irradiance| runtime_irradiance[3] > f32::EPSILON);
    let exact_runtime_includes_scene_truth = frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .map(|runtime| runtime.hierarchy_irradiance_includes_scene_truth(source.probe_id))
        .unwrap_or(false);
    let exact_scene_truth_runtime_irradiance =
        exact_runtime_irradiance.filter(|_| exact_runtime_includes_scene_truth);
    let exact_continuation_runtime_irradiance =
        exact_runtime_irradiance.filter(|_| !exact_runtime_includes_scene_truth);
    let exact_scene_truth_runtime_irradiance_present =
        exact_scene_truth_runtime_irradiance.is_some();
    let inherited_scene_truth_runtime_irradiance = (!exact_scene_truth_runtime_irradiance_present)
        .then(|| {
            gather_runtime_parent_chain_rgb_without_depth_falloff(
                frame,
                source.probe_id,
                |runtime, ancestor_probe_id| {
                    runtime
                        .hierarchy_irradiance_includes_scene_truth(ancestor_probe_id)
                        .then(|| runtime.hierarchy_irradiance(ancestor_probe_id))
                        .flatten()
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
        })
        .flatten()
        .filter(|runtime_irradiance| runtime_irradiance[3] > f32::EPSILON);
    let inherited_continuation_runtime_irradiance = (!exact_scene_truth_runtime_irradiance_present)
        .then(|| {
            gather_runtime_parent_chain_rgb(frame, source.probe_id, |runtime, ancestor_probe_id| {
                if runtime.hierarchy_irradiance_includes_scene_truth(ancestor_probe_id) {
                    return None;
                }

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
        })
        .flatten()
        .filter(|runtime_irradiance| runtime_irradiance[3] > f32::EPSILON);
    let descendant_scene_truth_runtime_irradiance = (!exact_scene_truth_runtime_irradiance_present)
        .then(|| {
            gather_runtime_descendant_chain_rgb_without_depth_falloff(
                frame,
                source.probe_id,
                |runtime, descendant_probe_id| {
                    if !runtime.hierarchy_irradiance_includes_scene_truth(descendant_probe_id) {
                        return None;
                    }

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
        })
        .flatten()
        .filter(|runtime_irradiance| runtime_irradiance[3] > f32::EPSILON);
    let descendant_continuation_runtime_irradiance =
        (!exact_scene_truth_runtime_irradiance_present)
            .then(|| {
                gather_runtime_descendant_chain_rgb(
                    frame,
                    source.probe_id,
                    |runtime, descendant_probe_id| {
                        if runtime.hierarchy_irradiance_includes_scene_truth(descendant_probe_id) {
                            return None;
                        }

                        runtime.hierarchy_irradiance(descendant_probe_id).map(
                            |hierarchy_irradiance| {
                                (
                                    [
                                        hierarchy_irradiance[0],
                                        hierarchy_irradiance[1],
                                        hierarchy_irradiance[2],
                                    ],
                                    hierarchy_irradiance[3],
                                )
                            },
                        )
                    },
                )
            })
            .flatten()
            .filter(|runtime_irradiance| runtime_irradiance[3] > f32::EPSILON);
    let scene_truth_runtime_irradiance = blend_runtime_rgb_lineage_sources(
        exact_scene_truth_runtime_irradiance,
        inherited_scene_truth_runtime_irradiance,
        descendant_scene_truth_runtime_irradiance,
    );
    let continuation_runtime_irradiance = blend_runtime_rgb_lineage_sources(
        exact_continuation_runtime_irradiance,
        inherited_continuation_runtime_irradiance,
        descendant_continuation_runtime_irradiance,
    );
    let selected_runtime_irradiance_is_scene_truth =
        scene_driven_frame && scene_truth_runtime_irradiance.is_some();
    let selected_runtime_irradiance = if selected_runtime_irradiance_is_scene_truth {
        scene_truth_runtime_irradiance
    } else if runtime_probe_scene_truth && scene_truth_runtime_irradiance.is_none() {
        None
    } else {
        blend_runtime_rgb_lineage_sources(
            scene_truth_runtime_irradiance,
            continuation_runtime_irradiance,
            None,
        )
    };
    if let Some(runtime_irradiance) = selected_runtime_irradiance {
        if current_trace_schedule_is_empty && !selected_runtime_irradiance_is_scene_truth {
            if let Some(scene_prepare_irradiance) = scene_prepare_surface_cache_irradiance_fallback(
                frame,
                source,
                scene_prepare_resources,
            )
            .filter(|scene_prepare_irradiance| scene_prepare_irradiance[3] > f32::EPSILON)
            {
                let total_support = runtime_irradiance[3] + scene_prepare_irradiance[3];
                if total_support > f32::EPSILON {
                    return [
                        (runtime_irradiance[0] * runtime_irradiance[3]
                            + scene_prepare_irradiance[0] * scene_prepare_irradiance[3])
                            / total_support,
                        (runtime_irradiance[1] * runtime_irradiance[3]
                            + scene_prepare_irradiance[1] * scene_prepare_irradiance[3])
                            / total_support,
                        (runtime_irradiance[2] * runtime_irradiance[3]
                            + scene_prepare_irradiance[2] * scene_prepare_irradiance[3])
                            / total_support,
                        total_support.clamp(0.0, 0.75),
                    ];
                }
            }
        }
        return runtime_irradiance;
    }

    if runtime_parent_topology_is_authoritative(frame) {
        return scene_prepare_surface_cache_irradiance_fallback(
            frame,
            source,
            scene_prepare_resources,
        )
        .unwrap_or([0.0; 4]);
    }

    let Some(extract) = frame.extract.lighting.hybrid_global_illumination.as_ref() else {
        return [0.0; 4];
    };

    let probes_by_id = extract
        .probes
        .iter()
        .copied()
        .map(|probe| (probe.probe_id, probe))
        .collect::<BTreeMap<_, _>>();
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

fn scene_prepare_surface_cache_irradiance_fallback(
    frame: &ViewportRenderFrame,
    source: &RenderHybridGiProbe,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> Option<[f32; 4]> {
    let scene_prepare = frame.hybrid_gi_scene_prepare.as_ref()?;
    let (rgb, support) = scene_prepare_surface_cache_fallback_rgb_and_support(
        scene_prepare,
        source.position,
        source.radius,
        scene_prepare_resources,
    )?;
    Some([
        rgb[0],
        rgb[1],
        rgb[2],
        (support * SCENE_PREPARE_SURFACE_CACHE_IRRADIANCE_WEIGHT_SCALE).clamp(0.18, 0.62),
    ])
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::hybrid_gi_hierarchy_irradiance;
    use crate::core::framework::render::{
        FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderHybridGiExtract,
        RenderHybridGiProbe, RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
        RenderWorldSnapshotHandle, ViewportCameraSnapshot,
    };
    use crate::core::math::{UVec2, Vec4};
    use crate::graphics::types::{
        HybridGiPrepareFrame, HybridGiPrepareProbe, HybridGiPrepareSurfaceCachePageContent,
        HybridGiResolveRuntime, HybridGiScenePrepareFrame, ViewportRenderFrame,
    };

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

    #[test]
    fn exact_runtime_irradiance_blends_current_surface_cache_truth_when_trace_schedule_is_empty() {
        let exact_runtime = HybridGiResolveRuntime::pack_rgb_and_weight([0.47, 0.47, 0.47], 0.62);
        let warm = hierarchy_irradiance_with_exact_runtime(
            HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                    page_id: 11,
                    owner_card_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: crate::core::math::Vec3::ZERO,
                    bounds_radius: 0.6,
                    atlas_sample_rgba: [224, 112, 64, 255],
                    capture_sample_rgba: [240, 96, 48, 255],
                }],
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            },
            exact_runtime,
        );
        let cool = hierarchy_irradiance_with_exact_runtime(
            HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                    page_id: 11,
                    owner_card_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: crate::core::math::Vec3::ZERO,
                    bounds_radius: 0.6,
                    atlas_sample_rgba: [64, 112, 224, 255],
                    capture_sample_rgba: [48, 96, 240, 255],
                }],
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            },
            exact_runtime,
        );

        assert!(
            warm[0] > cool[0] + 0.12,
            "expected stale exact runtime irradiance to keep blending with current warm surface-cache truth when there is no current trace schedule, instead of flattening both frames back to the same runtime-only color; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            cool[2] > warm[2] + 0.12,
            "expected stale exact runtime irradiance to keep blending with current cool surface-cache truth when there is no current trace schedule, instead of flattening both frames back to the same runtime-only color; warm={warm:?}, cool={cool:?}"
        );
    }

    #[test]
    fn exact_runtime_irradiance_skips_scene_prepare_reblend_when_runtime_source_is_already_scene_driven(
    ) {
        let exact_runtime = HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58);
        let scene_prepare = HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 11,
                owner_card_id: 11,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: crate::core::math::Vec3::ZERO,
                bounds_radius: 0.6,
                atlas_sample_rgba: [64, 112, 224, 255],
                capture_sample_rgba: [48, 96, 240, 255],
            }],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        };

        let scene_driven = hierarchy_irradiance_with_exact_runtime_and_scene_driven_flag(
            scene_prepare.clone(),
            exact_runtime,
            true,
        );
        let reblended = hierarchy_irradiance_with_exact_runtime_and_scene_driven_flag(
            scene_prepare,
            exact_runtime,
            false,
        );

        assert!(
            scene_driven[0] > reblended[0] + 0.08,
            "expected renderer-side hierarchy irradiance to trust a runtime source that already includes current scene truth instead of blending the same surface-cache signal a second time; scene_driven={scene_driven:?}, reblended={reblended:?}"
        );
        assert!(
            reblended[2] > scene_driven[2] + 0.08,
            "expected unflagged runtime irradiance to keep drifting toward the cool scene-prepare page while the scene-driven runtime source stays closer to its authored warm color; scene_driven={scene_driven:?}, reblended={reblended:?}"
        );
    }

    #[test]
    fn scene_driven_exact_runtime_irradiance_ignores_descendant_lineage_tint() {
        let exact_runtime = HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58);
        let warm = hierarchy_irradiance_with_scene_driven_exact_and_descendant(
            exact_runtime,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.9, 0.32, 0.18], 0.66),
        );
        let cool = hierarchy_irradiance_with_scene_driven_exact_and_descendant(
            exact_runtime,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.18, 0.32, 0.9], 0.66),
        );

        assert!(
            (warm[0] - cool[0]).abs() < 0.03,
            "expected scene-driven exact runtime irradiance to stay anchored to current exact scene truth instead of drifting with descendant lineage tint; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            (warm[2] - cool[2]).abs() < 0.03,
            "expected scene-driven exact runtime irradiance to keep blue output stable while descendant lineage tint changes; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            warm[0] > warm[2] + 0.35,
            "expected scene-driven exact runtime irradiance to keep its authored warm bias after descendant lineage tint changes; warm={warm:?}"
        );
    }

    #[test]
    fn scene_driven_lineage_runtime_irradiance_ignores_scene_prepare_surface_cache_tint() {
        let inherited_warm = hierarchy_irradiance_with_scene_driven_lineage_and_scene_prepare_page(
            true,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58),
            [240, 96, 48, 255],
        );
        let inherited_cool = hierarchy_irradiance_with_scene_driven_lineage_and_scene_prepare_page(
            true,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58),
            [48, 96, 240, 255],
        );
        let descendant_warm = hierarchy_irradiance_with_scene_driven_lineage_and_scene_prepare_page(
            false,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58),
            [240, 96, 48, 255],
        );
        let descendant_cool = hierarchy_irradiance_with_scene_driven_lineage_and_scene_prepare_page(
            false,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58),
            [48, 96, 240, 255],
        );

        assert!(
            (inherited_warm[0] - inherited_cool[0]).abs() < 0.03
                && (inherited_warm[2] - inherited_cool[2]).abs() < 0.03,
            "expected inherited scene-driven runtime irradiance to stay anchored to lineage scene truth instead of drifting with current scene_prepare surface-cache tint; inherited_warm={inherited_warm:?}, inherited_cool={inherited_cool:?}"
        );
        assert!(
            inherited_warm[0] > inherited_warm[2] + 0.35,
            "expected inherited scene-driven runtime irradiance to keep its authored warm bias after scene_prepare page tint changes; inherited_warm={inherited_warm:?}"
        );
        assert!(
            (descendant_warm[0] - descendant_cool[0]).abs() < 0.03
                && (descendant_warm[2] - descendant_cool[2]).abs() < 0.03,
            "expected descendant scene-driven runtime irradiance to stay anchored to lineage scene truth instead of drifting with current scene_prepare surface-cache tint; descendant_warm={descendant_warm:?}, descendant_cool={descendant_cool:?}"
        );
        assert!(
            descendant_warm[0] > descendant_warm[2] + 0.35,
            "expected descendant scene-driven runtime irradiance to keep its authored warm bias after scene_prepare page tint changes; descendant_warm={descendant_warm:?}"
        );
    }

    #[test]
    fn scene_prepare_atlas_only_surface_cache_page_samples_provide_irradiance_fallback_without_runtime_irradiance(
    ) {
        let warm =
            hierarchy_irradiance_with_scene_prepare_page_content([240, 96, 48, 255], [0, 0, 0, 0]);
        let cool =
            hierarchy_irradiance_with_scene_prepare_page_content([48, 96, 240, 255], [0, 0, 0, 0]);

        assert!(
            warm[3] > 0.1,
            "expected atlas-only persisted page truth to provide nonzero irradiance fallback when runtime irradiance is absent; warm={warm:?}"
        );
        assert!(
            warm[0] > cool[0] + 0.2,
            "expected atlas-only persisted page truth to warm the fallback irradiance instead of leaving it black or flat; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            cool[2] > warm[2] + 0.2,
            "expected atlas-only persisted page truth to preserve blue authority in irradiance fallback when runtime irradiance is absent; warm={warm:?}, cool={cool:?}"
        );
    }

    #[test]
    fn scene_prepare_capture_surface_cache_page_samples_stay_preferred_for_irradiance_fallback_without_runtime_irradiance(
    ) {
        let warm = hierarchy_irradiance_with_scene_prepare_page_content(
            [48, 96, 240, 255],
            [240, 96, 48, 255],
        );
        let cool = hierarchy_irradiance_with_scene_prepare_page_content(
            [240, 96, 48, 255],
            [48, 96, 240, 255],
        );

        assert!(
            warm[3] > 0.1,
            "expected capture-side persisted page truth to provide nonzero irradiance fallback when runtime irradiance is absent; warm={warm:?}"
        );
        assert!(
            warm[0] > cool[0] + 0.2,
            "expected capture-side persisted page truth to stay preferred over atlas-side truth in irradiance fallback; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            cool[2] > warm[2] + 0.2,
            "expected capture-side persisted page truth to preserve blue authority over atlas-side truth in irradiance fallback; warm={warm:?}, cool={cool:?}"
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

    fn hierarchy_irradiance_with_scene_prepare_page_content(
        atlas_sample_rgba: [u8; 4],
        capture_sample_rgba: [u8; 4],
    ) -> [f32; 4] {
        let probe = RenderHybridGiProbe {
            probe_id: 200,
            resident: true,
            ray_budget: 128,
            radius: 1.8,
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
            card_budget: 1,
            voxel_budget: 0,
            probes: vec![probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: probe.probe_id,
                    slot: 0,
                    ray_budget: probe.ray_budget,
                    irradiance_rgb: [0, 0, 0],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                    page_id: 11,
                    owner_card_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: crate::core::math::Vec3::ZERO,
                    bounds_radius: 0.6,
                    atlas_sample_rgba,
                    capture_sample_rgba,
                }],
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            }));

        hybrid_gi_hierarchy_irradiance(&frame, &probe)
    }

    fn hierarchy_irradiance_with_exact_runtime(
        scene_prepare: HybridGiScenePrepareFrame,
        exact_runtime: [u8; 4],
    ) -> [f32; 4] {
        hierarchy_irradiance_with_exact_runtime_and_scene_driven_flag(
            scene_prepare,
            exact_runtime,
            false,
        )
    }

    fn hierarchy_irradiance_with_exact_runtime_and_scene_driven_flag(
        scene_prepare: HybridGiScenePrepareFrame,
        exact_runtime: [u8; 4],
        scene_driven: bool,
    ) -> [f32; 4] {
        let probe = RenderHybridGiProbe {
            probe_id: 200,
            resident: true,
            ray_budget: 128,
            radius: 1.8,
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
            card_budget: 1,
            voxel_budget: 0,
            probes: vec![probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: probe.probe_id,
                    slot: 0,
                    ray_budget: probe.ray_budget,
                    irradiance_rgb: [0, 0, 0],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_scene_prepare(Some(scene_prepare))
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                probe_hierarchy_irradiance_rgb_and_weight: BTreeMap::from([(
                    probe.probe_id,
                    exact_runtime,
                )]),
                probe_scene_driven_hierarchy_irradiance_ids: scene_driven
                    .then(|| BTreeSet::from([probe.probe_id]))
                    .unwrap_or_default(),
                ..Default::default()
            }));

        hybrid_gi_hierarchy_irradiance(&frame, &probe)
    }

    fn hierarchy_irradiance_with_scene_driven_exact_and_descendant(
        exact_runtime: [u8; 4],
        descendant_runtime: [u8; 4],
    ) -> [f32; 4] {
        let probe = RenderHybridGiProbe {
            probe_id: 200,
            resident: true,
            ray_budget: 128,
            radius: 1.8,
            ..Default::default()
        };
        let descendant_probe = RenderHybridGiProbe {
            probe_id: 260,
            parent_probe_id: Some(probe.probe_id),
            resident: false,
            ray_budget: 88,
            radius: 1.2,
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
            card_budget: 1,
            voxel_budget: 0,
            probes: vec![probe, descendant_probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: probe.probe_id,
                    slot: 0,
                    ray_budget: probe.ray_budget,
                    irradiance_rgb: [0, 0, 0],
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
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                probe_hierarchy_irradiance_rgb_and_weight: BTreeMap::from([
                    (probe.probe_id, exact_runtime),
                    (descendant_probe.probe_id, descendant_runtime),
                ]),
                probe_scene_driven_hierarchy_irradiance_ids: BTreeSet::from([
                    probe.probe_id,
                    descendant_probe.probe_id,
                ]),
                ..Default::default()
            }));

        hybrid_gi_hierarchy_irradiance(&frame, &probe)
    }

    fn hierarchy_irradiance_with_scene_driven_lineage_and_scene_prepare_page(
        scene_truth_on_ancestor: bool,
        lineage_runtime: [u8; 4],
        page_capture_sample_rgba: [u8; 4],
    ) -> [f32; 4] {
        let parent_probe = RenderHybridGiProbe {
            probe_id: 100,
            resident: true,
            ray_budget: 128,
            radius: 1.8,
            ..Default::default()
        };
        let child_probe = RenderHybridGiProbe {
            probe_id: 200,
            parent_probe_id: Some(parent_probe.probe_id),
            resident: true,
            ray_budget: 96,
            radius: 1.2,
            ..Default::default()
        };
        let encoded_probe = if scene_truth_on_ancestor {
            child_probe
        } else {
            parent_probe
        };
        let runtime_probe_id = if scene_truth_on_ancestor {
            parent_probe.probe_id
        } else {
            child_probe.probe_id
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
            card_budget: 1,
            voxel_budget: 0,
            probes: vec![parent_probe, child_probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: encoded_probe.probe_id,
                    slot: 0,
                    ray_budget: encoded_probe.ray_budget,
                    irradiance_rgb: [0, 0, 0],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                    page_id: 11,
                    owner_card_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: crate::core::math::Vec3::ZERO,
                    bounds_radius: 0.6,
                    atlas_sample_rgba: page_capture_sample_rgba,
                    capture_sample_rgba: page_capture_sample_rgba,
                }],
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                probe_hierarchy_irradiance_rgb_and_weight: BTreeMap::from([(
                    runtime_probe_id,
                    lineage_runtime,
                )]),
                probe_scene_driven_hierarchy_irradiance_ids: BTreeSet::from([runtime_probe_id]),
                ..Default::default()
            }));

        hybrid_gi_hierarchy_irradiance(&frame, &encoded_probe)
    }
}
