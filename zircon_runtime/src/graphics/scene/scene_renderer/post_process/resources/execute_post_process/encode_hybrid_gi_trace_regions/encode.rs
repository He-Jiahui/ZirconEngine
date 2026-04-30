use crate::core::math::UVec2;
use bytemuck::Zeroable;

use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::constants::MAX_HYBRID_GI_TRACE_REGIONS;
use super::super::super::super::hybrid_gi_trace_region_gpu::GpuHybridGiTraceRegion;
use super::super::encode_hybrid_gi_probes::runtime_parent_chain::{
    frame_has_runtime_scene_truth, scheduled_live_trace_region_ids,
    scheduled_runtime_trace_region_ids,
};
use super::super::hybrid_gi_trace_region_source::{
    fallback_trace_region_sources_by_id, HybridGiTraceRegionSource,
};
use super::encode_hybrid_gi_trace_region_screen_data::{
    dequantized_trace_region_coverage, encode_hybrid_gi_runtime_trace_region_screen_data,
    encode_hybrid_gi_trace_region_screen_data,
};
use super::hybrid_gi_trace_region_intensity::{
    hybrid_gi_trace_region_intensity, hybrid_gi_trace_region_intensity_from_coverage,
};
use super::hybrid_gi_trace_region_rt_lighting::{
    hybrid_gi_trace_region_rt_lighting, hybrid_gi_trace_region_rt_lighting_from_rgb,
};
use crate::graphics::hybrid_gi_extract_sources::{
    enabled_hybrid_gi_extract, hybrid_gi_extract_uses_scene_representation_budget,
};

pub(in super::super) fn encode_hybrid_gi_trace_regions(
    frame: &ViewportRenderFrame,
    viewport_size: UVec2,
    enabled: bool,
) -> ([GpuHybridGiTraceRegion; MAX_HYBRID_GI_TRACE_REGIONS], u32) {
    let mut trace_regions = [GpuHybridGiTraceRegion::zeroed(); MAX_HYBRID_GI_TRACE_REGIONS];
    if !enabled {
        return (trace_regions, 0);
    }
    if frame.hybrid_gi_resolve_runtime.is_some() {
        return encode_runtime_hybrid_gi_trace_regions(frame, viewport_size, trace_regions);
    }
    if frame.hybrid_gi_scene_prepare.is_some() || frame_has_runtime_scene_truth(frame) {
        return (trace_regions, 0);
    }

    if frame.hybrid_gi_prepare.is_none() {
        return (trace_regions, 0);
    }
    let Some(hybrid_gi_extract) =
        enabled_hybrid_gi_extract(frame.extract.lighting.hybrid_global_illumination.as_ref())
    else {
        return (trace_regions, 0);
    };
    if hybrid_gi_extract_uses_scene_representation_budget(hybrid_gi_extract) {
        return (trace_regions, 0);
    }

    let mut count = 0;
    let trace_regions_by_id = fallback_trace_region_sources_by_id(Some(hybrid_gi_extract));
    for region in scheduled_live_trace_region_ids(frame)
        .into_iter()
        .filter_map(|region_id| trace_regions_by_id.get(&region_id))
    {
        trace_regions[count] = GpuHybridGiTraceRegion {
            screen_uv_and_radius: encode_hybrid_gi_trace_region_screen_data(
                &frame.extract,
                viewport_size,
                region,
            ),
            boost_and_coverage: [
                hybrid_gi_trace_region_intensity(region, hybrid_gi_extract.tracing_budget),
                region.screen_coverage().clamp(0.1, 1.0),
                0.0,
                0.0,
            ],
            rt_lighting_rgb_and_weight: hybrid_gi_trace_region_rt_lighting(region),
        };
        count += 1;
    }

    (trace_regions, count as u32)
}

