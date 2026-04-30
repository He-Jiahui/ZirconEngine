use crate::core::math::{UVec2, Vec3};
use bytemuck::Zeroable;

use crate::graphics::hybrid_gi_extract_sources::{
    enabled_hybrid_gi_extract, hybrid_gi_extract_uses_scene_representation_budget,
};
use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::constants::MAX_HYBRID_GI_PROBES;
use super::super::super::super::hybrid_gi_probe_gpu::GpuHybridGiProbe;
use super::count_scheduled_trace_regions::count_scheduled_trace_regions;
use super::encode_hybrid_gi_probe_screen_data::{
    encode_hybrid_gi_probe_screen_data, encode_hybrid_gi_scene_driven_probe_screen_data,
    encode_hybrid_gi_scene_truth_fallback_probe_screen_data,
};
use super::hybrid_gi_hierarchy_irradiance::hybrid_gi_hierarchy_irradiance_with_scene_prepare_resources;
use super::hybrid_gi_hierarchy_resolve_weight::hybrid_gi_hierarchy_resolve_weight;
use super::hybrid_gi_hierarchy_rt_lighting::hybrid_gi_hierarchy_rt_lighting_with_scene_prepare_resources;
use super::hybrid_gi_probe_source::{
    fallback_probe_sources_by_id, HybridGiProbeSource, HybridGiRuntimeProbeSource,
};
use super::hybrid_gi_temporal_signature::{
    hybrid_gi_temporal_scene_truth_confidence, hybrid_gi_temporal_signature,
};
use super::runtime_parent_chain::{
    frame_has_runtime_probe_lineage_scene_truth, runtime_parent_topology_is_authoritative,
    runtime_probe_lineage_has_scene_truth,
};

