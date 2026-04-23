use crate::core::math::UVec2;
use bytemuck::Zeroable;

use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::constants::MAX_HYBRID_GI_PROBES;
use super::super::super::super::hybrid_gi_probe_gpu::GpuHybridGiProbe;
use super::count_scheduled_trace_regions::count_scheduled_trace_regions;
use super::encode_hybrid_gi_probe_screen_data::{
    encode_hybrid_gi_probe_screen_data, encode_hybrid_gi_scene_driven_probe_screen_data,
};
use super::hybrid_gi_hierarchy_irradiance::hybrid_gi_hierarchy_irradiance_with_scene_prepare_resources;
use super::hybrid_gi_hierarchy_resolve_weight::hybrid_gi_hierarchy_resolve_weight;
use super::hybrid_gi_hierarchy_rt_lighting::hybrid_gi_hierarchy_rt_lighting_with_scene_prepare_resources;
use super::hybrid_gi_temporal_signature::{
    hybrid_gi_temporal_scene_truth_confidence, hybrid_gi_temporal_signature,
};

pub(in super::super) fn encode_hybrid_gi_probes(
    frame: &ViewportRenderFrame,
    viewport_size: UVec2,
    enabled: bool,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> ([GpuHybridGiProbe; MAX_HYBRID_GI_PROBES], u32, u32) {
    let mut probes = [GpuHybridGiProbe::zeroed(); MAX_HYBRID_GI_PROBES];
    if !enabled {
        return (probes, 0, 0);
    }

    let Some(prepare) = frame.hybrid_gi_prepare.as_ref() else {
        return (probes, 0, 0);
    };
    let scene_driven_frame = frame.hybrid_gi_scene_prepare.is_some();
    let hybrid_gi_extract = frame.extract.lighting.hybrid_global_illumination.as_ref();

    let mut count = 0;
    for probe in prepare.resident_probes.iter().take(MAX_HYBRID_GI_PROBES) {
        let source = hybrid_gi_extract.and_then(|extract| {
            extract
                .probes
                .iter()
                .find(|candidate| candidate.probe_id == probe.probe_id)
        });
        if scene_driven_frame && source.is_none() {
            continue;
        }
        let (screen_data, hierarchy_weight, hierarchy_irradiance, hierarchy_rt_lighting) = source
            .map(|source| {
                (
                    if scene_driven_frame {
                        frame.hybrid_gi_scene_prepare.as_ref().map_or_else(
                            || {
                                encode_hybrid_gi_probe_screen_data(
                                    &frame.extract,
                                    viewport_size,
                                    source,
                                )
                            },
                            |scene_prepare| {
                                encode_hybrid_gi_scene_driven_probe_screen_data(
                                    &frame.extract,
                                    viewport_size,
                                    scene_prepare,
                                    source.ray_budget,
                                )
                            },
                        )
                    } else {
                        encode_hybrid_gi_probe_screen_data(&frame.extract, viewport_size, source)
                    },
                    hybrid_gi_hierarchy_resolve_weight(frame, source),
                    hybrid_gi_hierarchy_irradiance_with_scene_prepare_resources(
                        frame,
                        source,
                        scene_prepare_resources,
                    ),
                    hybrid_gi_hierarchy_rt_lighting_with_scene_prepare_resources(
                        frame,
                        source,
                        scene_prepare_resources,
                    ),
                )
            })
            .unwrap_or(([0.5, 0.5, 1.0, 1.0], 1.0, [0.0; 4], [0.0; 4]));
        let temporal_signature = hybrid_gi_temporal_signature(
            frame,
            probe.probe_id,
            source.and_then(|probe| probe.parent_probe_id),
            source,
            scene_prepare_resources,
        );
        let temporal_scene_truth_confidence = hybrid_gi_temporal_scene_truth_confidence(
            frame,
            probe.probe_id,
            source,
            scene_prepare_resources,
        );
        probes[count] = GpuHybridGiProbe {
            screen_uv_and_radius: screen_data,
            irradiance_and_intensity: [
                if scene_driven_frame {
                    0.0
                } else {
                    probe.irradiance_rgb[0] as f32 / 255.0
                },
                if scene_driven_frame {
                    0.0
                } else {
                    probe.irradiance_rgb[1] as f32 / 255.0
                },
                if scene_driven_frame {
                    0.0
                } else {
                    probe.irradiance_rgb[2] as f32 / 255.0
                },
                hierarchy_weight,
            ],
            hierarchy_irradiance_rgb_and_weight: hierarchy_irradiance,
            hierarchy_rt_lighting_rgb_and_weight: hierarchy_rt_lighting,
            temporal_signature_and_padding: [
                temporal_signature,
                temporal_scene_truth_confidence,
                0.0,
                0.0,
            ],
        };
        count += 1;
    }

    (probes, count as u32, count_scheduled_trace_regions(frame))
}

#[cfg(test)]
mod tests {
    use super::encode_hybrid_gi_probes;
    use crate::core::framework::render::{
        FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderHybridGiExtract,
        RenderHybridGiProbe, RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
        RenderWorldSnapshotHandle, ViewportCameraSnapshot,
    };
    use crate::core::math::{UVec2, Vec3, Vec4};
    use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
    use crate::graphics::types::{
        HybridGiPrepareCardCaptureRequest, HybridGiPrepareFrame, HybridGiPrepareProbe,
        HybridGiPrepareSurfaceCachePageContent, HybridGiResolveRuntime, HybridGiScenePrepareFrame,
        ViewportRenderFrame,
    };

    #[test]
    fn encode_hybrid_gi_probes_uses_atlas_only_scene_prepare_card_capture_resources_for_hierarchy_irradiance(
    ) {
        let warm = encode_probe_hierarchy_irradiance_with_scene_prepare_resources(
            [240, 96, 48, 255],
            [0, 0, 0, 0],
        );
        let cool = encode_probe_hierarchy_irradiance_with_scene_prepare_resources(
            [48, 96, 240, 255],
            [0, 0, 0, 0],
        );

        assert!(
            warm[3] > 0.1,
            "expected atlas-only current-frame card-capture resources to provide nonzero hierarchy irradiance fallback during encode; warm={warm:?}"
        );
        assert!(
            warm[0] > cool[0] + 0.2,
            "expected atlas-only current-frame card-capture resources to warm encoded hierarchy irradiance instead of leaving it black or flat; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            cool[2] > warm[2] + 0.2,
            "expected atlas-only current-frame card-capture resources to preserve blue authority in encoded hierarchy irradiance; warm={warm:?}, cool={cool:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_prefers_capture_scene_prepare_card_capture_resources_for_hierarchy_irradiance(
    ) {
        let warm = encode_probe_hierarchy_irradiance_with_scene_prepare_resources(
            [48, 96, 240, 255],
            [240, 96, 48, 255],
        );
        let cool = encode_probe_hierarchy_irradiance_with_scene_prepare_resources(
            [240, 96, 48, 255],
            [48, 96, 240, 255],
        );

        assert!(
            warm[3] > 0.1,
            "expected capture-side current-frame card-capture resources to provide nonzero hierarchy irradiance fallback during encode; warm={warm:?}"
        );
        assert!(
            warm[0] > cool[0] + 0.2,
            "expected capture-side current-frame card-capture resources to stay preferred over atlas-side truth in encoded hierarchy irradiance; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            cool[2] > warm[2] + 0.2,
            "expected capture-side current-frame card-capture resources to preserve blue authority in encoded hierarchy irradiance; warm={warm:?}, cool={cool:?}"
        );
    }

    fn encode_probe_hierarchy_irradiance_with_scene_prepare_resources(
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
                card_capture_requests: vec![HybridGiPrepareCardCaptureRequest {
                    card_id: 11,
                    page_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.6,
                }],
                surface_cache_page_contents: Vec::new(),
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            }));
        let resources = HybridGiScenePrepareResourcesSnapshot {
            card_capture_request_count: 1,
            occupied_atlas_slots: vec![3],
            occupied_capture_slots: if capture_sample_rgba[3] > 0 {
                vec![4]
            } else {
                Vec::new()
            },
            atlas_slot_rgba_samples: vec![(3, atlas_sample_rgba)],
            capture_slot_rgba_samples: if capture_sample_rgba[3] > 0 {
                vec![(4, capture_sample_rgba)]
            } else {
                Vec::new()
            },
            voxel_clipmap_ids: Vec::new(),
            voxel_clipmap_rgba_samples: Vec::new(),
            voxel_clipmap_occupancy_masks: Vec::new(),
            voxel_clipmap_cell_rgba_samples: Vec::new(),
            voxel_clipmap_cell_occupancy_counts: Vec::new(),
            voxel_clipmap_cell_dominant_node_ids: Vec::new(),
            voxel_clipmap_cell_dominant_rgba_samples: Vec::new(),
            atlas_slot_count: 4,
            capture_slot_count: 4,
            atlas_texture_extent: (16, 16),
            capture_texture_extent: (16, 16),
            capture_layer_count: 1,
        };

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, Some(&resources));
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        probes[0].hierarchy_irradiance_rgb_and_weight
    }

    #[test]
    fn encode_hybrid_gi_probes_scales_temporal_scene_truth_confidence_with_runtime_support() {
        let strong = encode_probe_temporal_scene_truth_confidence_with_runtime_support(0.52);
        let weak = encode_probe_temporal_scene_truth_confidence_with_runtime_support(0.08);

        assert!(
            strong > weak + 0.4,
            "expected stronger scene-driven runtime support to encode higher temporal scene-truth confidence instead of collapsing every scene-driven source to the same binary value; strong={strong:.3}, weak={weak:.3}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_scales_temporal_scene_truth_confidence_with_runtime_quality() {
        let high_quality =
            encode_probe_temporal_scene_truth_confidence_with_runtime_quality(0.52, 1.0);
        let low_quality =
            encode_probe_temporal_scene_truth_confidence_with_runtime_quality(0.52, 0.2);

        assert!(
            high_quality > low_quality + 0.45,
            "expected higher-quality scene-driven runtime truth to encode substantially higher temporal confidence at the same support, instead of ignoring runtime source quality; high_quality={high_quality:.3}, low_quality={low_quality:.3}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_accumulates_temporal_scene_truth_confidence_across_sources() {
        let single =
            encode_probe_temporal_scene_truth_confidence_with_runtime_sources(0.24, false, 0.0);
        let combined =
            encode_probe_temporal_scene_truth_confidence_with_runtime_sources(0.24, true, 0.24);

        assert!(
            combined > single + 0.15,
            "expected multiple reinforcing scene-driven sources to encode more temporal confidence than a single source at the same per-source support, instead of collapsing to the strongest source only; single={single:.3}, combined={combined:.3}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_prefers_exact_scene_truth_confidence_over_descendant_scene_truth() {
        let exact = encode_probe_temporal_scene_truth_confidence_from_lineage_source(true, 0.24);
        let descendant =
            encode_probe_temporal_scene_truth_confidence_from_lineage_source(false, 0.24);

        assert!(
            exact > descendant + 0.08,
            "expected exact scene-driven runtime truth to encode higher temporal confidence than descendant-derived truth at the same support, instead of treating lineage fallback as equally trustworthy; exact={exact:.3}, descendant={descendant:.3}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_prefers_exact_runtime_scene_truth_confidence_over_surface_cache_proxy(
    ) {
        let exact_runtime =
            encode_probe_temporal_scene_truth_confidence_with_runtime_sources(0.17, false, 0.0);
        let surface_cache =
            encode_probe_temporal_scene_truth_confidence_with_surface_cache_proxy(0.2);

        assert!(
            exact_runtime > surface_cache + 0.02,
            "expected exact runtime scene truth to encode higher temporal confidence than a similarly-supported surface-cache proxy, instead of treating capture/page fallback as equally trustworthy; exact_runtime={exact_runtime:.3}, surface_cache={surface_cache:.3}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_prefers_real_card_capture_truth_over_synthetic_request_proxy() {
        let capture_resource =
            encode_probe_temporal_scene_truth_confidence_with_card_capture_request_proxy(true);
        let synthetic_request =
            encode_probe_temporal_scene_truth_confidence_with_card_capture_request_proxy(false);

        assert!(
            capture_resource > synthetic_request + 0.08,
            "expected real card-capture resource truth to encode higher temporal confidence than a synthetic request fallback at the same spatial support, instead of treating placeholder request RGB as equally trustworthy; capture_resource={capture_resource:.3}, synthetic_request={synthetic_request:.3}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_ignores_surface_cache_proxy_signature_when_exact_runtime_scene_truth_exists(
    ) {
        let stable_signature =
            encode_probe_temporal_signature_with_exact_runtime_and_surface_cache_proxy(11, 22);
        let changed_proxy_signature =
            encode_probe_temporal_signature_with_exact_runtime_and_surface_cache_proxy(31, 47);

        assert!(
            (stable_signature - changed_proxy_signature).abs() < f32::EPSILON,
            "expected exact runtime scene truth to keep the same temporal signature when only the non-authoritative scene_prepare surface-cache proxy seed changes; stable_signature={stable_signature:.6}, changed_proxy_signature={changed_proxy_signature:.6}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_ignores_surface_cache_proxy_confidence_when_exact_runtime_scene_truth_exists(
    ) {
        let without_proxy =
            encode_probe_temporal_scene_truth_confidence_with_exact_runtime_scene_truth(None);
        let with_proxy = encode_probe_temporal_scene_truth_confidence_with_exact_runtime_scene_truth(
            Some((11, 22)),
        );

        assert!(
            (without_proxy - with_proxy).abs() < f32::EPSILON,
            "expected exact runtime scene truth to keep the same temporal confidence whether or not a non-authoritative scene_prepare surface-cache proxy is present; without_proxy={without_proxy:.6}, with_proxy={with_proxy:.6}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_ignores_surface_cache_proxy_signature_when_lineage_runtime_scene_truth_exists(
    ) {
        let inherited_stable =
            encode_probe_temporal_signature_with_lineage_runtime_scene_truth(true, Some((11, 22)));
        let inherited_changed =
            encode_probe_temporal_signature_with_lineage_runtime_scene_truth(true, Some((31, 47)));
        let descendant_stable =
            encode_probe_temporal_signature_with_lineage_runtime_scene_truth(false, Some((11, 22)));
        let descendant_changed =
            encode_probe_temporal_signature_with_lineage_runtime_scene_truth(false, Some((31, 47)));

        assert!(
            (inherited_stable - inherited_changed).abs() < f32::EPSILON,
            "expected inherited runtime scene truth to keep the same temporal signature when only the non-authoritative scene_prepare surface-cache proxy seed changes; inherited_stable={inherited_stable:.6}, inherited_changed={inherited_changed:.6}"
        );
        assert!(
            (descendant_stable - descendant_changed).abs() < f32::EPSILON,
            "expected descendant runtime scene truth to keep the same temporal signature when only the non-authoritative scene_prepare surface-cache proxy seed changes; descendant_stable={descendant_stable:.6}, descendant_changed={descendant_changed:.6}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_ignores_surface_cache_proxy_confidence_when_lineage_runtime_scene_truth_exists(
    ) {
        let inherited_without =
            encode_probe_temporal_scene_truth_confidence_with_lineage_runtime_scene_truth(true, None);
        let inherited_with = encode_probe_temporal_scene_truth_confidence_with_lineage_runtime_scene_truth(
            true,
            Some((11, 22)),
        );
        let descendant_without =
            encode_probe_temporal_scene_truth_confidence_with_lineage_runtime_scene_truth(false, None);
        let descendant_with = encode_probe_temporal_scene_truth_confidence_with_lineage_runtime_scene_truth(
            false,
            Some((11, 22)),
        );

        assert!(
            (inherited_without - inherited_with).abs() < f32::EPSILON,
            "expected inherited runtime scene truth to keep the same temporal confidence whether or not a non-authoritative scene_prepare surface-cache proxy is present; inherited_without={inherited_without:.6}, inherited_with={inherited_with:.6}"
        );
        assert!(
            (descendant_without - descendant_with).abs() < f32::EPSILON,
            "expected descendant runtime scene truth to keep the same temporal confidence whether or not a non-authoritative scene_prepare surface-cache proxy is present; descendant_without={descendant_without:.6}, descendant_with={descendant_with:.6}"
        );
    }

    fn encode_probe_temporal_scene_truth_confidence_with_runtime_support(support: f32) -> f32 {
        encode_probe_temporal_scene_truth_confidence_with_runtime_sources(support, false, 0.0)
    }

    fn encode_probe_temporal_signature_with_lineage_runtime_scene_truth(
        scene_truth_on_ancestor: bool,
        proxy_card_and_page: Option<(u32, u32)>,
    ) -> f32 {
        encode_probe_temporal_signature_and_confidence_with_lineage_runtime_scene_truth(
            scene_truth_on_ancestor,
            proxy_card_and_page,
        )
        .0
    }

    fn encode_probe_temporal_scene_truth_confidence_with_lineage_runtime_scene_truth(
        scene_truth_on_ancestor: bool,
        proxy_card_and_page: Option<(u32, u32)>,
    ) -> f32 {
        encode_probe_temporal_signature_and_confidence_with_lineage_runtime_scene_truth(
            scene_truth_on_ancestor,
            proxy_card_and_page,
        )
        .1
    }

    fn encode_probe_temporal_signature_and_confidence_with_lineage_runtime_scene_truth(
        scene_truth_on_ancestor: bool,
        proxy_card_and_page: Option<(u32, u32)>,
    ) -> (f32, f32) {
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
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                probe_hierarchy_resolve_weight_q8: std::collections::BTreeMap::from([(
                    runtime_probe_id,
                    HybridGiResolveRuntime::pack_resolve_weight_q8(0.9),
                )]),
                probe_hierarchy_irradiance_rgb_and_weight: std::collections::BTreeMap::from([(
                    runtime_probe_id,
                    HybridGiResolveRuntime::pack_rgb_and_weight([0.6, 0.6, 0.6], 0.52),
                )]),
                probe_scene_driven_hierarchy_irradiance_ids: std::collections::BTreeSet::from([
                    runtime_probe_id,
                ]),
                probe_scene_driven_hierarchy_irradiance_quality_q8:
                    std::collections::BTreeMap::from([(
                        runtime_probe_id,
                        HybridGiResolveRuntime::pack_scene_truth_quality_q8(1.0),
                    )]),
                ..Default::default()
            }));
        let frame = if let Some((card_id, page_id)) = proxy_card_and_page {
            frame.with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                card_capture_requests: vec![HybridGiPrepareCardCaptureRequest {
                    card_id,
                    page_id,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.2,
                }],
                surface_cache_page_contents: Vec::new(),
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            }))
        } else {
            frame
        };

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        (
            probes[0].temporal_signature_and_padding[0],
            probes[0].temporal_signature_and_padding[1],
        )
    }

    fn encode_probe_temporal_signature_with_exact_runtime_and_surface_cache_proxy(
        card_id: u32,
        page_id: u32,
    ) -> f32 {
        encode_probe_temporal_signature_and_confidence_with_exact_runtime_scene_truth(Some((
            card_id, page_id,
        )))
        .0
    }

    fn encode_probe_temporal_scene_truth_confidence_with_exact_runtime_scene_truth(
        proxy_card_and_page: Option<(u32, u32)>,
    ) -> f32 {
        encode_probe_temporal_signature_and_confidence_with_exact_runtime_scene_truth(
            proxy_card_and_page,
        )
        .1
    }

    fn encode_probe_temporal_signature_and_confidence_with_exact_runtime_scene_truth(
        proxy_card_and_page: Option<(u32, u32)>,
    ) -> (f32, f32) {
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
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                probe_hierarchy_resolve_weight_q8: std::collections::BTreeMap::from([(
                    probe.probe_id,
                    HybridGiResolveRuntime::pack_resolve_weight_q8(0.9),
                )]),
                probe_hierarchy_irradiance_rgb_and_weight: std::collections::BTreeMap::from([(
                    probe.probe_id,
                    HybridGiResolveRuntime::pack_rgb_and_weight([0.6, 0.6, 0.6], 0.52),
                )]),
                probe_scene_driven_hierarchy_irradiance_ids: std::collections::BTreeSet::from([
                    probe.probe_id,
                ]),
                probe_scene_driven_hierarchy_irradiance_quality_q8:
                    std::collections::BTreeMap::from([(
                        probe.probe_id,
                        HybridGiResolveRuntime::pack_scene_truth_quality_q8(1.0),
                    )]),
                ..Default::default()
            }));
        let frame = if let Some((card_id, page_id)) = proxy_card_and_page {
            frame.with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                card_capture_requests: vec![HybridGiPrepareCardCaptureRequest {
                    card_id,
                    page_id,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.2,
                }],
                surface_cache_page_contents: Vec::new(),
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            }))
        } else {
            frame
        };

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        (
            probes[0].temporal_signature_and_padding[0],
            probes[0].temporal_signature_and_padding[1],
        )
    }

    fn encode_probe_temporal_scene_truth_confidence_with_runtime_quality(
        support: f32,
        scene_truth_quality: f32,
    ) -> f32 {
        encode_probe_temporal_scene_truth_confidence_with_runtime_sources_and_quality(
            support,
            scene_truth_quality,
            false,
            0.0,
            1.0,
        )
    }

    fn encode_probe_temporal_scene_truth_confidence_with_runtime_sources(
        irradiance_support: f32,
        includes_rt_scene_truth: bool,
        rt_support: f32,
    ) -> f32 {
        encode_probe_temporal_scene_truth_confidence_with_runtime_sources_and_quality(
            irradiance_support,
            1.0,
            includes_rt_scene_truth,
            rt_support,
            1.0,
        )
    }

    fn encode_probe_temporal_scene_truth_confidence_with_runtime_sources_and_quality(
        irradiance_support: f32,
        irradiance_quality: f32,
        includes_rt_scene_truth: bool,
        rt_support: f32,
        rt_quality: f32,
    ) -> f32 {
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
            card_budget: 0,
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
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                probe_hierarchy_resolve_weight_q8: std::collections::BTreeMap::from([(
                    probe.probe_id,
                    HybridGiResolveRuntime::pack_resolve_weight_q8(0.9),
                )]),
                probe_hierarchy_irradiance_rgb_and_weight: std::collections::BTreeMap::from([(
                    probe.probe_id,
                    HybridGiResolveRuntime::pack_rgb_and_weight(
                        [0.6, 0.6, 0.6],
                        irradiance_support,
                    ),
                )]),
                probe_hierarchy_rt_lighting_rgb_and_weight: std::collections::BTreeMap::from([(
                    probe.probe_id,
                    HybridGiResolveRuntime::pack_rgb_and_weight([0.6, 0.6, 0.6], rt_support),
                )]),
                probe_scene_driven_hierarchy_irradiance_ids: std::collections::BTreeSet::from([
                    probe.probe_id,
                ]),
                probe_scene_driven_hierarchy_irradiance_quality_q8:
                    std::collections::BTreeMap::from([(
                        probe.probe_id,
                        HybridGiResolveRuntime::pack_scene_truth_quality_q8(irradiance_quality),
                    )]),
                probe_scene_driven_hierarchy_rt_lighting_ids: includes_rt_scene_truth
                    .then_some(std::collections::BTreeSet::from([probe.probe_id]))
                    .unwrap_or_default(),
                probe_scene_driven_hierarchy_rt_lighting_quality_q8: includes_rt_scene_truth
                    .then_some(std::collections::BTreeMap::from([(
                        probe.probe_id,
                        HybridGiResolveRuntime::pack_scene_truth_quality_q8(rt_quality),
                    )]))
                    .unwrap_or_default(),
                ..Default::default()
            }));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        probes[0].temporal_signature_and_padding[1]
    }

    fn encode_probe_temporal_scene_truth_confidence_from_lineage_source(
        exact_source: bool,
        support: f32,
    ) -> f32 {
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
            ray_budget: 96,
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
            card_budget: 0,
            voxel_budget: 0,
            probes: vec![parent_probe, child_probe],
            ..Default::default()
        });

        let runtime_probe_id = if exact_source {
            parent_probe.probe_id
        } else {
            child_probe.probe_id
        };
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: parent_probe.probe_id,
                    slot: 0,
                    ray_budget: parent_probe.ray_budget,
                    irradiance_rgb: [0, 0, 0],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                probe_hierarchy_resolve_weight_q8: std::collections::BTreeMap::from([(
                    runtime_probe_id,
                    HybridGiResolveRuntime::pack_resolve_weight_q8(0.9),
                )]),
                probe_hierarchy_irradiance_rgb_and_weight: std::collections::BTreeMap::from([(
                    runtime_probe_id,
                    HybridGiResolveRuntime::pack_rgb_and_weight([0.6, 0.6, 0.6], support),
                )]),
                probe_scene_driven_hierarchy_irradiance_ids: std::collections::BTreeSet::from([
                    runtime_probe_id,
                ]),
                probe_scene_driven_hierarchy_irradiance_quality_q8:
                    std::collections::BTreeMap::from([(
                        runtime_probe_id,
                        HybridGiResolveRuntime::pack_scene_truth_quality_q8(1.0),
                    )]),
                ..Default::default()
            }));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        probes[0].temporal_signature_and_padding[1]
    }

    fn encode_probe_temporal_scene_truth_confidence_with_surface_cache_proxy(
        bounds_radius: f32,
    ) -> f32 {
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
            card_budget: 0,
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
                    bounds_center: Vec3::ZERO,
                    bounds_radius,
                    atlas_sample_rgba: [160, 160, 160, 255],
                    capture_sample_rgba: [160, 160, 160, 255],
                }],
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            }));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        probes[0].temporal_signature_and_padding[1]
    }

    fn encode_probe_temporal_scene_truth_confidence_with_card_capture_request_proxy(
        include_capture_resource: bool,
    ) -> f32 {
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
                card_capture_requests: vec![HybridGiPrepareCardCaptureRequest {
                    card_id: 11,
                    page_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.2,
                }],
                surface_cache_page_contents: Vec::new(),
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            }));
        let resources = include_capture_resource.then_some(HybridGiScenePrepareResourcesSnapshot {
            card_capture_request_count: 1,
            occupied_atlas_slots: vec![3],
            occupied_capture_slots: vec![4],
            atlas_slot_rgba_samples: vec![(3, [64, 96, 128, 255])],
            capture_slot_rgba_samples: vec![(4, [180, 140, 96, 255])],
            voxel_clipmap_ids: Vec::new(),
            voxel_clipmap_rgba_samples: Vec::new(),
            voxel_clipmap_occupancy_masks: Vec::new(),
            voxel_clipmap_cell_rgba_samples: Vec::new(),
            voxel_clipmap_cell_occupancy_counts: Vec::new(),
            voxel_clipmap_cell_dominant_node_ids: Vec::new(),
            voxel_clipmap_cell_dominant_rgba_samples: Vec::new(),
            atlas_slot_count: 4,
            capture_slot_count: 4,
            atlas_texture_extent: (16, 16),
            capture_texture_extent: (16, 16),
            capture_layer_count: 1,
        });

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, resources.as_ref());
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        probes[0].temporal_signature_and_padding[1]
    }
}
