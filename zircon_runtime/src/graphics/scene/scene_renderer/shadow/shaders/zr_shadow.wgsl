struct ZrShadowSlot {
    view_proj: mat4x4<f32>,
    atlas_scale_bias: vec4<f32>,
    params: vec4<f32>,
};

struct ZrShadowGlobals {
    cascade_splits: vec4<f32>,
    cascade_fade_lengths: vec4<f32>,
    atlas_params: vec4<f32>,
};

@group(1) @binding(8) var zr_shadow_atlas: texture_depth_2d;
@group(1) @binding(9) var zr_shadow_sampler: sampler_comparison;
@group(1) @binding(10) var<storage, read> zr_shadow_slots: array<ZrShadowSlot>;
@group(1) @binding(11) var<uniform> zr_shadow_globals: ZrShadowGlobals;

const ZR_SHADOW_SLOT_NONE: u32 = 0xFFFFFFFFu;
const ZR_SHADOW_SLOT_FLAG_VALID: u32 = 1u;
const ZR_SHADOW_SLOT_PCF_QUALITY_SHIFT: u32 = 8u;
const ZR_SHADOW_SLOT_PCF_QUALITY_MASK: u32 = 0x00000300u;
const ZR_SHADOW_PCF_QUALITY_LOW: u32 = 0u;
const ZR_SHADOW_PCF_QUALITY_MEDIUM: u32 = 1u;
const ZR_SHADOW_PCF_QUALITY_HIGH: u32 = 2u;
const ZR_SHADOW_PCF_MEDIUM_RADIUS_TEXELS: i32 = 1;
const ZR_SHADOW_PCF_HIGH_RADIUS_TEXELS: i32 = 8;
const ZR_SHADOW_EPSILON: f32 = 0.000001;

fn zr_shadow_slot_flags(slot: ZrShadowSlot) -> u32 {
    return bitcast<u32>(slot.params.w);
}

fn zr_shadow_slot_valid(slot_index: u32) -> bool {
    if (slot_index == ZR_SHADOW_SLOT_NONE || slot_index >= arrayLength(&zr_shadow_slots)) {
        return false;
    }
    let flags = zr_shadow_slot_flags(zr_shadow_slots[slot_index]);
    return (flags & ZR_SHADOW_SLOT_FLAG_VALID) != 0u;
}

fn zr_shadow_slot_pcf_quality(slot: ZrShadowSlot) -> u32 {
    return (zr_shadow_slot_flags(slot) & ZR_SHADOW_SLOT_PCF_QUALITY_MASK) >> ZR_SHADOW_SLOT_PCF_QUALITY_SHIFT;
}

fn zr_shadow_slot_project(slot: ZrShadowSlot, world_position: vec3<f32>) -> vec4<f32> {
    let light_clip = slot.view_proj * vec4<f32>(world_position, 1.0);
    if (abs(light_clip.w) <= ZR_SHADOW_EPSILON) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let light_ndc = light_clip.xyz / light_clip.w;
    if (any(light_ndc.xy < vec2<f32>(-1.0, -1.0)) || light_ndc.z < 0.0 || any(light_ndc > vec3<f32>(1.0, 1.0, 1.0))) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let local_uv = light_ndc.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    let atlas_uv = local_uv * slot.atlas_scale_bias.xy + slot.atlas_scale_bias.zw;
    return vec4<f32>(atlas_uv, light_ndc.z, 1.0);
}

fn zr_shadow_compare(slot: ZrShadowSlot, atlas_uv: vec2<f32>, receiver_depth: f32, offset: vec2<i32>) -> f32 {
    let atlas_texel = max(slot.params.z, ZR_SHADOW_EPSILON) * slot.atlas_scale_bias.xy;
    let atlas_min = slot.atlas_scale_bias.zw;
    let atlas_max = slot.atlas_scale_bias.zw + slot.atlas_scale_bias.xy;
    let sample_uv = clamp(atlas_uv + vec2<f32>(offset) * atlas_texel, atlas_min, atlas_max);
    return textureSampleCompareLevel(zr_shadow_atlas, zr_shadow_sampler, sample_uv, receiver_depth);
}

fn zr_sample_shadow_slot_low(slot: ZrShadowSlot, atlas_uv: vec2<f32>, receiver_depth: f32) -> f32 {
    return zr_shadow_compare(slot, atlas_uv, receiver_depth, vec2<i32>(0, 0));
}

fn zr_sample_shadow_slot_medium(slot: ZrShadowSlot, atlas_uv: vec2<f32>, receiver_depth: f32) -> f32 {
    let offsets = array<vec2<i32>, 5>(
        vec2<i32>(0, 0),
        vec2<i32>(-ZR_SHADOW_PCF_MEDIUM_RADIUS_TEXELS, 0),
        vec2<i32>(ZR_SHADOW_PCF_MEDIUM_RADIUS_TEXELS, 0),
        vec2<i32>(0, -ZR_SHADOW_PCF_MEDIUM_RADIUS_TEXELS),
        vec2<i32>(0, ZR_SHADOW_PCF_MEDIUM_RADIUS_TEXELS),
    );
    var lit = 0.0;
    for (var i = 0u; i < 5u; i = i + 1u) {
        lit = lit + zr_shadow_compare(slot, atlas_uv, receiver_depth, offsets[i]);
    }
    return lit / 5.0;
}

