use crate::core::framework::render::{
    RenderHybridGiPreparedFrame, RenderHybridGiPreparedProbe,
    RenderHybridGiPreparedProbeRtLighting, RenderHybridGiPreparedProbeSceneData,
};
use crate::core::math::{Mat4, UVec2, Vec3};
use bytemuck::Zeroable;

use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::constants::MAX_HYBRID_GI_PROBES;
use super::super::super::super::hybrid_gi_probe_gpu::GpuHybridGiProbe;
use super::super::camera_matrices::view_projection;

const HYBRID_GI_POSITION_BIAS: i32 = 2048;
const HYBRID_GI_POSITION_SCALE: f32 = 64.0;
const HYBRID_GI_RADIUS_SCALE: f32 = 64.0;
const PREPARED_RT_LIGHTING_WEIGHT: f32 = 0.75;

struct PreparedProbeSidebandLookup<'a> {
    scene_data: &'a [RenderHybridGiPreparedProbeSceneData],
    rt_lighting: &'a [RenderHybridGiPreparedProbeRtLighting],
    scene_data_is_canonical: bool,
    rt_lighting_is_canonical: bool,
}

impl<'a> PreparedProbeSidebandLookup<'a> {
    fn new(prepared_frame: &'a RenderHybridGiPreparedFrame) -> Self {
        let scene_data = prepared_frame.probe_scene_data.as_slice();
        let rt_lighting = prepared_frame.probe_rt_lighting_rgb.as_slice();
        Self {
            scene_data,
            rt_lighting,
            scene_data_is_canonical: strictly_increasing_by_key(scene_data, scene_data_probe_id),
            rt_lighting_is_canonical: strictly_increasing_by_key(rt_lighting, rt_lighting_probe_id),
        }
    }

    fn scene_data(&self, probe_id: u32) -> Option<&RenderHybridGiPreparedProbeSceneData> {
        lookup_by_probe_id(
            self.scene_data,
            probe_id,
            self.scene_data_is_canonical,
            scene_data_probe_id,
        )
    }

    fn rt_lighting(&self, probe_id: u32) -> Option<&RenderHybridGiPreparedProbeRtLighting> {
        lookup_by_probe_id(
            self.rt_lighting,
            probe_id,
            self.rt_lighting_is_canonical,
            rt_lighting_probe_id,
        )
    }
}

pub(in super::super) fn encode_hybrid_gi_probes(
    frame: &ViewportRenderFrame,
    viewport_size: UVec2,
    enabled: bool,
) -> ([GpuHybridGiProbe; MAX_HYBRID_GI_PROBES], u32) {
    let mut probes = [GpuHybridGiProbe::zeroed(); MAX_HYBRID_GI_PROBES];
    if !enabled {
        return (probes, 0);
    }

    let Some(_hybrid_gi) = frame
        .extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .filter(|extract| extract.enabled)
    else {
        return (probes, 0);
    };

    let camera = &frame.extract.view.camera;
    let view_proj = view_projection(camera, viewport_size);
    let camera_position = camera.transform.translation;
    let mut count = 0;

    if let Some(prepared_frame) = frame
        .prepared_runtime_sidebands()
        .hybrid_gi_prepared_frame()
    {
        let sidebands = PreparedProbeSidebandLookup::new(prepared_frame);
        for prepared_probe in &prepared_frame.resident_probes {
            if count >= MAX_HYBRID_GI_PROBES {
                break;
            }
            let Some(gpu_probe) = project_prepared_hybrid_gi_probe(
                prepared_probe,
                prepared_frame,
                &sidebands,
                view_proj,
                camera_position,
            ) else {
                continue;
            };
            probes[count] = gpu_probe;
            count += 1;
        }
    }

    (probes, count as u32)
}

