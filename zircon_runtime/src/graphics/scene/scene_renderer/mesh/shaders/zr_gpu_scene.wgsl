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
    _pad0: u32,
};

struct ZrSkinnedJointPaletteUniform {
    joint_matrices: array<mat4x4<f32>, 256>,
    params: vec4<u32>,
};

@group(3) @binding(0) var<storage, read> zr_primitive_data: array<ZrGpuPrimitiveData>;
@group(3) @binding(1) var<storage, read> zr_instance_data: array<ZrGpuInstanceData>;
@group(3) @binding(2) var<storage, read> zr_light_data: array<vec4<f32>>;
@group(3) @binding(3) var<uniform> zr_skinned_joint_palette: ZrSkinnedJointPaletteUniform;
@group(3) @binding(4) var<uniform> zr_previous_skinned_joint_palette: ZrSkinnedJointPaletteUniform;

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

fn zr_gpu_scene_instance(instance_index: u32) -> ZrGpuInstanceData {
    return zr_instance_data[instance_index];
}

fn zr_gpu_scene_primitive(instance: ZrGpuInstanceData) -> ZrGpuPrimitiveData {
    return zr_primitive_data[instance.primitive_index];
}

fn zr_gpu_scene_primitive_for_instance(instance_index: u32) -> ZrGpuPrimitiveData {
    return zr_gpu_scene_primitive(zr_gpu_scene_instance(instance_index));
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
