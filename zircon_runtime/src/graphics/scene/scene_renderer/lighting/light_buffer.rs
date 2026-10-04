use std::collections::HashSet;

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
const VOLUMETRIC_LINEAR_SCAN_LIMIT: usize = 8;

enum VolumetricLightIdIndex<'a> {
    Linear(&'a [u64]),
    Hashed(HashSet<u64>),
}

impl<'a> VolumetricLightIdIndex<'a> {
    fn new(light_ids: &'a [u64]) -> Self {
        if light_ids.len() <= VOLUMETRIC_LINEAR_SCAN_LIMIT {
            Self::Linear(light_ids)
        } else {
            Self::Hashed(light_ids.iter().copied().collect())
        }
    }

    fn contains(&self, light_id: u64) -> bool {
        match self {
            Self::Linear(light_ids) => light_ids.contains(&light_id),
            Self::Hashed(light_ids) => light_ids.contains(&light_id),
        }
    }
}

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

    pack_light_slices_with_advanced_metadata(
        &lighting.directional_lights,
        &lighting.point_lights,
        &lighting.spot_lights,
        &lighting.rect_lights,
        &[],
        &lighting.advanced_lighting.volumetric_light_ids,
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
    pack_light_slices_with_advanced_metadata(
        &lighting.directional_lights,
        &lighting.point_lights,
        &lighting.spot_lights,
        &lighting.rect_lights,
        cookies,
        &lighting.advanced_lighting.volumetric_light_ids,
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
    pack_light_slices_with_advanced_metadata(
        directional_lights,
        point_lights,
        spot_lights,
        rect_lights,
        cookies,
        &[],
    )
}

pub(crate) fn pack_light_slices_with_advanced_metadata(
    directional_lights: &[RenderDirectionalLightSnapshot],
    point_lights: &[RenderPointLightSnapshot],
    spot_lights: &[RenderSpotLightSnapshot],
    rect_lights: &[RenderRectLightSnapshot],
    cookies: &[LightCookieData],
    volumetric_light_ids: &[u64],
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
    // GPU 行与下面的光 ID 链必须保持同一拼接顺序；cookie 和体积光标记按 ID 回填，不能按 cookie 输入位置对齐。
    let cookie_plan = build_cookie_frame_plan(cookies);
    let volumetric_light_ids = VolumetricLightIdIndex::new(volumetric_light_ids);
    let light_ids = directional_lights
        .iter()
        .map(|light| light.light_id)
        .chain(point_lights.iter().map(|light| light.light_id))
        .chain(spot_lights.iter().map(|light| light.light_id))
        .chain(rect_lights.iter().map(|light| light.light_id));
    for (light, light_id) in packed.lights.iter_mut().zip(light_ids) {
        light.cookie_misc[2] = u32::from(volumetric_light_ids.contains(light_id));
        if let Some(metadata) = cookie_plan.metadata_for_light(light_id) {
            apply_cookie_metadata(light, metadata);
        }
    }
    packed
}

fn apply_cookie_metadata(light: &mut GpuLightData, metadata: CookieGpuMetadata) {
    light.cookie_uv_rect = metadata.uv_rect;
    light.cookie_misc[0] = metadata.misc[0];
    light.cookie_misc[1] = metadata.misc[1];
    light.cookie_misc[3] = 0;
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
#[path = "light_buffer/tests/performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "tests/light_buffer.rs"]
mod tests;