const RUNTIME_PROBE_POSITION_SCALE: f32 = 64.0;
const RUNTIME_PROBE_POSITION_BIAS: i32 = 2048;
const RUNTIME_PROBE_RADIUS_SCALE: f32 = 96.0;

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
    let frame_has_scene_prepare = frame.hybrid_gi_scene_prepare.is_some();
    let frame_has_stripped_runtime_scene_truth =
        !frame_has_scene_prepare && frame_has_runtime_probe_lineage_scene_truth(frame);
    let frame_has_runtime_owner = frame.hybrid_gi_resolve_runtime.is_some();
    let frame_has_flat_runtime_owner =
        frame_has_runtime_owner && !runtime_parent_topology_is_authoritative(frame);
    let hybrid_gi_extract =
        enabled_hybrid_gi_extract(frame.extract.lighting.hybrid_global_illumination.as_ref());
    let extract_uses_scene_representation_budget = hybrid_gi_extract
        .map(hybrid_gi_extract_uses_scene_representation_budget)
        .unwrap_or(false);
    let hybrid_gi_probe_sources_by_id = fallback_probe_sources_by_id(hybrid_gi_extract);

    let mut count = 0;
    for probe in prepare.resident_probes.iter().take(MAX_HYBRID_GI_PROBES) {
        let extract_source = hybrid_gi_probe_sources_by_id.get(&probe.probe_id);
        let resident_has_runtime_scene_truth =
            runtime_probe_lineage_has_scene_truth(frame, probe.probe_id);
        let resident_has_runtime_payload =
            frame_has_exact_runtime_probe_payload(frame, probe.probe_id);
        let resident_has_runtime_scene_data =
            frame_has_runtime_probe_scene_data(frame, probe.probe_id);
        let runtime_scene_data_owns_source = frame_has_runtime_owner
            && resident_has_runtime_scene_data
            && (frame_has_scene_prepare
                || frame_has_stripped_runtime_scene_truth
                || resident_has_runtime_scene_truth);
        let should_synthesize_runtime_source = runtime_scene_data_owns_source
            || (extract_source.is_none()
                && ((frame_has_stripped_runtime_scene_truth && resident_has_runtime_scene_truth)
                    || (frame_has_scene_prepare
                        && frame_has_runtime_owner
                        && (resident_has_runtime_payload
                            || resident_has_runtime_scene_truth
                            || resident_has_runtime_scene_data))));
        let runtime_probe_source = should_synthesize_runtime_source
            .then(|| {
                runtime_probe_scene_source(
                    frame,
                    probe.probe_id,
                    probe.ray_budget,
                    frame_has_stripped_runtime_scene_truth && resident_has_runtime_scene_truth,
                )
            })
            .flatten();
        let source = match (runtime_probe_source.as_ref(), extract_source) {
            (Some(source), _) => Some(source as &dyn HybridGiProbeSource),
            (None, Some(source)) => Some(source as &dyn HybridGiProbeSource),
            (None, None) => None,
        };
        let source_is_scene_driven = source
            .map(|source| {
                runtime_probe_lineage_has_scene_truth(frame, source.probe_id())
                    || runtime_scene_data_owns_source
                    || (frame_has_scene_prepare
                        && (!frame_has_runtime_owner
                            || resident_has_runtime_payload
                            || resident_has_runtime_scene_truth
                            || resident_has_runtime_scene_data))
            })
            .unwrap_or(false);
        if (frame_has_scene_prepare || frame_has_stripped_runtime_scene_truth) && source.is_none() {
            continue;
        }
        if extract_uses_scene_representation_budget && !source_is_scene_driven {
            continue;
        }
        if frame_has_scene_prepare && frame_has_runtime_owner && !source_is_scene_driven {
            continue;
        }
        if frame_has_flat_runtime_owner && !source_is_scene_driven && !resident_has_runtime_payload
        {
            continue;
        }
        if frame_has_stripped_runtime_scene_truth && !source_is_scene_driven {
            continue;
        }
        let (screen_data, hierarchy_weight, hierarchy_irradiance, hierarchy_rt_lighting) = source
            .map(|source| {
                (
                    if frame_has_scene_prepare {
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
                                    source.ray_budget(),
                                )
                            },
                        )
                    } else if source_is_scene_driven {
                        encode_hybrid_gi_scene_truth_fallback_probe_screen_data(source.ray_budget())
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
            source.and_then(|probe| probe.parent_probe_id()),
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
                if source_is_scene_driven {
                    0.0
                } else {
                    probe.irradiance_rgb[0] as f32 / 255.0
                },
                if source_is_scene_driven {
                    0.0
                } else {
                    probe.irradiance_rgb[1] as f32 / 255.0
                },
                if source_is_scene_driven {
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

fn frame_has_exact_runtime_probe_payload(frame: &ViewportRenderFrame, probe_id: u32) -> bool {
    frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .map(|runtime| {
            runtime
                .hierarchy_irradiance(probe_id)
                .map(|source| source[3] > f32::EPSILON)
                .unwrap_or(false)
                || runtime
                    .hierarchy_rt_lighting(probe_id)
                    .map(|source| source[3] > f32::EPSILON)
                    .unwrap_or(false)
                || runtime.has_hierarchy_resolve_weight(probe_id)
                || runtime.has_probe_rt_lighting(probe_id)
        })
        .unwrap_or(false)
}

fn frame_has_runtime_probe_scene_data(frame: &ViewportRenderFrame, probe_id: u32) -> bool {
    frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .and_then(|runtime| runtime.probe_scene_data(probe_id))
        .is_some()
}

fn runtime_probe_scene_source(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    ray_budget: u32,
    allow_neutral_fallback: bool,
) -> Option<HybridGiRuntimeProbeSource> {
    let runtime = frame.hybrid_gi_resolve_runtime.as_ref()?;
    if let Some(scene_data) = runtime.probe_scene_data(probe_id) {
        return Some(HybridGiRuntimeProbeSource::new(
            probe_id,
            dequantized_runtime_probe_position(
                scene_data.position_x_q(),
                scene_data.position_y_q(),
                scene_data.position_z_q(),
            ),
            scene_data.radius_q() as f32 / RUNTIME_PROBE_RADIUS_SCALE,
            runtime.parent_probe_id(probe_id),
            ray_budget,
        ));
    }

    allow_neutral_fallback.then_some(HybridGiRuntimeProbeSource::new(
        probe_id,
        Vec3::ZERO,
        0.0,
        None,
        ray_budget,
    ))
}

fn dequantized_runtime_probe_position(x_q: u32, y_q: u32, z_q: u32) -> Vec3 {
    Vec3::new(
        dequantized_runtime_probe_position_component(x_q),
        dequantized_runtime_probe_position_component(y_q),
        dequantized_runtime_probe_position_component(z_q),
    )
}

fn dequantized_runtime_probe_position_component(value: u32) -> f32 {
    (value as i32 - RUNTIME_PROBE_POSITION_BIAS) as f32 / RUNTIME_PROBE_POSITION_SCALE
}

#[cfg(test)]
mod tests {
    use super::super::super::super::super::constants::MAX_HYBRID_GI_TRACE_REGIONS;
    use super::encode_hybrid_gi_probes;
    use crate::core::framework::render::{
        FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode, RenderFrameExtract,
        RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion,
        RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
        RenderWorldSnapshotHandle, ViewportCameraSnapshot,
    };
    use crate::core::math::{Transform, UVec2, Vec3, Vec4};
    use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
    use crate::graphics::types::{
        HybridGiPrepareCardCaptureRequest, HybridGiPrepareFrame, HybridGiPrepareProbe,
        HybridGiPrepareSurfaceCachePageContent, HybridGiPrepareVoxelCell,
        HybridGiPrepareVoxelClipmap, HybridGiResolveProbeSceneData, HybridGiResolveRuntime,
        HybridGiScenePrepareFrame, ViewportRenderFrame,
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
        let has_capture_sample = capture_sample_rgba[3] > 0;
        let occupied_capture_slots = if has_capture_sample {
            vec![4]
        } else {
            Vec::new()
        };
        let capture_slot_rgba_samples = if has_capture_sample {
            vec![(4, capture_sample_rgba)]
        } else {
            Vec::new()
        };
        let mut resources = HybridGiScenePrepareResourcesSnapshot::new(
            1,
            Vec::new(),
            vec![3],
            occupied_capture_slots,
            4,
            4,
            (16, 16),
            (16, 16),
            1,
        );
        resources.store_texture_slot_rgba_samples(
            vec![(3, atlas_sample_rgba)],
            capture_slot_rgba_samples,
        );

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
    fn encode_hybrid_gi_probes_keeps_surface_cache_proxy_confidence_with_stale_scheduled_trace_id()
    {
        let stale_trace_confidence =
            encode_probe_temporal_scene_truth_confidence_with_surface_cache_proxy_and_stale_trace_id(
            );

        assert!(
            stale_trace_confidence > 0.08,
            "expected a stale scheduled trace id without a matching RenderHybridGiTraceRegion payload not to suppress surface-cache proxy participation in temporal confidence; stale_trace_confidence={stale_trace_confidence:.3}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_counts_live_trace_payloads_after_stale_scheduled_ids() {
        let trace_count = encode_probe_trace_count_with_stale_scheduled_ids_before_live_payload();

        assert_eq!(
            trace_count, 1,
            "expected stale scheduled trace ids without RenderHybridGiTraceRegion payloads not to consume the probe encoder's trace-region count budget before a live payload"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_counts_duplicate_scheduled_live_payload_once() {
        let trace_count = encode_probe_trace_count_with_duplicate_live_scheduled_payload();

        assert_eq!(
            trace_count, 1,
            "expected duplicate scheduled ids for the same live RenderHybridGiTraceRegion payload to count once instead of inflating the old trace-region path"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_reports_zero_trace_payloads_when_runtime_is_flat() {
        let trace_count = encode_probe_trace_count_with_flat_runtime();

        assert_eq!(
            trace_count, 0,
            "expected flat runtime ownership to suppress legacy scheduled RenderHybridGiTraceRegion counts instead of reporting old payloads to the probe encoder"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_ignores_legacy_probe_slots_when_runtime_is_flat() {
        let (probe_count, legacy_irradiance) = encode_probe_count_with_flat_runtime();

        assert_eq!(
            probe_count, 0,
            "expected flat runtime ownership to suppress legacy RenderHybridGiProbe resident slots instead of letting old probe payloads drive main probe output"
        );
        assert_eq!(
            legacy_irradiance,
            [0.0; 4],
            "expected flat runtime ownership to leave the legacy probe GPU slot zeroed; legacy_irradiance={legacy_irradiance:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_keeps_exact_runtime_probe_payload_when_runtime_is_flat() {
        let (probe_count, runtime_rt_lighting) =
            encode_probe_count_with_flat_runtime_exact_payload();

        assert_eq!(
            probe_count, 1,
            "expected flat runtime ownership to suppress only legacy-only probe slots, not resident probes with exact runtime payloads"
        );
        assert!(
            runtime_rt_lighting[0] > runtime_rt_lighting[2] + 0.4
                && runtime_rt_lighting[3] > 0.2,
            "expected exact runtime RT payload to drive encoded hierarchy lighting even without runtime parent topology; runtime_rt_lighting={runtime_rt_lighting:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_keeps_exact_runtime_resolve_weight_when_runtime_is_flat() {
        let high_weight = encode_probe_resolve_weight_with_flat_runtime_exact_weight(2.4);
        let low_weight = encode_probe_resolve_weight_with_flat_runtime_exact_weight(0.6);

        assert!(
            high_weight > low_weight + 1.5,
            "expected exact runtime resolve weight to reach the encoded probe even when runtime topology is flat; high_weight={high_weight:.3}, low_weight={low_weight:.3}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_keeps_legacy_parent_resolve_weight_without_runtime_owner() {
        let flat_weight = encode_child_probe_resolve_weight_without_runtime_parent(false);
        let hierarchical_weight = encode_child_probe_resolve_weight_without_runtime_parent(true);

        assert!(
            hierarchical_weight > flat_weight + 0.2,
            "expected legacy RenderHybridGiProbe parent topology to keep affecting resolve weight when no runtime owner is present; flat_weight={flat_weight:.3}, hierarchical_weight={hierarchical_weight:.3}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_ignores_legacy_probe_slots_when_scene_representation_is_budgeted() {
        let (probe_count, legacy_irradiance) =
            encode_probe_count_with_budgeted_scene_representation_and_legacy_probe_slot();

        assert_eq!(
            probe_count, 0,
            "expected budgeted scene-representation extracts to suppress legacy RenderHybridGiProbe resident slots instead of letting authored payloads drive post-process probe output"
        );
        assert_eq!(
            legacy_irradiance,
            [0.0; 4],
            "expected budgeted scene-representation ownership to leave the legacy probe GPU slot zeroed; legacy_irradiance={legacy_irradiance:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_ignores_legacy_probe_slots_when_flat_runtime_has_scene_prepare() {
        let (probe_count, legacy_irradiance) =
            encode_probe_count_with_flat_runtime_and_scene_prepare();

        assert_eq!(
            probe_count, 0,
            "expected flat runtime ownership to suppress legacy RenderHybridGiProbe resident slots even when scene prepare exists"
        );
        assert_eq!(
            legacy_irradiance,
            [0.0; 4],
            "expected scene prepare not to reclassify a legacy RenderHybridGiProbe slot as scene-driven; legacy_irradiance={legacy_irradiance:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_ignores_legacy_probe_slots_when_unrelated_runtime_topology_has_scene_prepare(
    ) {
        let (probe_count, legacy_irradiance) =
            encode_probe_count_with_unrelated_runtime_topology_and_scene_prepare();

        assert_eq!(
            probe_count, 0,
            "expected unrelated runtime parent topology not to reclassify a legacy RenderHybridGiProbe resident slot as scene-driven"
        );
        assert_eq!(
            legacy_irradiance,
            [0.0; 4],
            "expected unrelated runtime parent topology with scene prepare to leave the legacy probe GPU slot zeroed; legacy_irradiance={legacy_irradiance:?}"
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
    fn encode_hybrid_gi_probes_scales_temporal_scene_truth_confidence_with_rt_continuation_surface_cache_proxy_quality(
    ) {
        let capture_resource =
            encode_probe_temporal_scene_truth_confidence_with_rt_continuation_surface_cache_proxy(
                true,
            );
        let synthetic_request =
            encode_probe_temporal_scene_truth_confidence_with_rt_continuation_surface_cache_proxy(
                false,
            );

        assert!(
            capture_resource > synthetic_request + 0.08,
            "expected higher-quality current surface-cache truth mixed through RT continuation to encode higher temporal confidence than a synthetic request fallback at the same support, instead of dropping proxy quality on the RT path; capture_resource={capture_resource:.3}, synthetic_request={synthetic_request:.3}"
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
        let with_proxy =
            encode_probe_temporal_scene_truth_confidence_with_exact_runtime_scene_truth(Some((
                11, 22,
            )));

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
            encode_probe_temporal_scene_truth_confidence_with_lineage_runtime_scene_truth(
                true, None,
            );
        let inherited_with =
            encode_probe_temporal_scene_truth_confidence_with_lineage_runtime_scene_truth(
                true,
                Some((11, 22)),
            );
        let descendant_without =
            encode_probe_temporal_scene_truth_confidence_with_lineage_runtime_scene_truth(
                false, None,
            );
        let descendant_with =
            encode_probe_temporal_scene_truth_confidence_with_lineage_runtime_scene_truth(
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

    #[test]
    fn encode_hybrid_gi_probes_keeps_authored_irradiance_when_lineage_scene_truth_flag_has_no_supported_source(
    ) {
        let encoded_irradiance =
            encode_probe_irradiance_with_lineage_scene_truth_flag_without_source();

        assert!(
            encoded_irradiance[0] > 0.85,
            "expected unsupported runtime scene-truth lineage flags not to demote authored resident-probe irradiance; encoded_irradiance={encoded_irradiance:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_temporal_signature_tracks_legacy_probe_rt_scene_truth() {
        let warm = encode_probe_temporal_signature_and_confidence_with_legacy_rt_scene_truth([
            240, 96, 48,
        ]);
        let cool = encode_probe_temporal_signature_and_confidence_with_legacy_rt_scene_truth([
            48, 96, 240,
        ]);

        assert!(
            warm.1 > 0.2 && cool.1 > 0.2,
            "expected legacy probe_rt_lighting_rgb scene truth to carry nonzero temporal confidence through the same RT scene-truth path as packed hierarchy RT; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            (warm.0 - cool.0).abs() > 0.01,
            "expected temporal signature to change when legacy probe_rt_lighting_rgb scene truth changes RGB without packed hierarchy RT, instead of treating it as probe-only continuation; warm={warm:?}, cool={cool:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_drops_continuation_irradiance_when_rt_scene_truth_owns_stripped_frame(
    ) {
        let hierarchy_irradiance =
            encode_probe_hierarchy_irradiance_with_rt_only_stripped_runtime_truth();

        assert_eq!(
            hierarchy_irradiance,
            [0.0; 4],
            "expected RT-only stripped runtime scene truth to drop continuation-only hierarchy irradiance instead of letting legacy RGB re-enter the encoded probe; hierarchy_irradiance={hierarchy_irradiance:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_drops_continuation_irradiance_when_rt_scene_truth_owns_scene_prepare_frame(
    ) {
        let hierarchy_irradiance =
            encode_probe_hierarchy_irradiance_with_rt_only_scene_prepare_truth();

        assert_eq!(
            hierarchy_irradiance,
            [0.0; 4],
            "expected RT-only scene_prepare scene truth to drop continuation-only hierarchy irradiance instead of letting legacy RGB re-enter the encoded probe; hierarchy_irradiance={hierarchy_irradiance:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_drops_continuation_rt_when_irradiance_scene_truth_owns_stripped_frame(
    ) {
        let hierarchy_rt_lighting =
            encode_probe_hierarchy_rt_lighting_with_irradiance_only_stripped_runtime_truth();

        assert_eq!(
            hierarchy_rt_lighting,
            [0.0; 4],
            "expected irradiance-only stripped runtime scene truth to drop continuation-only hierarchy RT lighting instead of letting legacy RT RGB re-enter the encoded probe; hierarchy_rt_lighting={hierarchy_rt_lighting:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_drops_continuation_rt_when_irradiance_scene_truth_owns_scene_prepare_frame(
    ) {
        let hierarchy_rt_lighting =
            encode_probe_hierarchy_rt_lighting_with_irradiance_only_scene_prepare_truth();

        assert_eq!(
            hierarchy_rt_lighting,
            [0.0; 4],
            "expected irradiance-only scene_prepare scene truth to drop continuation-only hierarchy RT lighting instead of letting legacy RT RGB re-enter the encoded probe; hierarchy_rt_lighting={hierarchy_rt_lighting:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_skips_unmatched_resident_slots_when_stripped_runtime_truth_exists() {
        let (probe_count, unmatched_irradiance) =
            encode_probe_count_and_unmatched_irradiance_with_stripped_runtime_truth();

        assert_eq!(
            probe_count, 1,
            "expected stripped-scene-prepare runtime scene truth to drop unmatched compatibility-only resident slots just like full scene_prepare frames, instead of letting authored container slots inflate probe_count"
        );
        assert_eq!(
            unmatched_irradiance, [0.0; 4],
            "expected unmatched compatibility-only resident slot to stay zeroed once stripped runtime scene truth owns the frame; unmatched_irradiance={unmatched_irradiance:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_keeps_source_stripped_runtime_truth_slot_when_scene_prepare_is_stripped(
    ) {
        let (probe_count, screen_data, authored_irradiance, hierarchy_irradiance) =
            encode_probe_count_and_unmatched_runtime_truth_slot_without_source();

        assert_eq!(
            probe_count, 1,
            "expected stripped runtime scene truth without an authored source to synthesize a scene-driven probe instead of dropping runtime-owned data"
        );
        assert_eq!(
            [screen_data[0], screen_data[1], screen_data[2]],
            [0.5, 0.5, 1.0],
            "expected source-stripped runtime scene truth to use neutral full-frame screen support instead of authored probe coordinates; screen_data={screen_data:?}"
        );
        assert!(
            screen_data[3] > 0.0,
            "expected source-stripped runtime scene truth to preserve ray-budget support weight; screen_data={screen_data:?}"
        );
        assert_eq!(
            authored_irradiance[0..3],
            [0.0, 0.0, 0.0],
            "expected source-stripped runtime scene truth to demote authored prepare irradiance; authored_irradiance={authored_irradiance:?}"
        );
        assert!(
            hierarchy_irradiance[0] > 0.55
                && hierarchy_irradiance[0] < 0.65
                && hierarchy_irradiance[1] > 0.55
                && hierarchy_irradiance[1] < 0.65
                && hierarchy_irradiance[2] > 0.55
                && hierarchy_irradiance[2] < 0.65
                && hierarchy_irradiance[3] > 0.45,
            "expected source-stripped runtime scene truth to carry runtime hierarchy irradiance; hierarchy_irradiance={hierarchy_irradiance:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_keeps_source_stripped_rt_truth_without_legacy_extract_container() {
        let (probe_count, screen_data, authored_irradiance, hierarchy_rt_lighting) =
            encode_probe_count_and_rt_truth_without_legacy_extract_container();

        assert_eq!(
            probe_count, 1,
            "expected source-stripped runtime RT scene truth to synthesize a scene-driven probe even when the legacy hybrid GI extract container is absent"
        );
        assert_eq!(
            [screen_data[0], screen_data[1], screen_data[2]],
            [0.5, 0.5, 1.0],
            "expected source-stripped runtime RT scene truth without a legacy extract container to use neutral full-frame support; screen_data={screen_data:?}"
        );
        assert_eq!(
            authored_irradiance[0..3],
            [0.0, 0.0, 0.0],
            "expected source-stripped runtime RT scene truth without a legacy extract container to demote authored prepare irradiance; authored_irradiance={authored_irradiance:?}"
        );
        assert!(
            hierarchy_rt_lighting[0] > 0.45
                && hierarchy_rt_lighting[0] < 0.58
                && hierarchy_rt_lighting[1] > 0.45
                && hierarchy_rt_lighting[1] < 0.58
                && hierarchy_rt_lighting[2] > 0.45
                && hierarchy_rt_lighting[2] < 0.58
                && hierarchy_rt_lighting[3] > 0.35,
            "expected source-stripped runtime RT scene truth to carry packed runtime RT lighting without requiring a legacy extract container; hierarchy_rt_lighting={hierarchy_rt_lighting:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_keeps_scene_prepare_runtime_probe_scene_data_without_legacy_probe_source(
    ) {
        let probe_id = 300;
        let viewport_size = UVec2::new(32, 32);
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
        let extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(99), snapshot);
        let frame = ViewportRenderFrame::from_extract(extract, viewport_size)
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id,
                    slot: 0,
                    ray_budget: 128,
                    irradiance_rgb: [240, 96, 48],
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
                    .with_probe_scene_data(std::collections::BTreeMap::from([(
                        probe_id,
                        HybridGiResolveProbeSceneData::new(2048, 2048, 2048, 96),
                    )]))
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.62, 0.25, 0.12], 0.7),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([probe_id]),
                    )
                    .build(),
            ));

        let (probes, probe_count, _) = encode_hybrid_gi_probes(&frame, viewport_size, true, None);

        assert_eq!(
            probe_count, 1,
            "expected runtime probe scene data to keep a scene-prepare resident slot without requiring a legacy RenderHybridGiProbe source"
        );
        assert_eq!(
            probes[0].irradiance_and_intensity[0..3],
            [0.0, 0.0, 0.0],
            "expected runtime-owned scene-prepare output to demote authored resident irradiance"
        );
        assert!(
            probes[0].hierarchy_irradiance_rgb_and_weight[0] > 0.55
                && probes[0].hierarchy_irradiance_rgb_and_weight[3] > 0.65,
            "expected runtime hierarchy irradiance to drive the encoded probe; encoded={:?}",
            probes[0].hierarchy_irradiance_rgb_and_weight
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_prefers_runtime_probe_scene_data_over_stale_legacy_probe_source_in_scene_prepare(
    ) {
        let probe_id = 301;
        let viewport_size = UVec2::new(32, 32);
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
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(100), snapshot);
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            probe_budget: 1,
            probes: vec![RenderHybridGiProbe {
                probe_id,
                resident: true,
                ray_budget: 128,
                position: Vec3::new(18.0, 0.0, 0.0),
                radius: 0.2,
                ..Default::default()
            }],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, viewport_size)
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id,
                    slot: 0,
                    ray_budget: 128,
                    irradiance_rgb: [240, 96, 48],
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
                    atlas_slot_id: 0,
                    capture_slot_id: 0,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 1.0,
                    atlas_sample_rgba: [0, 0, 0, 0],
                    capture_sample_rgba: [240, 64, 32, 255],
                }],
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_scene_data(std::collections::BTreeMap::from([(
                        probe_id,
                        HybridGiResolveProbeSceneData::new(2048, 2048, 2048, 96),
                    )]))
                    .build(),
            ));

        let (probes, probe_count, _) = encode_hybrid_gi_probes(&frame, viewport_size, true, None);

        assert_eq!(
            probe_count, 1,
            "expected runtime probe scene data to override a stale legacy RenderHybridGiProbe source during scene prepare"
        );
        assert_eq!(
            probes[0].irradiance_and_intensity[0..3],
            [0.0, 0.0, 0.0],
            "expected runtime-owned scene data to demote authored resident irradiance even when a stale legacy probe source is present"
        );
        assert!(
            probes[0].hierarchy_irradiance_rgb_and_weight[0] > 0.85
                && probes[0].hierarchy_irradiance_rgb_and_weight[3] > 0.2,
            "expected scene-prepare surface-cache fallback to use runtime probe coordinates instead of stale legacy coordinates; encoded={:?}",
            probes[0].hierarchy_irradiance_rgb_and_weight
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_keeps_runtime_parent_scene_truth_without_legacy_extract_container() {
        let (probe_count, child_irradiance) =
            encode_child_probe_with_runtime_parent_truth_without_legacy_extract_container();

        assert_eq!(
            probe_count, 1,
            "expected source-stripped runtime parent scene truth to keep the child resident slot even when the legacy hybrid GI extract container is absent"
        );
        assert!(
            child_irradiance[0] > 0.55
                && child_irradiance[1] < 0.2
                && child_irradiance[2] < 0.2
                && child_irradiance[3] > 0.45,
            "expected runtime-only parent-chain scene truth to inherit warm parent irradiance without requiring RenderHybridGiProbe parent topology; child_irradiance={child_irradiance:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_keeps_runtime_parent_scene_truth_when_legacy_extract_topology_is_empty(
    ) {
        let (probe_count, child_irradiance) =
            encode_child_probe_with_runtime_parent_truth_and_empty_legacy_extract_topology();

        assert_eq!(
            probe_count, 1,
            "expected runtime parent scene truth to keep the child resident slot when the legacy hybrid GI extract container remains only as settings"
        );
        assert!(
            child_irradiance[0] > 0.55
                && child_irradiance[1] < 0.2
                && child_irradiance[2] < 0.2
                && child_irradiance[3] > 0.45,
            "expected runtime parent topology to override an empty legacy RenderHybridGiProbe topology during migration; child_irradiance={child_irradiance:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_prefers_runtime_parent_scene_truth_over_stale_legacy_extract_topology(
    ) {
        let (screen_data, authored_irradiance, child_irradiance) =
            encode_child_probe_with_runtime_parent_truth_and_stale_legacy_extract_topology();

        assert_eq!(
            [screen_data[0], screen_data[1], screen_data[2]],
            [0.5, 0.5, 1.0],
            "expected stale legacy RenderHybridGiProbe coordinates not to localize a runtime-owned child slot; screen_data={screen_data:?}"
        );
        assert_eq!(
            authored_irradiance[0..3],
            [0.0, 0.0, 0.0],
            "expected runtime parent scene truth to demote authored child prepare irradiance even when a stale legacy source probe still exists; authored_irradiance={authored_irradiance:?}"
        );
        assert!(
            child_irradiance[0] > 0.55
                && child_irradiance[1] < 0.2
                && child_irradiance[2] < 0.2
                && child_irradiance[3] > 0.45,
            "expected runtime parent topology to stay authoritative over stale legacy RenderHybridGiProbe parent links; child_irradiance={child_irradiance:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_ignores_stale_legacy_parent_scene_truth_when_runtime_topology_has_no_link(
    ) {
        let (authored_irradiance, child_irradiance) =
            encode_child_probe_with_stale_legacy_parent_truth_and_unlinked_runtime_topology();

        assert!(
            authored_irradiance[0] > 0.9
                && authored_irradiance[1] > 0.3
                && authored_irradiance[2] > 0.15,
            "expected unlinked runtime topology not to let stale RenderHybridGiProbe parent scene truth demote authored child irradiance; authored_irradiance={authored_irradiance:?}"
        );
        assert_eq!(
            child_irradiance,
            [0.0, 0.0, 0.0, 0.0],
            "expected runtime topology to be authoritative for an unlinked child instead of inheriting stale legacy parent scene truth; child_irradiance={child_irradiance:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_ignores_stale_legacy_parent_scene_truth_when_runtime_scene_truth_has_flat_topology(
    ) {
        let (authored_irradiance, child_irradiance) =
            encode_child_probe_with_stale_legacy_parent_truth_and_flat_runtime_scene_truth_topology(
            );

        assert!(
            authored_irradiance[0] > 0.9
                && authored_irradiance[1] > 0.3
                && authored_irradiance[2] > 0.15,
            "expected flat runtime scene-truth topology not to let stale RenderHybridGiProbe parent scene truth demote authored child irradiance; authored_irradiance={authored_irradiance:?}"
        );
        assert_eq!(
            child_irradiance,
            [0.0, 0.0, 0.0, 0.0],
            "expected flat runtime scene-truth topology to stay authoritative instead of inheriting stale legacy parent scene truth; child_irradiance={child_irradiance:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_ignores_stale_legacy_parent_continuation_when_runtime_topology_has_no_link(
    ) {
        let child_irradiance =
            encode_child_probe_with_stale_legacy_parent_continuation_and_unlinked_runtime_topology(
            );

        assert_eq!(
            child_irradiance,
            [0.0, 0.0, 0.0, 0.0],
            "expected runtime topology to be authoritative for continuation fallback instead of walking stale RenderHybridGiProbe parent links; child_irradiance={child_irradiance:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_ignores_stale_legacy_parent_identity_when_runtime_topology_has_no_link(
    ) {
        let left_signature =
            encode_child_probe_temporal_signature_with_unlinked_runtime_topology_and_legacy_parent(
                400,
            );
        let right_signature =
            encode_child_probe_temporal_signature_with_unlinked_runtime_topology_and_legacy_parent(
                800,
            );

        assert_eq!(
            left_signature, right_signature,
            "expected runtime topology to own temporal parent identity for an unlinked child instead of letting stale RenderHybridGiProbe parent ids perturb temporal signature; left_signature={left_signature:.4}, right_signature={right_signature:.4}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_prefers_packed_rt_scene_truth_over_legacy_direct_rt_fallback() {
        let hierarchy_rt_lighting =
            encode_probe_rt_truth_with_conflicting_legacy_direct_rt_fallback();

        assert!(
            (hierarchy_rt_lighting[0] - hierarchy_rt_lighting[1]).abs() < 0.03
                && (hierarchy_rt_lighting[0] - hierarchy_rt_lighting[2]).abs() < 0.03,
            "expected packed hierarchy RT scene truth to stay authoritative over legacy direct probe_rt_lighting fallback instead of being recolored by it; hierarchy_rt_lighting={hierarchy_rt_lighting:?}"
        );
        assert!(
            hierarchy_rt_lighting[0] > 0.48
                && hierarchy_rt_lighting[0] < 0.56
                && hierarchy_rt_lighting[3] > 0.35,
            "expected packed hierarchy RT scene truth to preserve its gray runtime source and support; hierarchy_rt_lighting={hierarchy_rt_lighting:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_skips_matched_legacy_probe_slots_when_stripped_runtime_truth_exists()
    {
        let (probe_count, legacy_irradiance) =
            encode_probe_count_and_matched_legacy_irradiance_with_stripped_runtime_truth();

        assert_eq!(
            probe_count, 1,
            "expected stripped-scene-prepare runtime scene truth to drop matched authored-only legacy probe slots just like unmatched compatibility slots"
        );
        assert_eq!(
            legacy_irradiance,
            [0.0; 4],
            "expected matched authored-only legacy probe slot to stay zeroed once stripped runtime scene truth owns the frame; legacy_irradiance={legacy_irradiance:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_ignores_authored_probe_position_when_stripped_runtime_truth_exists()
    {
        let left_screen_data =
            encode_probe_screen_data_with_stripped_runtime_truth(Vec3::new(-0.9, 0.0, 0.0));
        let right_screen_data =
            encode_probe_screen_data_with_stripped_runtime_truth(Vec3::new(0.9, 0.0, 0.0));

        assert!(
            screen_data_delta(left_screen_data, right_screen_data) < 0.001,
            "expected stripped-scene-prepare runtime scene truth to stop using authored probe position/radius as final-composite support once runtime scene truth owns the probe; left_screen_data={left_screen_data:?}, right_screen_data={right_screen_data:?}"
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_temporal_signature_changes_when_rt_continuation_reblends_current_surface_cache_truth(
    ) {
        let warm =
            encode_probe_temporal_signature_with_rt_continuation_and_surface_cache_page_tint([
                240, 96, 48, 255,
            ]);
        let cool =
            encode_probe_temporal_signature_with_rt_continuation_and_surface_cache_page_tint([
                48, 96, 240, 255,
            ]);

        assert!(
            (warm.1[0] - cool.1[0]).abs() > 0.08 || (warm.1[2] - cool.1[2]).abs() > 0.08,
            "expected current-frame hierarchy RT lighting itself to change when continuation RT reblends different current surface-cache page tint; warm_rt={:?}, cool_rt={:?}",
            warm.1,
            cool.1
        );
        assert!(
            (warm.0 - cool.0).abs() > 0.01,
            "expected temporal signature to change when current GI changes through RT-continuation reblend against current surface-cache page tint, instead of only tracking irradiance-side proxy participation; warm_signature={:.4}, cool_signature={:.4}, warm_rt={:?}, cool_rt={:?}",
            warm.0,
            cool.0,
            warm.1,
            cool.1
        );
    }

    #[test]
    fn encode_hybrid_gi_probes_temporal_signature_changes_when_rt_continuation_reblends_surface_cache_owner_voxel_fallback(
    ) {
        let warm = encode_probe_temporal_signature_with_rt_continuation_and_surface_cache_owner_voxel_fallback_page_tint(
            [240, 96, 48, 255],
        );
        let cool = encode_probe_temporal_signature_with_rt_continuation_and_surface_cache_owner_voxel_fallback_page_tint(
            [48, 96, 240, 255],
        );

        assert!(
            (warm.1[0] - cool.1[0]).abs() > 0.08 || (warm.1[2] - cool.1[2]).abs() > 0.08,
            "expected current-frame hierarchy RT lighting itself to change when continuation RT reblends a different owner-card surface-cache sample through voxel fallback; warm_rt={:?}, cool_rt={:?}",
            warm.1,
            cool.1
        );
        assert!(
            (warm.0 - cool.0).abs() > 0.01,
            "expected temporal signature to change when current GI changes through RT-continuation reblend against owner-card surface-cache fallback carried by scene voxels, instead of only tracking direct page fallback; warm_signature={:.4}, cool_signature={:.4}, warm_rt={:?}, cool_rt={:?}",
            warm.0,
            cool.0,
            warm.1,
            cool.1
        );
    }

    fn encode_probe_temporal_scene_truth_confidence_with_runtime_support(support: f32) -> f32 {
        encode_probe_temporal_scene_truth_confidence_with_runtime_sources(support, false, 0.0)
    }

    fn encode_probe_temporal_signature_with_rt_continuation_and_surface_cache_page_tint(
        page_capture_sample_rgba: [u8; 4],
    ) -> (f32, [f32; 4]) {
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
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([(
                        probe.probe_id,
                        HybridGiResolveRuntime::pack_resolve_weight_q8(2.0),
                    )]))
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.6, 0.6, 0.6], 0.52),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([probe.probe_id]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_quality_q8(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_scene_truth_quality_q8(1.0),
                        )]),
                    )
                    .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.82, 0.34, 0.16], 0.44),
                        )]),
                    )
                    .build(),
            ))
            .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                    page_id: 11,
                    owner_card_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 1.0,
                    atlas_sample_rgba: page_capture_sample_rgba,
                    capture_sample_rgba: page_capture_sample_rgba,
                }],
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            }));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        (
            probes[0].temporal_signature_and_padding[0],
            probes[0].hierarchy_rt_lighting_rgb_and_weight,
        )
    }

    fn encode_probe_temporal_signature_with_rt_continuation_and_surface_cache_owner_voxel_fallback_page_tint(
        page_capture_sample_rgba: [u8; 4],
    ) -> (f32, [f32; 4]) {
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
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([(
                        probe.probe_id,
                        HybridGiResolveRuntime::pack_resolve_weight_q8(2.0),
                    )]))
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.6, 0.6, 0.6], 0.52),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([probe.probe_id]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_quality_q8(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_scene_truth_quality_q8(1.0),
                        )]),
                    )
                    .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.82, 0.34, 0.16], 0.44),
                        )]),
                    )
                    .build(),
            ))
            .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                    page_id: 11,
                    owner_card_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 1.0,
                    atlas_sample_rgba: page_capture_sample_rgba,
                    capture_sample_rgba: page_capture_sample_rgba,
                }],
                voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                    clipmap_id: 7,
                    center: Vec3::ZERO,
                    half_extent: 4.0,
                }],
                voxel_cells: [20_u32, 21, 24, 25]
                    .into_iter()
                    .map(|cell_index| HybridGiPrepareVoxelCell {
                        clipmap_id: 7,
                        cell_index,
                        occupancy_count: 4,
                        dominant_card_id: 11,
                        radiance_present: false,
                        radiance_rgb: [0, 0, 0],
                    })
                    .collect(),
            }));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        (
            probes[0].temporal_signature_and_padding[0],
            probes[0].hierarchy_rt_lighting_rgb_and_weight,
        )
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
            card_budget: 0,
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
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_parent_probes(std::collections::BTreeMap::from([(
                        child_probe.probe_id,
                        parent_probe.probe_id,
                    )]))
                    .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([(
                        runtime_probe_id,
                        HybridGiResolveRuntime::pack_resolve_weight_q8(0.9),
                    )]))
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            runtime_probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.6, 0.6, 0.6], 0.52),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([runtime_probe_id]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_quality_q8(
                        std::collections::BTreeMap::from([(
                            runtime_probe_id,
                            HybridGiResolveRuntime::pack_scene_truth_quality_q8(1.0),
                        )]),
                    )
                    .build(),
            ));
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

    fn encode_probe_irradiance_with_lineage_scene_truth_flag_without_source() -> [f32; 4] {
        let parent_probe = RenderHybridGiProbe {
            probe_id: 100,
            resident: false,
            ray_budget: 96,
            radius: 1.8,
            ..Default::default()
        };
        let child_probe = RenderHybridGiProbe {
            probe_id: 200,
            parent_probe_id: Some(parent_probe.probe_id),
            resident: true,
            ray_budget: 128,
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

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: child_probe.probe_id,
                    slot: 0,
                    ray_budget: child_probe.ray_budget,
                    irradiance_rgb: [240, 96, 48],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_parent_probes(std::collections::BTreeMap::from([(
                        child_probe.probe_id,
                        parent_probe.probe_id,
                    )]))
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([parent_probe.probe_id]),
                    )
                    .with_probe_scene_driven_hierarchy_rt_lighting_ids(
                        std::collections::BTreeSet::from([parent_probe.probe_id]),
                    )
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        probes[0].irradiance_and_intensity
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
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([(
                        probe.probe_id,
                        HybridGiResolveRuntime::pack_resolve_weight_q8(0.9),
                    )]))
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.6, 0.6, 0.6], 0.52),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([probe.probe_id]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_quality_q8(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_scene_truth_quality_q8(1.0),
                        )]),
                    )
                    .build(),
            ));
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
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([(
                        probe.probe_id,
                        HybridGiResolveRuntime::pack_resolve_weight_q8(0.9),
                    )]))
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight(
                                [0.6, 0.6, 0.6],
                                irradiance_support,
                            ),
                        )]),
                    )
                    .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight(
                                [0.6, 0.6, 0.6],
                                rt_support,
                            ),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([probe.probe_id]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_quality_q8(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_scene_truth_quality_q8(irradiance_quality),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_rt_lighting_ids(
                        includes_rt_scene_truth
                            .then_some(std::collections::BTreeSet::from([probe.probe_id]))
                            .unwrap_or_default(),
                    )
                    .with_probe_scene_driven_hierarchy_rt_lighting_quality_q8(
                        includes_rt_scene_truth
                            .then_some(std::collections::BTreeMap::from([(
                                probe.probe_id,
                                HybridGiResolveRuntime::pack_scene_truth_quality_q8(rt_quality),
                            )]))
                            .unwrap_or_default(),
                    )
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        probes[0].temporal_signature_and_padding[1]
    }

    fn encode_probe_temporal_signature_and_confidence_with_legacy_rt_scene_truth(
        rt_lighting_rgb: [u8; 3],
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
                    irradiance_rgb: [240, 96, 48],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_rt_lighting_rgb(std::collections::BTreeMap::from([(
                        probe.probe_id,
                        rt_lighting_rgb,
                    )]))
                    .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([(
                        probe.probe_id,
                        HybridGiResolveRuntime::pack_resolve_weight_q8(2.0),
                    )]))
                    .with_probe_scene_driven_hierarchy_rt_lighting_ids(
                        std::collections::BTreeSet::from([probe.probe_id]),
                    )
                    .with_probe_scene_driven_hierarchy_rt_lighting_quality_q8(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_scene_truth_quality_q8(1.0),
                        )]),
                    )
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        (
            probes[0].temporal_signature_and_padding[0],
            probes[0].temporal_signature_and_padding[1],
        )
    }

    fn encode_probe_hierarchy_irradiance_with_rt_only_stripped_runtime_truth() -> [f32; 4] {
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
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.92, 0.24, 0.12], 0.58),
                        )]),
                    )
                    .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.24, 0.48, 0.92], 0.58),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_rt_lighting_ids(
                        std::collections::BTreeSet::from([probe.probe_id]),
                    )
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        probes[0].hierarchy_irradiance_rgb_and_weight
    }

    fn encode_probe_hierarchy_irradiance_with_rt_only_scene_prepare_truth() -> [f32; 4] {
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
                surface_cache_page_contents: Vec::new(),
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.92, 0.24, 0.12], 0.58),
                        )]),
                    )
                    .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.24, 0.48, 0.92], 0.58),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_rt_lighting_ids(
                        std::collections::BTreeSet::from([probe.probe_id]),
                    )
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        probes[0].hierarchy_irradiance_rgb_and_weight
    }

    fn encode_probe_hierarchy_rt_lighting_with_irradiance_only_stripped_runtime_truth() -> [f32; 4]
    {
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
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.92, 0.24, 0.12], 0.58),
                        )]),
                    )
                    .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.24, 0.48, 0.92], 0.58),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([probe.probe_id]),
                    )
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        probes[0].hierarchy_rt_lighting_rgb_and_weight
    }

    fn encode_probe_hierarchy_rt_lighting_with_irradiance_only_scene_prepare_truth() -> [f32; 4] {
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
                surface_cache_page_contents: Vec::new(),
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.92, 0.24, 0.12], 0.58),
                        )]),
                    )
                    .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.24, 0.48, 0.92], 0.58),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([probe.probe_id]),
                    )
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        probes[0].hierarchy_rt_lighting_rgb_and_weight
    }

    fn encode_probe_count_and_unmatched_irradiance_with_stripped_runtime_truth() -> (u32, [f32; 4])
    {
        let source_probe = RenderHybridGiProbe {
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
            probe_budget: 2,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            probes: vec![source_probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: source_probe.probe_id,
                        slot: 0,
                        ray_budget: source_probe.ray_budget,
                        irradiance_rgb: [0, 0, 0],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 999,
                        slot: 1,
                        ray_budget: 128,
                        irradiance_rgb: [240, 96, 48],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            source_probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.6, 0.6, 0.6], 0.52),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([source_probe.probe_id]),
                    )
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        (probe_count, probes[1].irradiance_and_intensity)
    }

    fn encode_probe_count_and_unmatched_runtime_truth_slot_without_source(
    ) -> (u32, [f32; 4], [f32; 4], [f32; 4]) {
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
            probes: Vec::new(),
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 128,
                    irradiance_rgb: [240, 96, 48],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            200,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.6, 0.6, 0.6], 0.52),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([200]),
                    )
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        (
            probe_count,
            probes[0].screen_uv_and_radius,
            probes[0].irradiance_and_intensity,
            probes[0].hierarchy_irradiance_rgb_and_weight,
        )
    }

    fn encode_child_probe_with_runtime_parent_truth_without_legacy_extract_container(
    ) -> (u32, [f32; 4]) {
        let child_probe_id = 300;
        let parent_probe_id = 200;
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
        let extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: child_probe_id,
                    slot: 0,
                    ray_budget: 128,
                    irradiance_rgb: [32, 32, 32],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_parent_probes(std::collections::BTreeMap::from([(
                        child_probe_id,
                        parent_probe_id,
                    )]))
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            parent_probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.68, 0.08, 0.06], 0.58),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([parent_probe_id]),
                    )
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        (probe_count, probes[0].hierarchy_irradiance_rgb_and_weight)
    }

    fn encode_child_probe_with_runtime_parent_truth_and_stale_legacy_extract_topology(
    ) -> ([f32; 4], [f32; 4], [f32; 4]) {
        let child_probe_id = 300;
        let runtime_parent_probe_id = 200;
        let stale_parent_probe_id = 400;
        let child_probe = RenderHybridGiProbe {
            probe_id: child_probe_id,
            parent_probe_id: Some(stale_parent_probe_id),
            resident: true,
            ray_budget: 128,
            position: Vec3::new(-0.8, 0.0, 0.0),
            radius: 1.8,
            ..Default::default()
        };
        let stale_parent_probe = RenderHybridGiProbe {
            probe_id: stale_parent_probe_id,
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
            probe_budget: 2,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            probes: vec![child_probe, stale_parent_probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: child_probe_id,
                    slot: 0,
                    ray_budget: 128,
                    irradiance_rgb: [240, 96, 48],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_parent_probes(std::collections::BTreeMap::from([(
                        child_probe_id,
                        runtime_parent_probe_id,
                    )]))
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            runtime_parent_probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.68, 0.08, 0.06], 0.58),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([runtime_parent_probe_id]),
                    )
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        (
            probes[0].screen_uv_and_radius,
            probes[0].irradiance_and_intensity,
            probes[0].hierarchy_irradiance_rgb_and_weight,
        )
    }

    fn encode_child_probe_with_stale_legacy_parent_truth_and_unlinked_runtime_topology(
    ) -> ([f32; 4], [f32; 4]) {
        encode_child_probe_with_stale_legacy_parent_truth_and_runtime_scene_truth_topology(
            std::collections::BTreeMap::from([(900, 901)]),
        )
    }

    fn encode_child_probe_with_stale_legacy_parent_truth_and_flat_runtime_scene_truth_topology(
    ) -> ([f32; 4], [f32; 4]) {
        encode_child_probe_with_stale_legacy_parent_truth_and_runtime_scene_truth_topology(
            std::collections::BTreeMap::new(),
        )
    }

    fn encode_child_probe_with_stale_legacy_parent_truth_and_runtime_scene_truth_topology(
        probe_parent_probes: std::collections::BTreeMap<u32, u32>,
    ) -> ([f32; 4], [f32; 4]) {
        let child_probe_id = 300;
        let stale_parent_probe_id = 400;
        let child_probe = RenderHybridGiProbe {
            probe_id: child_probe_id,
            parent_probe_id: Some(stale_parent_probe_id),
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let stale_parent_probe = RenderHybridGiProbe {
            probe_id: stale_parent_probe_id,
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
            probe_budget: 2,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            probes: vec![child_probe, stale_parent_probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: child_probe_id,
                    slot: 0,
                    ray_budget: 128,
                    irradiance_rgb: [240, 96, 48],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_parent_probes(probe_parent_probes)
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            stale_parent_probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.68, 0.08, 0.06], 0.58),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([stale_parent_probe_id]),
                    )
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        (
            probes[0].irradiance_and_intensity,
            probes[0].hierarchy_irradiance_rgb_and_weight,
        )
    }

    fn encode_child_probe_with_stale_legacy_parent_continuation_and_unlinked_runtime_topology(
    ) -> [f32; 4] {
        let child_probe_id = 300;
        let stale_parent_probe_id = 400;
        let stale_grandparent_probe_id = 500;
        let child_probe = RenderHybridGiProbe {
            probe_id: child_probe_id,
            parent_probe_id: Some(stale_parent_probe_id),
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let stale_parent_probe = RenderHybridGiProbe {
            probe_id: stale_parent_probe_id,
            parent_probe_id: Some(stale_grandparent_probe_id),
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let stale_grandparent_probe = RenderHybridGiProbe {
            probe_id: stale_grandparent_probe_id,
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
            probe_budget: 3,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            probes: vec![child_probe, stale_parent_probe, stale_grandparent_probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: child_probe_id,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [0, 0, 0],
                    },
                    HybridGiPrepareProbe {
                        probe_id: stale_parent_probe_id,
                        slot: 1,
                        ray_budget: 128,
                        irradiance_rgb: [0, 0, 0],
                    },
                    HybridGiPrepareProbe {
                        probe_id: stale_grandparent_probe_id,
                        slot: 2,
                        ray_budget: 128,
                        irradiance_rgb: [240, 96, 48],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_parent_probes(std::collections::BTreeMap::from([(900, 901)]))
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 3, "expected three encoded probes");
        probes[0].hierarchy_irradiance_rgb_and_weight
    }

    fn encode_child_probe_temporal_signature_with_unlinked_runtime_topology_and_legacy_parent(
        stale_parent_probe_id: u32,
    ) -> f32 {
        let child_probe_id = 300;
        let child_probe = RenderHybridGiProbe {
            probe_id: child_probe_id,
            parent_probe_id: Some(stale_parent_probe_id),
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let stale_parent_probe = RenderHybridGiProbe {
            probe_id: stale_parent_probe_id,
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
            probe_budget: 2,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            probes: vec![child_probe, stale_parent_probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: child_probe_id,
                    slot: 0,
                    ray_budget: 128,
                    irradiance_rgb: [96, 96, 96],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_parent_probes(std::collections::BTreeMap::from([(900, 901)]))
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        probes[0].temporal_signature_and_padding[0]
    }

    fn encode_child_probe_with_runtime_parent_truth_and_empty_legacy_extract_topology(
    ) -> (u32, [f32; 4]) {
        let child_probe_id = 300;
        let parent_probe_id = 200;
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
            probes: Vec::new(),
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: child_probe_id,
                    slot: 0,
                    ray_budget: 128,
                    irradiance_rgb: [32, 32, 32],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_parent_probes(std::collections::BTreeMap::from([(
                        child_probe_id,
                        parent_probe_id,
                    )]))
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            parent_probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.68, 0.08, 0.06], 0.58),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([parent_probe_id]),
                    )
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        (probe_count, probes[0].hierarchy_irradiance_rgb_and_weight)
    }

    fn encode_probe_count_and_rt_truth_without_legacy_extract_container(
    ) -> (u32, [f32; 4], [f32; 4], [f32; 4]) {
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
        let extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 128,
                    irradiance_rgb: [240, 96, 48],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            200,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.52, 0.52, 0.52], 0.42),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_rt_lighting_ids(
                        std::collections::BTreeSet::from([200]),
                    )
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        (
            probe_count,
            probes[0].screen_uv_and_radius,
            probes[0].irradiance_and_intensity,
            probes[0].hierarchy_rt_lighting_rgb_and_weight,
        )
    }

    fn encode_probe_rt_truth_with_conflicting_legacy_direct_rt_fallback() -> [f32; 4] {
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
        let extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 128,
                    irradiance_rgb: [240, 96, 48],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            200,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.52, 0.52, 0.52], 0.42),
                        )]),
                    )
                    .with_probe_rt_lighting_rgb(std::collections::BTreeMap::from([(
                        200,
                        [255, 16, 16],
                    )]))
                    .with_probe_scene_driven_hierarchy_rt_lighting_ids(
                        std::collections::BTreeSet::from([200]),
                    )
                    .build(),
            ));

        let (probes, _, _) = encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        probes[0].hierarchy_rt_lighting_rgb_and_weight
    }

    fn encode_probe_count_and_matched_legacy_irradiance_with_stripped_runtime_truth(
    ) -> (u32, [f32; 4]) {
        let scene_truth_probe = RenderHybridGiProbe {
            probe_id: 200,
            resident: true,
            ray_budget: 128,
            radius: 1.8,
            ..Default::default()
        };
        let legacy_probe = RenderHybridGiProbe {
            probe_id: 999,
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
            probe_budget: 2,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            probes: vec![scene_truth_probe, legacy_probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: scene_truth_probe.probe_id,
                        slot: 0,
                        ray_budget: scene_truth_probe.ray_budget,
                        irradiance_rgb: [0, 0, 0],
                    },
                    HybridGiPrepareProbe {
                        probe_id: legacy_probe.probe_id,
                        slot: 1,
                        ray_budget: legacy_probe.ray_budget,
                        irradiance_rgb: [240, 96, 48],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            scene_truth_probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.6, 0.6, 0.6], 0.52),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([scene_truth_probe.probe_id]),
                    )
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        (probe_count, probes[1].irradiance_and_intensity)
    }

    fn encode_probe_screen_data_with_stripped_runtime_truth(position: Vec3) -> [f32; 4] {
        let source_probe = RenderHybridGiProbe {
            probe_id: 200,
            resident: true,
            ray_budget: 128,
            position,
            radius: 1.8,
            ..Default::default()
        };
        let mut camera = ViewportCameraSnapshot {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 4.0),
                ..Transform::default()
            },
            projection_mode: ProjectionMode::Orthographic,
            ortho_size: 1.2,
            ..ViewportCameraSnapshot::default()
        };
        camera.apply_viewport_size(UVec2::new(32, 32));
        let snapshot = RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera,
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
            probes: vec![source_probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: source_probe.probe_id,
                    slot: 0,
                    ray_budget: source_probe.ray_budget,
                    irradiance_rgb: [0, 0, 0],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            source_probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.6, 0.6, 0.6], 0.52),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([source_probe.probe_id]),
                    )
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        probes[0].screen_uv_and_radius
    }

    fn screen_data_delta(lhs: [f32; 4], rhs: [f32; 4]) -> f32 {
        lhs.into_iter()
            .zip(rhs)
            .map(|(lhs, rhs)| (lhs - rhs).abs())
            .fold(0.0_f32, f32::max)
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
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([(
                        runtime_probe_id,
                        HybridGiResolveRuntime::pack_resolve_weight_q8(0.9),
                    )]))
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            runtime_probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.6, 0.6, 0.6], support),
                        )]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_ids(
                        std::collections::BTreeSet::from([runtime_probe_id]),
                    )
                    .with_probe_scene_driven_hierarchy_irradiance_quality_q8(
                        std::collections::BTreeMap::from([(
                            runtime_probe_id,
                            HybridGiResolveRuntime::pack_scene_truth_quality_q8(1.0),
                        )]),
                    )
                    .build(),
            ));

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

    fn encode_probe_temporal_scene_truth_confidence_with_surface_cache_proxy_and_stale_trace_id(
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
            trace_regions: Vec::new(),
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
                scheduled_trace_region_ids: vec![404],
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_irradiance_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.24, 0.24, 0.24], 0.32),
                        )]),
                    )
                    .build(),
            ))
            .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                    page_id: 11,
                    owner_card_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 1.8,
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

    fn encode_probe_trace_count_with_stale_scheduled_ids_before_live_payload() -> u32 {
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

        let (_, _, trace_count) = encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        trace_count
    }

    fn encode_probe_trace_count_with_duplicate_live_scheduled_payload() -> u32 {
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

        let (_, _, trace_count) = encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        trace_count
    }

    fn encode_probe_trace_count_with_flat_runtime() -> u32 {
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

        let (_, _, trace_count) = encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        trace_count
    }

    fn encode_probe_count_with_flat_runtime() -> (u32, [f32; 4]) {
        let probe = RenderHybridGiProbe {
            probe_id: 200,
            resident: true,
            ray_budget: 128,
            position: Vec3::ZERO,
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
                    irradiance_rgb: [240, 96, 48],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime::default()));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        (probe_count, probes[0].irradiance_and_intensity)
    }

    fn encode_probe_count_with_flat_runtime_exact_payload() -> (u32, [f32; 4]) {
        let probe = RenderHybridGiProbe {
            probe_id: 200,
            resident: true,
            ray_budget: 128,
            position: Vec3::ZERO,
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
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_rt_lighting_rgb(std::collections::BTreeMap::from([(
                        probe.probe_id,
                        [240, 96, 48],
                    )]))
                    .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([(
                        probe.probe_id,
                        HybridGiResolveRuntime::pack_resolve_weight_q8(2.0),
                    )]))
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        (probe_count, probes[0].hierarchy_rt_lighting_rgb_and_weight)
    }

    fn encode_probe_resolve_weight_with_flat_runtime_exact_weight(resolve_weight: f32) -> f32 {
        let probe = RenderHybridGiProbe {
            probe_id: 200,
            resident: true,
            ray_budget: 128,
            position: Vec3::ZERO,
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
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([(
                        probe.probe_id,
                        HybridGiResolveRuntime::pack_resolve_weight_q8(resolve_weight),
                    )]))
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 1, "expected one encoded probe");
        probes[0].irradiance_and_intensity[3]
    }

    fn encode_child_probe_resolve_weight_without_runtime_parent(link_child: bool) -> f32 {
        let parent_probe = RenderHybridGiProbe {
            probe_id: 200,
            resident: true,
            ray_budget: 128,
            position: Vec3::ZERO,
            radius: 2.2,
            ..Default::default()
        };
        let child_probe = RenderHybridGiProbe {
            probe_id: 300,
            parent_probe_id: link_child.then_some(parent_probe.probe_id),
            resident: true,
            ray_budget: 128,
            position: Vec3::ZERO,
            radius: 2.2,
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
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: parent_probe.probe_id,
                        slot: 0,
                        ray_budget: parent_probe.ray_budget,
                        irradiance_rgb: [255, 80, 40],
                    },
                    HybridGiPrepareProbe {
                        probe_id: child_probe.probe_id,
                        slot: 1,
                        ray_budget: child_probe.ray_budget,
                        irradiance_rgb: [40, 96, 255],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        assert_eq!(probe_count, 2, "expected parent and child probes to encode");
        probes[1].irradiance_and_intensity[3]
    }

    fn encode_probe_count_with_budgeted_scene_representation_and_legacy_probe_slot(
    ) -> (u32, [f32; 4]) {
        let probe = RenderHybridGiProbe {
            probe_id: 200,
            resident: true,
            ray_budget: 128,
            position: Vec3::ZERO,
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
            trace_budget: 1,
            card_budget: 1,
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
                    irradiance_rgb: [240, 96, 48],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        (probe_count, probes[0].irradiance_and_intensity)
    }

    fn encode_probe_count_with_flat_runtime_and_scene_prepare() -> (u32, [f32; 4]) {
        let probe = RenderHybridGiProbe {
            probe_id: 200,
            resident: true,
            ray_budget: 128,
            position: Vec3::ZERO,
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
                    irradiance_rgb: [240, 96, 48],
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
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime::default()));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        (probe_count, probes[0].irradiance_and_intensity)
    }

    fn encode_probe_count_with_unrelated_runtime_topology_and_scene_prepare() -> (u32, [f32; 4]) {
        let probe = RenderHybridGiProbe {
            probe_id: 200,
            resident: true,
            ray_budget: 128,
            position: Vec3::ZERO,
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
                    irradiance_rgb: [240, 96, 48],
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
                    .with_probe_parent_probes(std::collections::BTreeMap::from([(301, 300)]))
                    .build(),
            ));

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, None);
        (probe_count, probes[0].irradiance_and_intensity)
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
        let resources = include_capture_resource.then(|| {
            let mut resources = HybridGiScenePrepareResourcesSnapshot::new(
                1,
                Vec::new(),
                vec![3],
                vec![4],
                4,
                4,
                (16, 16),
                (16, 16),
                1,
            );
            resources.store_texture_slot_rgba_samples(
                vec![(3, [64, 96, 128, 255])],
                vec![(4, [180, 140, 96, 255])],
            );
            resources
        });

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, resources.as_ref());
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        probes[0].temporal_signature_and_padding[1]
    }

    fn encode_probe_temporal_scene_truth_confidence_with_rt_continuation_surface_cache_proxy(
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
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([(
                        probe.probe_id,
                        HybridGiResolveRuntime::pack_resolve_weight_q8(0.9),
                    )]))
                    .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                        std::collections::BTreeMap::from([(
                            probe.probe_id,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.82, 0.34, 0.16], 0.44),
                        )]),
                    )
                    .build(),
            ))
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
        let resources = include_capture_resource.then(|| {
            let mut resources = HybridGiScenePrepareResourcesSnapshot::new(
                1,
                Vec::new(),
                vec![3],
                vec![4],
                4,
                4,
                (16, 16),
                (16, 16),
                1,
            );
            resources.store_texture_slot_rgba_samples(
                vec![(3, [64, 96, 128, 255])],
                vec![(4, [180, 140, 96, 255])],
            );
            resources
        });

        let (probes, probe_count, _) =
            encode_hybrid_gi_probes(&frame, UVec2::new(32, 32), true, resources.as_ref());
        assert_eq!(probe_count, 1, "expected exactly one encoded probe");
        probes[0].temporal_signature_and_padding[1]
    }
}