fn encode_runtime_hybrid_gi_trace_regions(
    frame: &ViewportRenderFrame,
    viewport_size: UVec2,
    mut trace_regions: [GpuHybridGiTraceRegion; MAX_HYBRID_GI_TRACE_REGIONS],
) -> ([GpuHybridGiTraceRegion; MAX_HYBRID_GI_TRACE_REGIONS], u32) {
    let Some(runtime) = frame.hybrid_gi_resolve_runtime.as_ref() else {
        return (trace_regions, 0);
    };
    let tracing_budget =
        enabled_hybrid_gi_extract(frame.extract.lighting.hybrid_global_illumination.as_ref())
            .map(|extract| extract.tracing_budget)
            .unwrap_or(MAX_HYBRID_GI_TRACE_REGIONS as u32);

    let mut count = 0;
    for region in scheduled_runtime_trace_region_ids(frame)
        .into_iter()
        .filter_map(|region_id| runtime.trace_region_scene_data(region_id))
    {
        let coverage = dequantized_trace_region_coverage(region).clamp(0.0, 1.0);
        trace_regions[count] = GpuHybridGiTraceRegion {
            screen_uv_and_radius: encode_hybrid_gi_runtime_trace_region_screen_data(
                &frame.extract,
                viewport_size,
                region,
            ),
            boost_and_coverage: [
                hybrid_gi_trace_region_intensity_from_coverage(coverage, tracing_budget),
                coverage.clamp(0.1, 1.0),
                0.0,
                0.0,
            ],
            rt_lighting_rgb_and_weight: hybrid_gi_trace_region_rt_lighting_from_rgb(
                region.rt_lighting_rgb(),
            ),
        };
        count += 1;
    }

    (trace_regions, count as u32)
}

#[cfg(test)]
mod tests {
    use super::super::super::super::super::constants::MAX_HYBRID_GI_TRACE_REGIONS;
    use super::encode_hybrid_gi_trace_regions;
    use crate::core::framework::render::{
        FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderHybridGiExtract,
        RenderHybridGiTraceRegion, RenderOverlayExtract, RenderSceneGeometryExtract,
        RenderSceneSnapshot, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
    };
    use crate::core::math::{UVec2, Vec3, Vec4};
    use crate::graphics::types::{
        HybridGiPrepareFrame, HybridGiResolveRuntime, HybridGiResolveTraceRegionSceneData,
        HybridGiScenePrepareFrame, ViewportRenderFrame,
    };

    #[test]
    fn encode_hybrid_gi_trace_regions_ignores_legacy_regions_when_stripped_runtime_truth_has_no_resident_probe(
    ) {
        let count =
            encode_legacy_trace_region_count_with_stripped_runtime_truth_without_resident_probe();

        assert_eq!(
            count, 0,
            "expected stripped runtime scene truth to suppress legacy RenderHybridGiTraceRegion encoding even when the prepare frame no longer has a resident probe"
        );
    }

    #[test]
    fn encode_hybrid_gi_trace_regions_ignores_legacy_regions_when_runtime_is_flat() {
        let count = encode_legacy_trace_region_count_with_flat_runtime();

        assert_eq!(
            count, 0,
            "expected flat runtime ownership to suppress legacy RenderHybridGiTraceRegion encoding instead of letting the old payload path drive main trace-region output"
        );
    }

