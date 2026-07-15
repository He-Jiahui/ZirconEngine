use crate::core::framework::render::{
    GpuLightData, GpuLightType, LightCookieData, LightShadowSettings, LightingExtract,
    RenderDirectionalLightSnapshot, RenderLayerSet, RenderPointLightSnapshot,
    RenderRectLightSnapshot, RenderSpotLightSnapshot, SHADOW_SLOT_NONE,
};
use crate::core::math::{Vec2, Vec3};
use crate::graphics::scene::scene_renderer::advanced_lighting::light_cookie::{
    build_cookie_frame_plan, CookieGpuMetadata,
};

pub(crate) const GPU_LIGHT_FLAG_CASTS_SHADOW: u32 = 1 << 0;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PackedGpuLightBuffer {
    pub(crate) lights: Vec<GpuLightData>,
    pub(crate) directional_count: u32,
    pub(crate) point_count: u32,
    pub(crate) spot_count: u32,
    pub(crate) rect_count: u32,
}

impl PackedGpuLightBuffer {
    pub(crate) fn light_count(&self) -> u32 {
        self.lights.len() as u32
    }
}

pub(crate) fn pack_lighting_extract(
    lighting: &LightingExtract,
    lighting_enabled: bool,
) -> PackedGpuLightBuffer {
    if !lighting_enabled {
        return PackedGpuLightBuffer::default();
    }

    pack_light_slices_with_cookies(
        &lighting.directional_lights,
        &lighting.point_lights,
        &lighting.spot_lights,
        &lighting.rect_lights,
        &[],
    )
}

pub(crate) fn pack_lighting_extract_with_cookies(
    lighting: &LightingExtract,
    cookies: &[LightCookieData],
    lighting_enabled: bool,
) -> PackedGpuLightBuffer {
    if !lighting_enabled {
        return PackedGpuLightBuffer::default();
    }
    pack_light_slices_with_cookies(
        &lighting.directional_lights,
        &lighting.point_lights,
        &lighting.spot_lights,
        &lighting.rect_lights,
        cookies,
    )
}

pub(crate) fn pack_light_slices(
    directional_lights: &[RenderDirectionalLightSnapshot],
    point_lights: &[RenderPointLightSnapshot],
    spot_lights: &[RenderSpotLightSnapshot],
    rect_lights: &[RenderRectLightSnapshot],
) -> PackedGpuLightBuffer {
    pack_light_slices_with_cookies(
        directional_lights,
        point_lights,
        spot_lights,
        rect_lights,
        &[],
    )
}

pub(crate) fn pack_light_slices_with_cookies(
    directional_lights: &[RenderDirectionalLightSnapshot],
    point_lights: &[RenderPointLightSnapshot],
    spot_lights: &[RenderSpotLightSnapshot],
    rect_lights: &[RenderRectLightSnapshot],
    cookies: &[LightCookieData],
) -> PackedGpuLightBuffer {
    let mut packed = PackedGpuLightBuffer {
        lights: Vec::with_capacity(
            directional_lights.len() + point_lights.len() + spot_lights.len() + rect_lights.len(),
        ),
        directional_count: directional_lights.len() as u32,
        point_count: point_lights.len() as u32,
        spot_count: spot_lights.len() as u32,
        rect_count: rect_lights.len() as u32,
    };

    packed
        .lights
        .extend(directional_lights.iter().map(pack_directional_light));
    packed
        .lights
        .extend(point_lights.iter().map(pack_point_light));
    packed
        .lights
        .extend(spot_lights.iter().map(pack_spot_light));
    packed
        .lights
        .extend(rect_lights.iter().map(pack_rect_light));
    let cookie_plan = build_cookie_frame_plan(cookies);
    let light_ids = directional_lights
        .iter()
        .map(|light| light.light_id)
        .chain(point_lights.iter().map(|light| light.light_id))
        .chain(spot_lights.iter().map(|light| light.light_id))
        .chain(rect_lights.iter().map(|light| light.light_id));
    for (light, light_id) in packed.lights.iter_mut().zip(light_ids) {
        if let Some(metadata) = cookie_plan.metadata_for_light(light_id) {
            apply_cookie_metadata(light, metadata);
        }
    }
    packed
}

fn apply_cookie_metadata(light: &mut GpuLightData, metadata: CookieGpuMetadata) {
    light.cookie_uv_rect = metadata.uv_rect;
    light.cookie_misc = metadata.misc;
    if metadata.misc[0]
        == crate::graphics::scene::scene_renderer::advanced_lighting::light_cookie::COOKIE_PROJECTION_DIRECTIONAL
    {
        light.position_range[0] = metadata.directional_offset_scale[0];
        light.position_range[1] = metadata.directional_offset_scale[1];
        light.spot_angles_size[2] = metadata.directional_offset_scale[2];
        light.spot_angles_size[3] = metadata.directional_offset_scale[3];
    }
}

