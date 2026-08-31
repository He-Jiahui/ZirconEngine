struct ZrGpuPrimitiveData {
    local_bounds_center: vec3<f32>,
    local_bounds_radius: f32,
    tint: vec4<f32>,
    shadow_params: vec4<f32>,
    motion_params: vec4<f32>,
    flags: u32,
    first_instance_index: u32,
    instance_count: u32,
    payload_slot: u32,
    material_payload_slot: u32,
    hit_proxy_token: u32,
    material_payload_padding_1: u32,
    material_payload_padding_2: u32,
};

struct ZrGpuInstanceData {
    world_from_local: mat4x4<f32>,
    prev_world_from_local: mat4x4<f32>,
    primitive_index: u32,
    flags: u32,
    payload_slot: u32,
    morph_payload_slot: u32,
    lightmap_uv_rect: vec4<f32>,
    lightmap_params: vec4<u32>,
    skinning_palette_params: vec4<u32>,
};

struct ZrGpuLightData {
    position_range: vec4<f32>,
    color_intensity: vec4<f32>,
    direction_type: vec4<f32>,
    spot_angles_size: vec4<f32>,
    shadow_slot_layer: vec4<u32>,
    shadow_params: vec4<f32>,
    cookie_uv_rect: vec4<f32>,
    cookie_misc: vec4<u32>,
};

struct ZrGpuSceneVisibleInstanceRemapParams {
    values: vec4<u32>,
};

@group(3) @binding(0) var<storage, read> zr_primitive_data: array<ZrGpuPrimitiveData>;
@group(3) @binding(1) var<storage, read> zr_instance_data: array<ZrGpuInstanceData>;
@group(3) @binding(2) var<storage, read> zr_light_data: array<ZrGpuLightData>;
@group(3) @binding(3) var<storage, read> zr_skinned_joint_palette: array<mat4x4<f32>>;
@group(3) @binding(4) var<storage, read> zr_previous_skinned_joint_palette: array<mat4x4<f32>>;
@group(3) @binding(5) var<storage, read> zr_visible_instance_remap: array<u32>;
@group(3) @binding(6) var<uniform> zr_visible_instance_remap_params: ZrGpuSceneVisibleInstanceRemapParams;
@group(3) @binding(7) var<storage, read> zr_morph_deltas: array<vec4<f32>>;
@group(3) @binding(8) var<storage, read> zr_morph_weights: array<f32>;
@group(3) @binding(9) var<storage, read> zr_virtual_geometry_pages: array<vec4<u32>>;
@group(3) @binding(10) var<storage, read> zr_virtual_geometry_clusters: array<vec4<f32>>;
@group(3) @binding(11) var<storage, read> zr_morph_payloads: array<vec4<u32>>;

const ZR_GPU_LIGHT_TYPE_DIRECTIONAL: u32 = 0u;
const ZR_GPU_LIGHT_TYPE_POINT: u32 = 1u;
const ZR_GPU_LIGHT_TYPE_SPOT: u32 = 2u;
const ZR_GPU_LIGHT_TYPE_RECT: u32 = 3u;
const ZR_GPU_LIGHT_FLAG_CASTS_SHADOW: u32 = 1u;
const ZR_GPU_SCENE_INVALID_PAYLOAD_SLOT: u32 = 0xffffffffu;
const ZR_GPU_INSTANCE_FLAG_GENERAL_NORMAL_TRANSFORM: u32 = 1u;
const ZR_GPU_INSTANCE_FLAG_NEGATIVE_DETERMINANT: u32 = 2u;
const ZR_GPU_INSTANCE_FLAG_DEGENERATE_NORMAL_TRANSFORM: u32 = 4u;
const ZR_GPU_INSTANCE_FLAG_NON_ORTHOGONAL_TRANSFORM: u32 = 8u;
const ZR_GPU_PRIMITIVE_FLAG_VISIBLE: u32 = 1u;
const ZR_GPU_PRIMITIVE_FLAG_CAST_SHADOWS: u32 = 2u;
const ZR_GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM: u32 = 4u;
const ZR_GPU_PRIMITIVE_FLAG_FORCE_HZB_VISIBLE: u32 = 8u;

