use crate::core::math::UVec2;
use bytemuck::Zeroable;

use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::constants::MAX_HYBRID_GI_TRACE_REGIONS;
use super::super::super::super::hybrid_gi_trace_region_gpu::GpuHybridGiTraceRegion;
use super::super::encode_hybrid_gi_probes::runtime_parent_chain::{
    frame_has_runtime_scene_truth, scheduled_live_trace_region_ids,
};
use super::encode_hybrid_gi_trace_region_screen_data::encode_hybrid_gi_trace_region_screen_data;
use super::hybrid_gi_trace_region_intensity::hybrid_gi_trace_region_intensity;
use super::hybrid_gi_trace_region_rt_lighting::hybrid_gi_trace_region_rt_lighting;

pub(in super::super) fn encode_hybrid_gi_trace_regions(
    frame: &ViewportRenderFrame,
    viewport_size: UVec2,
    enabled: bool,
) -> ([GpuHybridGiTraceRegion; MAX_HYBRID_GI_TRACE_REGIONS], u32) {
    let mut trace_regions = [GpuHybridGiTraceRegion::zeroed(); MAX_HYBRID_GI_TRACE_REGIONS];
    if !enabled {
        return (trace_regions, 0);
    }
    if frame.hybrid_gi_scene_prepare.is_some() || frame_has_runtime_scene_truth(frame) {
        return (trace_regions, 0);
    }

    if frame.hybrid_gi_prepare.is_none() {
        return (trace_regions, 0);
    }
    let Some(hybrid_gi_extract) = frame.extract.lighting.hybrid_global_illumination.as_ref() else {
        return (trace_regions, 0);
    };

    let mut count = 0;
    for region in scheduled_live_trace_region_ids(frame)
        .into_iter()
        .filter_map(|region_id| {
            hybrid_gi_extract
                .trace_regions
                .iter()
                .find(|candidate| candidate.region_id == region_id)
        })
    {
        trace_regions[count] = GpuHybridGiTraceRegion {
            screen_uv_and_radius: encode_hybrid_gi_trace_region_screen_data(
                &frame.extract,
                viewport_size,
                region,
            ),
            boost_and_coverage: [
                hybrid_gi_trace_region_intensity(region, hybrid_gi_extract.tracing_budget),
                region.screen_coverage.clamp(0.1, 1.0),
                0.0,
                0.0,
            ],
            rt_lighting_rgb_and_weight: hybrid_gi_trace_region_rt_lighting(region),
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
        HybridGiPrepareFrame, HybridGiResolveRuntime, ViewportRenderFrame,
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
            trace_budget: 1,
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
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                probe_hierarchy_rt_lighting_rgb_and_weight: std::collections::BTreeMap::from([(
                    200,
                    HybridGiResolveRuntime::pack_rgb_and_weight([0.68, 0.08, 0.06], 0.58),
                )]),
                probe_scene_driven_hierarchy_rt_lighting_ids: std::collections::BTreeSet::from([
                    200,
                ]),
                ..Default::default()
            }));

        let (_, count) = encode_hybrid_gi_trace_regions(&frame, UVec2::new(32, 32), true);
        count
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
            trace_budget: 1,
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
            trace_budget: 2,
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