fn pack_directional_light(light: &RenderDirectionalLightSnapshot) -> GpuLightData {
    GpuLightData {
        position_range: [0.0, 0.0, 0.0, 0.0],
        color_intensity: color_intensity(light.color, light.intensity),
        direction_type: direction_type(light.direction, GpuLightType::Directional),
        spot_angles_size: [0.0; 4],
        shadow_slot_layer: shadow_slot_layer(light.light_id, &light.layer_mask, light.shadow),
        shadow_params: shadow_params(light.shadow),
        cookie_uv_rect: [0.0; 4],
        cookie_misc: [0; 4],
    }
}

fn pack_point_light(light: &RenderPointLightSnapshot) -> GpuLightData {
    GpuLightData {
        position_range: vec3_w(light.position, light.range.max(0.0)),
        color_intensity: color_intensity(light.color, light.intensity),
        direction_type: direction_type(Vec3::ZERO, GpuLightType::Point),
        spot_angles_size: [0.0; 4],
        shadow_slot_layer: shadow_slot_layer(light.light_id, &light.layer_mask, light.shadow),
        shadow_params: shadow_params(light.shadow),
        cookie_uv_rect: [0.0; 4],
        cookie_misc: [0; 4],
    }
}

fn pack_spot_light(light: &RenderSpotLightSnapshot) -> GpuLightData {
    GpuLightData {
        position_range: vec3_w(light.position, light.range.max(0.0)),
        color_intensity: color_intensity(light.color, light.intensity),
        direction_type: direction_type(light.direction, GpuLightType::Spot),
        spot_angles_size: [
            light.inner_angle_radians.cos(),
            light.outer_angle_radians.cos(),
            0.0,
            0.0,
        ],
        shadow_slot_layer: shadow_slot_layer(light.light_id, &light.layer_mask, light.shadow),
        shadow_params: shadow_params(light.shadow),
        cookie_uv_rect: [0.0; 4],
        cookie_misc: [0; 4],
    }
}

fn pack_rect_light(light: &RenderRectLightSnapshot) -> GpuLightData {
    GpuLightData {
        position_range: vec3_w(light.position, light.range.max(0.0)),
        color_intensity: color_intensity(light.color, light.intensity),
        direction_type: direction_type(light.direction, GpuLightType::Rect),
        spot_angles_size: rect_half_size(light.size),
        shadow_slot_layer: shadow_slot_layer(light.light_id, &light.layer_mask, light.shadow),
        shadow_params: shadow_params(light.shadow),
        cookie_uv_rect: [0.0; 4],
        cookie_misc: [0; 4],
    }
}

fn color_intensity(color: Vec3, intensity: f32) -> [f32; 4] {
    [color.x, color.y, color.z, intensity.max(0.0)]
}

fn direction_type(direction: Vec3, light_type: GpuLightType) -> [f32; 4] {
    [
        direction.x,
        direction.y,
        direction.z,
        light_type.as_f32_bits(),
    ]
}

fn vec3_w(value: Vec3, w: f32) -> [f32; 4] {
    [value.x, value.y, value.z, w]
}

fn rect_half_size(size: Vec2) -> [f32; 4] {
    [0.0, 0.0, size.x.max(0.0) * 0.5, size.y.max(0.0) * 0.5]
}

fn shadow_slot_layer(
    light_id: u64,
    layer_mask: &RenderLayerSet,
    shadow: Option<LightShadowSettings>,
) -> [u32; 4] {
    let flags = shadow
        .filter(|settings| settings.casts_shadow)
        .map(|_| GPU_LIGHT_FLAG_CASTS_SHADOW)
        .unwrap_or(0);
    [
        SHADOW_SLOT_NONE,
        layer_mask.to_scene_schema_v1_mask_lossy(),
        light_id as u32,
        flags,
    ]
}

fn shadow_params(shadow: Option<LightShadowSettings>) -> [f32; 4] {
    let Some(shadow) = shadow.filter(|settings| settings.casts_shadow) else {
        return [0.0; 4];
    };

    [
        shadow.strength.clamp(0.0, 1.0),
        shadow.depth_bias,
        shadow.normal_bias,
        0.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        CookieProjection, CookieWrapMode, LightShadowSettings, ShadowPcfQuality,
        ShadowResolutionTier, DEFAULT_RENDER_LAYER_MASK,
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
        let packed = pack_light_slices_with_cookies(
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
        );

        assert_eq!(packed.lights[0].cookie_misc[2], 1);
        assert_eq!(packed.lights[0].position_range[0..2], [0.25, 0.5]);
        assert_eq!(packed.lights[0].spot_angles_size[2..4], [2.0, 3.0]);
        assert_eq!(packed.lights[1].cookie_misc[2], 0);
        assert_eq!(packed.lights[1].position_range[0], 2.0);
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
}