fn zr_gpu_scene_light_count() -> u32 {
    return min(zr_visible_instance_remap_params.values.y, arrayLength(&zr_light_data));
}

fn zr_gpu_scene_virtual_geometry_page_count() -> u32 {
    return min(
        zr_visible_instance_remap_params.values.z,
        arrayLength(&zr_virtual_geometry_pages),
    );
}

fn zr_gpu_scene_virtual_geometry_cluster_word_count() -> u32 {
    return min(
        zr_visible_instance_remap_params.values.w,
        arrayLength(&zr_virtual_geometry_clusters),
    );
}

fn zr_gpu_light(light_index: u32) -> ZrGpuLightData {
    return zr_light_data[light_index];
}

fn zr_gpu_light_type(light: ZrGpuLightData) -> u32 {
    return bitcast<u32>(light.direction_type.w);
}

fn zr_gpu_light_casts_shadow(light: ZrGpuLightData) -> bool {
    return (light.shadow_slot_layer.w & ZR_GPU_LIGHT_FLAG_CASTS_SHADOW) != 0u;
}

fn zr_skinned_joint_count(instance_index: u32) -> u32 {
    return zr_gpu_scene_instance(instance_index).skinning_palette_params.y;
}

fn zr_skinned_joint_matrix(instance_index: u32, joint_index: u32) -> mat4x4<f32> {
    let base = zr_gpu_scene_instance(instance_index).skinning_palette_params.x;
    return zr_skinned_joint_palette[base + joint_index];
}

fn zr_previous_skinned_joint_count(instance_index: u32) -> u32 {
    return zr_gpu_scene_instance(instance_index).skinning_palette_params.w;
}

fn zr_previous_skinned_joint_matrix(instance_index: u32, joint_index: u32) -> mat4x4<f32> {
    let base = zr_gpu_scene_instance(instance_index).skinning_palette_params.z;
    return zr_previous_skinned_joint_palette[base + joint_index];
}

fn zr_gpu_scene_resolve_instance_index(instance_index: u32) -> u32 {
    if (zr_visible_instance_remap_params.values.x != 0u) {
        return zr_visible_instance_remap[instance_index];
    }
    return instance_index;
}

fn zr_gpu_scene_instance(instance_index: u32) -> ZrGpuInstanceData {
    return zr_instance_data[zr_gpu_scene_resolve_instance_index(instance_index)];
}

fn zr_gpu_scene_primitive(instance: ZrGpuInstanceData) -> ZrGpuPrimitiveData {
    return zr_primitive_data[instance.primitive_index];
}

fn zr_gpu_scene_primitive_for_instance(instance_index: u32) -> ZrGpuPrimitiveData {
    return zr_gpu_scene_primitive(zr_gpu_scene_instance(instance_index));
}

fn zr_hit_proxy_token(instance_index: u32) -> u32 {
    return zr_gpu_scene_primitive_for_instance(instance_index).hit_proxy_token;
}

fn zr_gpu_scene_valid_payload_slot(payload_slot: u32, element_count: u32) -> bool {
    return payload_slot != ZR_GPU_SCENE_INVALID_PAYLOAD_SLOT && payload_slot < element_count;
}

fn zr_gpu_scene_morph_delta(delta_index: u32) -> vec3<f32> {
    return zr_gpu_scene_morph_delta_row(delta_index).xyz;
}

fn zr_gpu_scene_morph_delta_row(delta_index: u32) -> vec4<f32> {
    if (delta_index < arrayLength(&zr_morph_deltas)) {
        return zr_morph_deltas[delta_index];
    }
    return vec4<f32>(0.0);
}

fn zr_gpu_scene_morph_weight(weight_index: u32) -> f32 {
    if (weight_index < arrayLength(&zr_morph_weights)) {
        return zr_morph_weights[weight_index];
    }
    return 0.0;
}