    #[test]
    fn encode_hybrid_gi_trace_regions_prefers_runtime_scene_data_over_stale_legacy_payload() {
        let live_region_id = 40;
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
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(2), snapshot);
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            probe_budget: 0,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            trace_regions: vec![RenderHybridGiTraceRegion {
                entity: u64::from(live_region_id),
                region_id: live_region_id,
                bounds_center: Vec3::new(24.0, 0.0, 0.0),
                bounds_radius: 0.2,
                screen_coverage: 0.1,
                rt_lighting_rgb: [240, 64, 32],
            }],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: Vec::new(),
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: vec![live_region_id, live_region_id],
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_trace_region_scene_data(std::collections::BTreeMap::from([(
                        live_region_id,
                        HybridGiResolveTraceRegionSceneData::new(
                            2048,
                            2048,
                            2048,
                            96,
                            128,
                            [32, 64, 240],
                        ),
                    )]))
                    .build(),
            ));

        let (regions, count) = encode_hybrid_gi_trace_regions(&frame, UVec2::new(32, 32), true);

        assert_eq!(
            count, 1,
            "expected runtime trace-region scene data to encode once and ignore duplicate stale legacy scheduling"
        );
        assert!(
            regions[0].rt_lighting_rgb_and_weight[2] > 0.9
                && regions[0].rt_lighting_rgb_and_weight[0] < 0.2,
            "expected runtime trace-region lighting to win over stale legacy RenderHybridGiTraceRegion payload; encoded={:?}",
            regions[0].rt_lighting_rgb_and_weight
        );
    }

    #[test]
    fn encode_hybrid_gi_trace_regions_ignores_scene_prepare_runtime_data_backed_by_legacy_payload()
    {
        let stale_region_id = 40;
        let frame = frame_with_scene_prepare_runtime_trace_scene_data_and_legacy_payload(
            stale_region_id,
            vec![stale_region_id],
            vec![(stale_region_id, [32, 64, 240])],
        );

        let (_, count) = encode_hybrid_gi_trace_regions(&frame, UVec2::new(32, 32), true);

        assert_eq!(
            count, 0,
            "expected scene-prepare trace-region encode to reject runtime trace scene data backed by old RenderHybridGiTraceRegion payloads"
        );
    }

    #[test]
    fn encode_hybrid_gi_trace_regions_keeps_scene_prepare_runtime_only_data_when_legacy_payload_is_scheduled(
    ) {
        let legacy_region_id = 40;
        let runtime_only_region_id = 41;
        let frame = frame_with_scene_prepare_runtime_trace_scene_data_and_legacy_payload(
            legacy_region_id,
            vec![
                legacy_region_id,
                runtime_only_region_id,
                runtime_only_region_id,
            ],
            vec![
                (legacy_region_id, [240, 96, 48]),
                (runtime_only_region_id, [32, 64, 240]),
            ],
        );

        let (regions, count) = encode_hybrid_gi_trace_regions(&frame, UVec2::new(32, 32), true);

        assert_eq!(
            count, 1,
            "expected scene-prepare trace-region encode to filter only legacy-backed runtime trace ids"
        );
        assert!(
            regions[0].rt_lighting_rgb_and_weight[2] > 0.9
                && regions[0].rt_lighting_rgb_and_weight[0] < 0.2,
            "expected the remaining trace-region GPU payload to come from runtime-only scene data; encoded={:?}",
            regions[0].rt_lighting_rgb_and_weight
        );
    }

    #[test]
    fn encode_hybrid_gi_trace_regions_keeps_stripped_runtime_only_data_when_legacy_payload_is_scheduled(
    ) {
        let legacy_region_id = 40;
        let runtime_only_region_id = 41;
        let frame = frame_with_stripped_runtime_trace_scene_data_and_legacy_payload(
            legacy_region_id,
            vec![
                legacy_region_id,
                runtime_only_region_id,
                runtime_only_region_id,
            ],
            vec![
                (legacy_region_id, [240, 96, 48]),
                (runtime_only_region_id, [32, 64, 240]),
            ],
        );

        let (regions, count) = encode_hybrid_gi_trace_regions(&frame, UVec2::new(32, 32), true);

        assert_eq!(
            count, 1,
            "expected stripped runtime scene truth to filter only legacy-backed trace ids"
        );
        assert!(
            regions[0].rt_lighting_rgb_and_weight[2] > 0.9
                && regions[0].rt_lighting_rgb_and_weight[0] < 0.2,
            "expected stripped runtime trace-region GPU payload to come from runtime-only scene data; encoded={:?}",
            regions[0].rt_lighting_rgb_and_weight
        );
    }

    #[test]
    fn encode_hybrid_gi_trace_regions_ignores_legacy_regions_when_scene_representation_is_budgeted()
    {
        let count = encode_legacy_trace_region_count_with_budgeted_scene_representation();

        assert_eq!(
            count, 0,
            "expected budgeted scene-representation extracts to suppress legacy RenderHybridGiTraceRegion encoding instead of letting authored trace regions drive post-process output"
        );
    }

    #[test]
    fn encode_hybrid_gi_trace_regions_skips_stale_scheduled_ids_before_applying_region_limit() {
        let count = encode_trace_region_count_with_stale_scheduled_ids_before_live_payload();

        assert_eq!(
            count, 1,
            "expected stale scheduled ids without matching RenderHybridGiTraceRegion payloads not to consume the trace-region GPU budget before the live payload is encoded"
        );
    }

    #[test]
    fn encode_hybrid_gi_trace_regions_counts_duplicate_scheduled_live_payload_once() {
        let count = encode_trace_region_count_with_duplicate_live_scheduled_payload();

        assert_eq!(
            count, 1,
            "expected duplicate scheduled ids for the same live RenderHybridGiTraceRegion payload to encode once instead of inflating the old trace-region path"
        );
    }

    fn frame_with_scene_prepare_runtime_trace_scene_data_and_legacy_payload(
        legacy_region_id: u32,
        scheduled_trace_region_ids: Vec<u32>,
        runtime_regions: Vec<(u32, [u8; 3])>,
    ) -> ViewportRenderFrame {
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
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(7), snapshot);
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            trace_regions: vec![RenderHybridGiTraceRegion {
                entity: u64::from(legacy_region_id),
                region_id: legacy_region_id,
                bounds_center: Vec3::ZERO,
                bounds_radius: 2.0,
                screen_coverage: 0.8,
                rt_lighting_rgb: [240, 96, 48],
            }],
            ..Default::default()
        });

        ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: Vec::new(),
                pending_updates: Vec::new(),
                scheduled_trace_region_ids,
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame::default()))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_trace_region_scene_data(runtime_trace_region_scene_data(runtime_regions))
                    .build(),
            ))
    }

    fn frame_with_stripped_runtime_trace_scene_data_and_legacy_payload(
        legacy_region_id: u32,
        scheduled_trace_region_ids: Vec<u32>,
        runtime_regions: Vec<(u32, [u8; 3])>,
    ) -> ViewportRenderFrame {
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
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(8), snapshot);
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            trace_regions: vec![RenderHybridGiTraceRegion {
                entity: u64::from(legacy_region_id),
                region_id: legacy_region_id,
                bounds_center: Vec3::ZERO,
                bounds_radius: 2.0,
                screen_coverage: 0.8,
                rt_lighting_rgb: [240, 96, 48],
            }],
            ..Default::default()
        });

        ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: Vec::new(),
                pending_updates: Vec::new(),
                scheduled_trace_region_ids,
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_trace_region_scene_data(runtime_trace_region_scene_data(runtime_regions))
                    .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            200,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.68, 0.08, 0.06], 0.58),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_rt_lighting_ids(
                        std::collections::BTreeSet::from([200]),
                    )
                    .build(),
            ))
    }

    fn encode_legacy_trace_region_count_with_stripped_runtime_truth_without_resident_probe() -> u32
    {
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
            probe_budget: 0,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            trace_regions: vec![RenderHybridGiTraceRegion {
                entity: 40,
                region_id: 40,
                bounds_center: Vec3::ZERO,
                bounds_radius: 2.0,
                screen_coverage: 0.8,
                rt_lighting_rgb: [240, 96, 48],
            }],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: Vec::new(),
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: vec![40],
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            200,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.68, 0.08, 0.06], 0.58),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_rt_lighting_ids(
                        std::collections::BTreeSet::from([200]),
                    )
                    .build(),
            ));

        let (_, count) = encode_hybrid_gi_trace_regions(&frame, UVec2::new(32, 32), true);
        count
    }

    fn encode_legacy_trace_region_count_with_flat_runtime() -> u32 {
        let live_region_id = 40;
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
            probe_budget: 0,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            trace_regions: vec![RenderHybridGiTraceRegion {
                entity: u64::from(live_region_id),
                region_id: live_region_id,
                bounds_center: Vec3::ZERO,
                bounds_radius: 2.0,
                screen_coverage: 0.8,
                rt_lighting_rgb: [240, 96, 48],
            }],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: Vec::new(),
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: vec![live_region_id],
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime::default()));

        let (_, count) = encode_hybrid_gi_trace_regions(&frame, UVec2::new(32, 32), true);
        count
    }

    fn encode_legacy_trace_region_count_with_budgeted_scene_representation() -> u32 {
        let live_region_id = 40;
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
            probe_budget: 0,
            trace_budget: 1,
            card_budget: 1,
            voxel_budget: 1,
            trace_regions: vec![RenderHybridGiTraceRegion {
                entity: u64::from(live_region_id),
                region_id: live_region_id,
                bounds_center: Vec3::ZERO,
                bounds_radius: 2.0,
                screen_coverage: 0.8,
                rt_lighting_rgb: [240, 96, 48],
            }],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: Vec::new(),
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: vec![live_region_id],
                evictable_probe_ids: Vec::new(),
            }));

        let (_, count) = encode_hybrid_gi_trace_regions(&frame, UVec2::new(32, 32), true);
        count
    }

    fn runtime_trace_region_scene_data(
        runtime_regions: Vec<(u32, [u8; 3])>,
    ) -> std::collections::BTreeMap<u32, HybridGiResolveTraceRegionSceneData> {
        runtime_regions
            .into_iter()
            .map(|(region_id, rt_lighting_rgb)| {
                (
                    region_id,
                    HybridGiResolveTraceRegionSceneData::new(
                        2048,
                        2048,
                        2048,
                        96,
                        128,
                        rt_lighting_rgb,
                    ),
                )
            })
            .collect()
    }

    fn encode_trace_region_count_with_stale_scheduled_ids_before_live_payload() -> u32 {
        let live_region_id = 40;
        let mut scheduled_trace_region_ids = (0..MAX_HYBRID_GI_TRACE_REGIONS)
            .map(|index| 10_000 + index as u32)
            .collect::<Vec<_>>();
        scheduled_trace_region_ids.push(live_region_id);

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
            probe_budget: 0,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            trace_regions: vec![RenderHybridGiTraceRegion {
                entity: u64::from(live_region_id),
                region_id: live_region_id,
                bounds_center: Vec3::ZERO,
                bounds_radius: 2.0,
                screen_coverage: 0.8,
                rt_lighting_rgb: [240, 96, 48],
            }],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: Vec::new(),
                pending_updates: Vec::new(),
                scheduled_trace_region_ids,
                evictable_probe_ids: Vec::new(),
            }));

        let (_, count) = encode_hybrid_gi_trace_regions(&frame, UVec2::new(32, 32), true);
        count
    }

    fn encode_trace_region_count_with_duplicate_live_scheduled_payload() -> u32 {
        let live_region_id = 40;
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
            probe_budget: 0,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            trace_regions: vec![RenderHybridGiTraceRegion {
                entity: u64::from(live_region_id),
                region_id: live_region_id,
                bounds_center: Vec3::ZERO,
                bounds_radius: 2.0,
                screen_coverage: 0.8,
                rt_lighting_rgb: [240, 96, 48],
            }],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: Vec::new(),
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: vec![live_region_id, live_region_id],
                evictable_probe_ids: Vec::new(),
            }));

        let (_, count) = encode_hybrid_gi_trace_regions(&frame, UVec2::new(32, 32), true);
        count
    }
}
