use super::*;
use crate::core::framework::render::{
    CookieProjection, CookieWrapMode, LightShadowSettings, ShadowPcfQuality, ShadowResolutionTier,
    DEFAULT_RENDER_LAYER_MASK,
};
use crate::core::resource::ResourceId;
use std::mem::{offset_of, size_of};

#[test]
fn render_cookie_gpu_light_data_extension_offsets() {
    assert_eq!(size_of::<GpuLightData>(), 128);
    assert_eq!(offset_of!(GpuLightData, cookie_uv_rect), 96);
    assert_eq!(offset_of!(GpuLightData, cookie_misc), 112);
    assert_eq!(GpuLightData::default().cookie_uv_rect, [0.0; 4]);
    assert_eq!(GpuLightData::default().cookie_misc, [0; 4]);
}

#[test]
fn render_cookie_metadata_aligns_with_packed_light_ids() {
    let point = |light_id, x| RenderPointLightSnapshot {
        node_id: light_id,
        light_id,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
        position: Vec3::new(x, 0.0, 0.0),
        color: Vec3::ONE,
        intensity: 1.0,
        range: 4.0,
        mobility: crate::core::framework::scene::Mobility::Dynamic,
        shadow: None,
    };
    let packed = pack_light_slices_with_advanced_metadata(
        &[],
        &[point(11, 1.0), point(5, 2.0)],
        &[],
        &[],
        &[
            LightCookieData {
                light_id: 5,
                texture: ResourceId::from_stable_label("runtime://cookie/five"),
                projection: CookieProjection::PointOctahedral,
            },
            LightCookieData {
                light_id: 11,
                texture: ResourceId::from_stable_label("runtime://cookie/eleven"),
                projection: CookieProjection::Directional {
                    offset: Vec2::new(0.25, 0.5),
                    scale: Vec2::new(2.0, 3.0),
                    wrap: CookieWrapMode::Repeat,
                },
            },
        ],
        &[11],
    );

    assert_eq!(packed.lights[0].cookie_misc, [1, 1, 1, 0]);
    assert_eq!(packed.lights[0].position_range[0..2], [0.25, 0.5]);
    assert_eq!(packed.lights[0].spot_angles_size[2..4], [2.0, 3.0]);
    assert_eq!(packed.lights[1].cookie_misc, [3, 0, 0, 0]);
    assert_eq!(packed.lights[1].position_range[0], 2.0);
}

#[test]
fn render_volumetric_light_participation_uses_cookie_misc_z_without_cookie() {
    let point = RenderPointLightSnapshot {
        node_id: 7,
        light_id: 7,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
        position: Vec3::ZERO,
        color: Vec3::ONE,
        intensity: 1.0,
        range: 4.0,
        mobility: crate::core::framework::scene::Mobility::Dynamic,
        shadow: None,
    };

    let packed = pack_light_slices_with_advanced_metadata(&[], &[point], &[], &[], &[], &[7]);

    assert_eq!(packed.lights[0].cookie_misc, [0, 0, 1, 0]);
}

#[test]
fn pack_light_slices_preserves_all_point_lights_without_scene_uniform_limit() {
    let points = (0..12)
        .map(|slot| RenderPointLightSnapshot {
            node_id: slot,
            light_id: slot + 100,
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
            position: Vec3::new(slot as f32, 1.0, -2.0),
            color: Vec3::new(1.0, 0.5, 0.25),
            intensity: 2.0,
            range: 3.0,
            mobility: crate::core::framework::scene::Mobility::Dynamic,
            shadow: None,
        })
        .collect::<Vec<_>>();

    let packed = pack_light_slices(&[], &points, &[], &[]);

    assert_eq!(packed.point_count, 12);
    assert_eq!(packed.light_count(), 12);
    assert_eq!(
        packed.lights[11].direction_type[3].to_bits(),
        GpuLightType::Point.as_u32()
    );
    assert_eq!(packed.lights[11].shadow_slot_layer[2], 111);
}

#[test]
fn pack_light_slices_encodes_directional_shadow_and_layer_contract() {
    let packed = pack_light_slices(
        &[RenderDirectionalLightSnapshot {
            node_id: 7,
            light_id: 0x1234_5678_9ABC_DEF0,
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(0b1010),
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::new(0.8, 0.7, 0.6),
            intensity: 4.0,
            mobility: crate::core::framework::scene::Mobility::Dynamic,
            shadow: Some(LightShadowSettings {
                casts_shadow: true,
                depth_bias: 0.25,
                normal_bias: 0.5,
                strength: 0.75,
                resolution_preference: ShadowResolutionTier::T1024,
                pcf_quality: ShadowPcfQuality::High,
            }),
        }],
        &[],
        &[],
        &[],
    );

    let light = packed.lights[0];
    assert_eq!(light.color_intensity, [0.8, 0.7, 0.6, 4.0]);
    assert_eq!(
        light.direction_type[3].to_bits(),
        GpuLightType::Directional.as_u32()
    );
    assert_eq!(light.shadow_slot_layer[0], SHADOW_SLOT_NONE);
    assert_eq!(light.shadow_slot_layer[1], 0b1010);
    assert_eq!(light.shadow_slot_layer[2], 0x9ABC_DEF0);
    assert_eq!(light.shadow_slot_layer[3], GPU_LIGHT_FLAG_CASTS_SHADOW);
    assert_eq!(light.shadow_params, [0.75, 0.25, 0.5, 0.0]);
}

#[test]
fn pack_light_slices_encodes_spot_angles_and_rect_size() {
    let packed = pack_light_slices(
        &[],
        &[],
        &[RenderSpotLightSnapshot {
            node_id: 3,
            light_id: 3,
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
            position: Vec3::new(1.0, 2.0, 3.0),
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::ONE,
            intensity: 1.0,
            range: 8.0,
            inner_angle_radians: 0.25,
            outer_angle_radians: 0.5,
            mobility: crate::core::framework::scene::Mobility::Dynamic,
            shadow: None,
        }],
        &[RenderRectLightSnapshot {
            node_id: 4,
            light_id: 4,
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
            position: Vec3::new(4.0, 5.0, 6.0),
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::ONE,
            intensity: 2.0,
            range: 10.0,
            size: Vec2::new(4.0, 2.0),
            shadow: None,
            renderer_degraded: true,
            degradation_reason: None,
        }],
    );

    assert_eq!(packed.spot_count, 1);
    assert_eq!(packed.rect_count, 1);
    assert_eq!(
        packed.lights[0].direction_type[3].to_bits(),
        GpuLightType::Spot.as_u32()
    );
    assert!((packed.lights[0].spot_angles_size[0] - 0.25_f32.cos()).abs() <= 0.0001);
    assert!((packed.lights[0].spot_angles_size[1] - 0.5_f32.cos()).abs() <= 0.0001);
    assert_eq!(
        packed.lights[1].direction_type[3].to_bits(),
        GpuLightType::Rect.as_u32()
    );
    assert_eq!(packed.lights[1].spot_angles_size, [0.0, 0.0, 2.0, 1.0]);
}