fn project_prepared_hybrid_gi_probe(
    probe: &RenderHybridGiPreparedProbe,
    prepared_frame: &RenderHybridGiPreparedFrame,
    sidebands: &PreparedProbeSidebandLookup<'_>,
    view_proj: Mat4,
    camera_position: Vec3,
) -> Option<GpuHybridGiProbe> {
    let scene_data = sidebands.scene_data(probe.probe_id)?;
    let position = dequantized_probe_position(scene_data);
    let radius = dequantized_probe_radius(scene_data);
    let (uv_x, uv_y) = project_screen_uv(view_proj, position)?;
    let screen_radius = projected_screen_radius(radius, position, camera_position);
    let budget_weight = ((probe.ray_budget.max(1) as f32) / 128.0).clamp(0.25, 1.5);
    let temporal_signature = probe_temporal_signature(probe, prepared_frame);
    let irradiance = rgb8_to_unit(probe.irradiance_rgb);
    let rt_lighting = sidebands
        .rt_lighting(probe.probe_id)
        .map(|rt_lighting| rgb8_to_unit(rt_lighting.rt_lighting_rgb))
        .unwrap_or([0.0; 3]);
    let rt_lighting_weight = if rt_lighting.iter().any(|channel| *channel > 0.0) {
        PREPARED_RT_LIGHTING_WEIGHT
    } else {
        0.0
    };

    Some(GpuHybridGiProbe {
        screen_uv_and_radius: [uv_x, uv_y, screen_radius, budget_weight],
        irradiance_and_intensity: [irradiance[0], irradiance[1], irradiance[2], 1.0],
        hierarchy_irradiance_rgb_and_weight: [0.0, 0.0, 0.0, 0.0],
        hierarchy_rt_lighting_rgb_and_weight: [
            rt_lighting[0],
            rt_lighting[1],
            rt_lighting[2],
            rt_lighting_weight,
        ],
        temporal_signature_and_padding: [
            temporal_signature,
            1.0,
            probe.source_mask as f32,
            f32::from(probe.dynamic_weight_q8) / 255.0,
        ],
    })
}

fn lookup_by_probe_id<T>(
    entries: &[T],
    probe_id: u32,
    is_canonical: bool,
    key: fn(&T) -> u32,
) -> Option<&T> {
    if is_canonical {
        let index = entries.binary_search_by_key(&probe_id, key).ok()?;
        return entries.get(index);
    }
    entries.iter().find(|entry| key(entry) == probe_id)
}

fn strictly_increasing_by_key<T>(entries: &[T], key: fn(&T) -> u32) -> bool {
    entries.windows(2).all(|pair| key(&pair[0]) < key(&pair[1]))
}

fn scene_data_probe_id(entry: &RenderHybridGiPreparedProbeSceneData) -> u32 {
    entry.probe_id
}

fn rt_lighting_probe_id(entry: &RenderHybridGiPreparedProbeRtLighting) -> u32 {
    entry.probe_id
}

fn probe_temporal_signature(
    probe: &RenderHybridGiPreparedProbe,
    prepared_frame: &RenderHybridGiPreparedFrame,
) -> f32 {
    let policy = prepared_frame.composite_policy;
    let generation = policy.baked_light_set_generation().unwrap_or_default();
    let mut signature = probe.probe_id
        ^ probe.stable_instance_key as u32
        ^ (probe.stable_instance_key >> 32) as u32
        ^ probe.source_mask.rotate_left(7)
        ^ (policy.participation_epoch() as u32).rotate_left(13)
        ^ (generation as u32).rotate_left(19)
        ^ ((generation >> 32) as u32).rotate_left(23);
    signature ^= signature >> 16;
    signature = signature.wrapping_mul(0x7FEB_352D);
    signature ^= signature >> 15;
    let bucket = signature % 1023 + 1;
    bucket as f32 / 1024.0
}

fn project_screen_uv(view_proj: Mat4, position: Vec3) -> Option<(f32, f32)> {
    let clip = view_proj * position.extend(1.0);
    if clip.w.abs() <= f32::EPSILON {
        return None;
    }

    let ndc = clip.truncate() / clip.w;
    if ndc.z < -1.0 || ndc.z > 1.0 {
        return None;
    }

    Some((
        (0.5 + ndc.x * 0.5).clamp(0.0, 1.0),
        (0.5 - ndc.y * 0.5).clamp(0.0, 1.0),
    ))
}

fn projected_screen_radius(radius: f32, position: Vec3, camera_position: Vec3) -> f32 {
    let distance = (camera_position - position).length().max(1.0);
    (radius.max(0.05) / distance).clamp(0.04, 0.75)
}

fn dequantized_probe_position(scene_data: &RenderHybridGiPreparedProbeSceneData) -> Vec3 {
    Vec3::new(
        dequantized_signed(scene_data.position_x_q),
        dequantized_signed(scene_data.position_y_q),
        dequantized_signed(scene_data.position_z_q),
    )
}

