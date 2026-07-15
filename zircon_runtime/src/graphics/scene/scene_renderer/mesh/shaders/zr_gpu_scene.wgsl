struct ZrGpuPrimitiveData {
    bounds_center: vec3<f32>,
    bounds_radius: f32,
    tint: vec4<f32>,
    shadow_params: vec4<f32>,
    motion_params: vec4<f32>,
    flags: u32,
    first_instance_index: u32,
    instance_count: u32,
    payload_slot: u32,
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

struct ZrSkinnedJointPaletteStorage {
    joint_matrices: array<mat4x4<f32>, 256>,
    params: vec4<u32>,
};

struct ZrGpuSceneVisibleInstanceRemapParams {
    values: vec4<u32>,
};

@group(3) @binding(0) var<storage, read> zr_primitive_data: array<ZrGpuPrimitiveData>;
@group(3) @binding(1) var<storage, read> zr_instance_data: array<ZrGpuInstanceData>;
@group(3) @binding(2) var<storage, read> zr_light_data: array<ZrGpuLightData>;
@group(3) @binding(3) var<storage, read> zr_skinned_joint_palette: ZrSkinnedJointPaletteStorage;
@group(3) @binding(4) var<storage, read> zr_previous_skinned_joint_palette: ZrSkinnedJointPaletteStorage;
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

fn zr_gpu_scene_light_count() -> u32 {
    return min(zr_visible_instance_remap_params.values.y, arrayLength(&zr_light_data));
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

fn zr_skinned_joint_count() -> u32 {
    return zr_skinned_joint_palette.params.x;
}

fn zr_skinned_joint_matrix(joint_index: u32) -> mat4x4<f32> {
    return zr_skinned_joint_palette.joint_matrices[joint_index];
}

fn zr_previous_skinned_joint_count() -> u32 {
    return zr_previous_skinned_joint_palette.params.x;
}

fn zr_previous_skinned_joint_matrix(joint_index: u32) -> mat4x4<f32> {
    return zr_previous_skinned_joint_palette.joint_matrices[joint_index];
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

fn zr_gpu_scene_tint(instance_index: u32) -> vec4<f32> {
    return zr_gpu_scene_primitive_for_instance(instance_index).tint;
}

fn zr_gpu_scene_shadow_params(instance_index: u32) -> vec4<f32> {
    return zr_gpu_scene_primitive_for_instance(instance_index).shadow_params;
}

fn zr_gpu_scene_motion_params(instance_index: u32) -> vec4<f32> {
    return zr_gpu_scene_primitive_for_instance(instance_index).motion_params;
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