fn zr_gpu_scene_morph_payload(payload_slot: u32) -> vec4<u32> {
    if (zr_gpu_scene_valid_payload_slot(payload_slot, arrayLength(&zr_morph_payloads))) {
        return zr_morph_payloads[payload_slot];
    }
    return vec4<u32>(0u);
}

fn zr_world_from_local(instance_index: u32) -> mat4x4<f32> {
    return zr_gpu_scene_instance(instance_index).world_from_local;
}

fn zr_previous_world_from_local(instance_index: u32) -> mat4x4<f32> {
    return zr_gpu_scene_instance(instance_index).prev_world_from_local;
}

fn zr_gpu_scene_normal_to_world_direction(
    world_from_local: mat4x4<f32>,
    instance_flags: u32,
    normal_os: vec3<f32>,
) -> vec3<f32> {
    if ((instance_flags & ZR_GPU_INSTANCE_FLAG_DEGENERATE_NORMAL_TRANSFORM) != 0u) {
        return vec3<f32>(0.0);
    }

    if ((instance_flags & ZR_GPU_INSTANCE_FLAG_GENERAL_NORMAL_TRANSFORM) == 0u) {
        return (world_from_local * vec4<f32>(normal_os, 0.0)).xyz;
    }

    let x = world_from_local[0].xyz;
    let y = world_from_local[1].xyz;
    let z = world_from_local[2].xyz;
    let adjugate_normal = cross(y, z) * normal_os.x
        + cross(z, x) * normal_os.y
        + cross(x, y) * normal_os.z;
    let determinant_sign = select(
        1.0,
        -1.0,
        (instance_flags & ZR_GPU_INSTANCE_FLAG_NEGATIVE_DETERMINANT) != 0u,
    );
    return adjugate_normal * determinant_sign;
}

fn zr_gpu_scene_tangent_to_world_direction(
    world_from_local: mat4x4<f32>,
    instance_flags: u32,
    tangent_os: vec3<f32>,
) -> vec3<f32> {
    if ((instance_flags & ZR_GPU_INSTANCE_FLAG_DEGENERATE_NORMAL_TRANSFORM) != 0u) {
        return vec3<f32>(0.0);
    }
    return (world_from_local * vec4<f32>(tangent_os, 0.0)).xyz;
}

fn zr_gpu_scene_tangent_handedness_scale(instance_flags: u32) -> f32 {
    if ((instance_flags & ZR_GPU_INSTANCE_FLAG_DEGENERATE_NORMAL_TRANSFORM) != 0u) {
        return 0.0;
    }
    return select(
        1.0,
        -1.0,
        (instance_flags & ZR_GPU_INSTANCE_FLAG_NEGATIVE_DETERMINANT) != 0u,
    );
}

fn zr_gpu_scene_tint(instance_index: u32) -> vec4<f32> {
    return zr_gpu_scene_primitive_for_instance(instance_index).tint;
}

fn zr_gpu_scene_shadow_params(instance_index: u32) -> vec4<f32> {
    return zr_gpu_scene_primitive_for_instance(instance_index).shadow_params;
}

fn zr_gpu_scene_motion_params(instance_index: u32) -> vec4<f32> {
    return zr_gpu_scene_primitive_for_instance(instance_index).motion_params;
}

fn zr_gpu_scene_material_payload_slot(instance_index: u32) -> u32 {
    return zr_gpu_scene_primitive_for_instance(instance_index).material_payload_slot;
}

fn zr_gpu_scene_has_lightmap(instance_index: u32) -> bool {
    return zr_gpu_scene_instance(instance_index).lightmap_params.y != 0u;
}

fn zr_gpu_scene_lightmap_uv_rect(instance_index: u32) -> vec4<f32> {
    return zr_gpu_scene_instance(instance_index).lightmap_uv_rect;
}

fn zr_gpu_scene_lightmap_params(instance_index: u32) -> vec4<u32> {
    return zr_gpu_scene_instance(instance_index).lightmap_params;
}