fn dequantized_signed(value: u32) -> f32 {
    (value as i32 - HYBRID_GI_POSITION_BIAS) as f32 / HYBRID_GI_POSITION_SCALE
}

fn dequantized_probe_radius(scene_data: &RenderHybridGiPreparedProbeSceneData) -> f32 {
    scene_data.radius_q as f32 / HYBRID_GI_RADIUS_SCALE
}

fn rgb8_to_unit(rgb: [u8; 3]) -> [f32; 3] {
    [
        rgb[0] as f32 / 255.0,
        rgb[1] as f32 / 255.0,
        rgb[2] as f32 / 255.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        RenderFrameExtract, RenderHybridGiExtract, RenderHybridGiPreparedProbeRtLighting,
        RenderPreparedRuntimeSidebands,
    };
    use crate::core::math::UVec2;
    use crate::graphics::ViewportRenderFrame;
    use crate::scene::world::World;

    #[test]
    fn hybrid_gi_probe_encoder_returns_no_resources_when_disabled() {
        let frame = ViewportRenderFrame::from_extract(
            World::new().to_render_frame_extract(),
            UVec2::new(160, 120),
        );

        let (_, probe_count) = encode_hybrid_gi_probes(&frame, UVec2::new(160, 120), false);

        assert_eq!(probe_count, 0);
    }

    #[test]
    fn hybrid_gi_probe_encoder_requires_prepared_scene_probe_sideband() {
        let frame = ViewportRenderFrame::from_extract(
            hybrid_gi_scene_representation_extract(),
            UVec2::new(160, 120),
        );

        let (_, probe_count) = encode_hybrid_gi_probes(&frame, UVec2::new(160, 120), true);

        assert_eq!(probe_count, 0);
    }

    #[test]
    fn hybrid_gi_sideband_lookup_preserves_reordered_duplicate_first_match() {
        let prepared = RenderHybridGiPreparedFrame {
            probe_scene_data: vec![scene_data(9, 90), scene_data(7, 70), scene_data(7, 71)],
            probe_rt_lighting_rgb: vec![
                rt_lighting(9, [90, 0, 0]),
                rt_lighting(7, [70, 0, 0]),
                rt_lighting(7, [71, 0, 0]),
            ],
            ..RenderHybridGiPreparedFrame::default()
        };

        let lookup = PreparedProbeSidebandLookup::new(&prepared);

        assert!(!lookup.scene_data_is_canonical);
        assert!(!lookup.rt_lighting_is_canonical);
        assert_eq!(
            lookup.scene_data(7).map(|entry| entry.position_x_q),
            Some(70)
        );
        assert_eq!(
            lookup.rt_lighting(7).map(|entry| entry.rt_lighting_rgb),
            Some([70, 0, 0])
        );
    }

    #[test]
    fn optimization_batch_20260830dv_hybrid_gi_sidebands_use_canonical_binary_lookup() {
        let prepared = RenderHybridGiPreparedFrame {
            probe_scene_data: vec![scene_data(3, 30), scene_data(7, 70), scene_data(9, 90)],
            probe_rt_lighting_rgb: vec![
                rt_lighting(3, [30, 0, 0]),
                rt_lighting(7, [70, 0, 0]),
                rt_lighting(9, [90, 0, 0]),
            ],
            ..RenderHybridGiPreparedFrame::default()
        };
        let lookup = PreparedProbeSidebandLookup::new(&prepared);

        assert!(lookup.scene_data_is_canonical);
        assert!(lookup.rt_lighting_is_canonical);
        assert_eq!(
            lookup.scene_data(7).map(|entry| entry.position_x_q),
            Some(70)
        );
        assert_eq!(
            lookup.rt_lighting(7).map(|entry| entry.rt_lighting_rgb),
            Some([70, 0, 0])
        );

        let source = include_str!("encode.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        assert!(production.contains("binary_search_by_key"));
        assert!(production.contains("strictly_increasing_by_key"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830dv_hybrid_gi_sideband_lookup_evidence() {
        const FRAME_COUNT: usize = 32_768;
        const PROBE_COUNT: usize = MAX_HYBRID_GI_PROBES;
        const MARKER: &str = "RUNTIME531_HYBRID_GI_SIDEBAND_BINARY_LOOKUP_BENCH_V1";

        let legacy_checks_per_frame = PROBE_COUNT
            .saturating_mul(PROBE_COUNT.saturating_add(1))
            .saturating_mul(2)
            / 2;
        let comparisons_per_lookup = usize::BITS as usize - PROBE_COUNT.leading_zeros() as usize;
        let indexed_checks_per_frame = PROBE_COUNT
            .saturating_sub(1)
            .saturating_mul(2)
            .saturating_add(
                PROBE_COUNT
                    .saturating_mul(comparisons_per_lookup)
                    .saturating_mul(2),
            );
        let legacy_candidate_checks = FRAME_COUNT.saturating_mul(legacy_checks_per_frame);
        let indexed_candidate_checks = FRAME_COUNT.saturating_mul(indexed_checks_per_frame);
        let reduction_bps = legacy_candidate_checks
            .saturating_sub(indexed_candidate_checks)
            .saturating_mul(10_000)
            / legacy_candidate_checks.max(1);

        assert!(
            indexed_candidate_checks.saturating_mul(100)
                <= legacy_candidate_checks.saturating_mul(70)
        );
        println!(
            "{MARKER} frames={FRAME_COUNT} probes={PROBE_COUNT} \
             legacy_candidate_checks={legacy_candidate_checks} \
             indexed_candidate_checks_upper_bound={indexed_candidate_checks} \
             comparisons_per_lookup={comparisons_per_lookup} reduction_bps={reduction_bps}"
        );
    }

    #[test]
    fn hybrid_gi_probe_encoder_projects_prepared_runtime_screen_probe_sideband() {
        let frame = ViewportRenderFrame::from_extract(
            hybrid_gi_scene_representation_extract(),
            UVec2::new(160, 120),
        )
        .with_prepared_runtime_sidebands(
            RenderPreparedRuntimeSidebands::default().with_hybrid_gi_prepared_frame(Some(
                RenderHybridGiPreparedFrame {
                    resident_probes: vec![RenderHybridGiPreparedProbe {
                        probe_id: 7,
                        slot: 0,
                        stable_instance_key: 77,
                        source_mask: crate::core::framework::render::HYBRID_GI_SOURCE_FULL_DYNAMIC,
                        dynamic_weight_q8: u8::MAX,
                        ray_budget: 1,
                        irradiance_rgb: [32, 40, 48],
                    }],
                    probe_scene_data: vec![RenderHybridGiPreparedProbeSceneData {
                        probe_id: 7,
                        position_x_q: 2048,
                        position_y_q: 2048,
                        position_z_q: 2048,
                        radius_q: 96,
                    }],
                    probe_rt_lighting_rgb: vec![RenderHybridGiPreparedProbeRtLighting {
                        probe_id: 7,
                        rt_lighting_rgb: [240, 64, 32],
                    }],
                    ..RenderHybridGiPreparedFrame::default()
                },
            )),
        );

        let (probes, probe_count) = encode_hybrid_gi_probes(&frame, UVec2::new(160, 120), true);

        assert_eq!(probe_count, 1);
        assert!(probes[0].screen_uv_and_radius[2] > 0.0);
        assert_eq!(probes[0].irradiance_and_intensity[0], 32.0_f32 / 255.0);
        assert_eq!(
            probes[0].hierarchy_rt_lighting_rgb_and_weight[0],
            240.0_f32 / 255.0
        );
        assert!(probes[0].hierarchy_rt_lighting_rgb_and_weight[3] > 0.0);
    }

    fn hybrid_gi_scene_representation_extract() -> RenderFrameExtract {
        let world = World::new();
        let mut extract = world.to_render_frame_extract();
        extract.apply_viewport_size(UVec2::new(160, 120));
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            trace_budget: 2,
            card_budget: 1,
            voxel_budget: 1,
            ..RenderHybridGiExtract::default()
        });
        extract
    }

    fn scene_data(probe_id: u32, position_x_q: u32) -> RenderHybridGiPreparedProbeSceneData {
        RenderHybridGiPreparedProbeSceneData {
            probe_id,
            position_x_q,
            position_y_q: 2048,
            position_z_q: 2048,
            radius_q: 96,
        }
    }

    fn rt_lighting(
        probe_id: u32,
        rt_lighting_rgb: [u8; 3],
    ) -> RenderHybridGiPreparedProbeRtLighting {
        RenderHybridGiPreparedProbeRtLighting {
            probe_id,
            rt_lighting_rgb,
        }
    }
}
