use crate::core::framework::render::{
    RenderHybridGiPreparedFrame, RenderHybridGiPreparedProbe, RenderHybridGiPreparedProbeSceneData,
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
        for prepared_probe in &prepared_frame.resident_probes {
            if count >= MAX_HYBRID_GI_PROBES {
                break;
            }
            let Some(gpu_probe) = project_prepared_hybrid_gi_probe(
                prepared_probe,
                prepared_frame,
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
    view_proj: Mat4,
    camera_position: Vec3,
) -> Option<GpuHybridGiProbe> {
    let scene_data = prepared_frame
        .probe_scene_data
        .iter()
        .find(|scene_data| scene_data.probe_id == probe.probe_id)?;
    let position = dequantized_probe_position(scene_data);
    let radius = dequantized_probe_radius(scene_data);
    let (uv_x, uv_y) = project_screen_uv(view_proj, position)?;
    let screen_radius = projected_screen_radius(radius, position, camera_position);
    let budget_weight = ((probe.ray_budget.max(1) as f32) / 128.0).clamp(0.25, 1.5);
    let temporal_signature = probe_temporal_signature(probe, prepared_frame);
    let irradiance = rgb8_to_unit(probe.irradiance_rgb);
    let rt_lighting = prepared_frame
        .probe_rt_lighting_rgb
        .iter()
        .find(|rt_lighting| rt_lighting.probe_id == probe.probe_id)
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
}