fn zr_sample_shadow_slot_high(slot: ZrShadowSlot, atlas_uv: vec2<f32>, receiver_depth: f32) -> f32 {
    let offsets = array<vec2<i32>, 9>(
        vec2<i32>(-ZR_SHADOW_PCF_HIGH_RADIUS_TEXELS, -ZR_SHADOW_PCF_HIGH_RADIUS_TEXELS),
        vec2<i32>(0, -ZR_SHADOW_PCF_HIGH_RADIUS_TEXELS),
        vec2<i32>(ZR_SHADOW_PCF_HIGH_RADIUS_TEXELS, -ZR_SHADOW_PCF_HIGH_RADIUS_TEXELS),
        vec2<i32>(-ZR_SHADOW_PCF_HIGH_RADIUS_TEXELS, 0),
        vec2<i32>(0, 0),
        vec2<i32>(ZR_SHADOW_PCF_HIGH_RADIUS_TEXELS, 0),
        vec2<i32>(-ZR_SHADOW_PCF_HIGH_RADIUS_TEXELS, ZR_SHADOW_PCF_HIGH_RADIUS_TEXELS),
        vec2<i32>(0, ZR_SHADOW_PCF_HIGH_RADIUS_TEXELS),
        vec2<i32>(ZR_SHADOW_PCF_HIGH_RADIUS_TEXELS, ZR_SHADOW_PCF_HIGH_RADIUS_TEXELS),
    );
    var lit = 0.0;
    for (var i = 0u; i < 9u; i = i + 1u) {
        lit = lit + zr_shadow_compare(slot, atlas_uv, receiver_depth, offsets[i]);
    }
    return lit / 9.0;
}

fn zr_sample_shadow_slot(slot_index: u32, world_position: vec3<f32>) -> f32 {
    if (!zr_shadow_slot_valid(slot_index)) {
        return 1.0;
    }

    let slot = zr_shadow_slots[slot_index];
    let shadow_coord = zr_shadow_slot_project(slot, world_position);
    if (shadow_coord.w <= 0.0) {
        return 1.0;
    }

    let receiver_depth = clamp(shadow_coord.z - slot.params.x, 0.0, 1.0);
    let quality = zr_shadow_slot_pcf_quality(slot);
    if (quality == ZR_SHADOW_PCF_QUALITY_LOW) {
        return zr_sample_shadow_slot_low(slot, shadow_coord.xy, receiver_depth);
    }
    if (quality == ZR_SHADOW_PCF_QUALITY_MEDIUM) {
        return zr_sample_shadow_slot_medium(slot, shadow_coord.xy, receiver_depth);
    }
    return zr_sample_shadow_slot_high(slot, shadow_coord.xy, receiver_depth);
}

fn zr_shadow_cascade_index(view_z: f32, cascade_count: u32) -> u32 {
    let count = min(cascade_count, 4u);
    if (count <= 1u) {
        return 0u;
    }
    for (var index = 0u; index < count; index = index + 1u) {
        if (view_z <= zr_shadow_globals.cascade_splits[index]) {
            return index;
        }
    }
    return count - 1u;
}

fn zr_sample_directional_shadow(light: ZrGpuLightData, world_position: vec3<f32>, view_z: f32) -> f32 {
    let slot_count = u32(max(light.shadow_params.w, 0.0));
    if (slot_count == 0u) {
        return 1.0;
    }
    let cascade = zr_shadow_cascade_index(view_z, slot_count);
    let first_slot = light.shadow_slot_layer.x;
    let primary = zr_sample_shadow_slot(first_slot + cascade, world_position);
    let fade_length = zr_shadow_globals.cascade_fade_lengths[cascade];
    if (fade_length <= ZR_SHADOW_EPSILON || cascade + 1u >= slot_count || cascade + 1u >= 4u) {
        return primary;
    }

    let split = zr_shadow_globals.cascade_splits[cascade];
    let fade = clamp((view_z - (split - fade_length)) / max(fade_length, ZR_SHADOW_EPSILON), 0.0, 1.0);
    if (fade <= 0.0) {
        return primary;
    }
    let next_visibility = zr_sample_shadow_slot(first_slot + cascade + 1u, world_position);
    return mix(primary, next_visibility, fade);
}

fn zr_point_shadow_face_index(light_position: vec3<f32>, world_position: vec3<f32>) -> u32 {
    let direction = world_position - light_position;
    let abs_direction = abs(direction);
    if (abs_direction.x >= abs_direction.y && abs_direction.x >= abs_direction.z) {
        return select(1u, 0u, direction.x >= 0.0);
    }
    if (abs_direction.y >= abs_direction.z) {
        return select(3u, 2u, direction.y >= 0.0);
    }
    return select(5u, 4u, direction.z >= 0.0);
}

fn zr_gpu_light_shadow_visibility(light: ZrGpuLightData, light_type: u32, world_position: vec3<f32>, view_z: f32) -> f32 {
    if (!zr_gpu_light_casts_shadow(light) || light.shadow_params.w <= 0.5) {
        return 1.0;
    }
    if (light.shadow_slot_layer.x == ZR_SHADOW_SLOT_NONE) {
        return 1.0;
    }
    if (light_type == ZR_GPU_LIGHT_TYPE_DIRECTIONAL) {
        return zr_sample_directional_shadow(light, world_position, view_z);
    }
    if (light_type == ZR_GPU_LIGHT_TYPE_POINT && light.shadow_params.w >= 6.0) {
        let face = zr_point_shadow_face_index(light.position_range.xyz, world_position);
        return zr_sample_shadow_slot(light.shadow_slot_layer.x + face, world_position);
    }
    return zr_sample_shadow_slot(light.shadow_slot_layer.x, world_position);
}
