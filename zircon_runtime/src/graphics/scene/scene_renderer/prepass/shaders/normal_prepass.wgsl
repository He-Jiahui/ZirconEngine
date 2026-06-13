struct SceneUniform {
    view_proj: mat4x4<f32>,
};

struct MaterialPropertyUniform {
    data0: vec4<f32>,
    data1: vec4<f32>,
    data2: vec4<f32>,
    data3: vec4<f32>,
    data4: vec4<f32>,
    data5: vec4<f32>,
    data6: vec4<f32>,
    data7: vec4<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(2) @binding(2) var normal_tex: texture_2d<f32>;
@group(2) @binding(3) var normal_sampler: sampler;
@group(2) @binding(10) var<uniform> material_properties: MaterialPropertyUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) joint_indices: vec4<u32>,
    @location(4) joint_weights: vec4<f32>,
    @location(5) tangent: vec4<f32>,
    @location(7) uv1: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_tangent: vec3<f32>,
    @location(3) tangent_handedness: f32,
    @location(4) uv1: vec2<f32>,
    @location(5) motion_params: vec4<f32>,
};

const EPSILON: f32 = 0.000001;

fn skin_weight(joint_index: u32, weight: f32) -> f32 {
    if (weight <= EPSILON || joint_index >= zr_skinned_joint_count()) {
        return 0.0;
    }
    return weight;
}

fn skin_weight_sum(joint_indices: vec4<u32>, joint_weights: vec4<f32>) -> f32 {
    return skin_weight(joint_indices.x, joint_weights.x)
        + skin_weight(joint_indices.y, joint_weights.y)
        + skin_weight(joint_indices.z, joint_weights.z)
        + skin_weight(joint_indices.w, joint_weights.w);
}

fn skin_vertex_position(position: vec3<f32>, joint_indices: vec4<u32>, joint_weights: vec4<f32>, motion_params: vec4<f32>) -> vec3<f32> {
    if (motion_params.y <= 0.5 || zr_skinned_joint_count() == 0u) {
        return position;
    }

    let weight_sum = skin_weight_sum(joint_indices, joint_weights);
    if (weight_sum <= EPSILON) {
        return position;
    }

    var skinned = vec3<f32>(0.0, 0.0, 0.0);
    let weight_x = skin_weight(joint_indices.x, joint_weights.x);
    let weight_y = skin_weight(joint_indices.y, joint_weights.y);
    let weight_z = skin_weight(joint_indices.z, joint_weights.z);
    let weight_w = skin_weight(joint_indices.w, joint_weights.w);
    if (weight_x > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(joint_indices.x) * vec4<f32>(position, 1.0)).xyz * (weight_x / weight_sum);
    }
    if (weight_y > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(joint_indices.y) * vec4<f32>(position, 1.0)).xyz * (weight_y / weight_sum);
    }
    if (weight_z > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(joint_indices.z) * vec4<f32>(position, 1.0)).xyz * (weight_z / weight_sum);
    }
    if (weight_w > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(joint_indices.w) * vec4<f32>(position, 1.0)).xyz * (weight_w / weight_sum);
    }
    return skinned;
}

fn skin_vertex_normal(normal: vec3<f32>, joint_indices: vec4<u32>, joint_weights: vec4<f32>, motion_params: vec4<f32>) -> vec3<f32> {
    if (motion_params.y <= 0.5 || zr_skinned_joint_count() == 0u) {
        return normal;
    }

    let weight_sum = skin_weight_sum(joint_indices, joint_weights);
    if (weight_sum <= EPSILON) {
        return normal;
    }

    var skinned = vec3<f32>(0.0, 0.0, 0.0);
    let weight_x = skin_weight(joint_indices.x, joint_weights.x);
    let weight_y = skin_weight(joint_indices.y, joint_weights.y);
    let weight_z = skin_weight(joint_indices.z, joint_weights.z);
    let weight_w = skin_weight(joint_indices.w, joint_weights.w);
    if (weight_x > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(joint_indices.x) * vec4<f32>(normal, 0.0)).xyz * (weight_x / weight_sum);
    }
    if (weight_y > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(joint_indices.y) * vec4<f32>(normal, 0.0)).xyz * (weight_y / weight_sum);
    }
    if (weight_z > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(joint_indices.z) * vec4<f32>(normal, 0.0)).xyz * (weight_z / weight_sum);
    }
    if (weight_w > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(joint_indices.w) * vec4<f32>(normal, 0.0)).xyz * (weight_w / weight_sum);
    }

    let normal_length = length(skinned);
    if (normal_length <= EPSILON) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    return skinned / normal_length;
}

fn skin_vertex_tangent(tangent: vec3<f32>, joint_indices: vec4<u32>, joint_weights: vec4<f32>, motion_params: vec4<f32>) -> vec3<f32> {
    if (motion_params.y <= 0.5 || zr_skinned_joint_count() == 0u) {
        return tangent;
    }

    let weight_sum = skin_weight_sum(joint_indices, joint_weights);
    if (weight_sum <= EPSILON) {
        return tangent;
    }

    var skinned = vec3<f32>(0.0, 0.0, 0.0);
    let weight_x = skin_weight(joint_indices.x, joint_weights.x);
    let weight_y = skin_weight(joint_indices.y, joint_weights.y);
    let weight_z = skin_weight(joint_indices.z, joint_weights.z);
    let weight_w = skin_weight(joint_indices.w, joint_weights.w);
    if (weight_x > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(joint_indices.x) * vec4<f32>(tangent, 0.0)).xyz * (weight_x / weight_sum);
    }
    if (weight_y > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(joint_indices.y) * vec4<f32>(tangent, 0.0)).xyz * (weight_y / weight_sum);
    }
    if (weight_z > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(joint_indices.z) * vec4<f32>(tangent, 0.0)).xyz * (weight_z / weight_sum);
    }
    if (weight_w > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(joint_indices.w) * vec4<f32>(tangent, 0.0)).xyz * (weight_w / weight_sum);
    }

    let tangent_length = length(skinned);
    if (tangent_length <= EPSILON) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    return skinned / tangent_length;
}

fn normalize_or_zero(value: vec3<f32>) -> vec3<f32> {
    let value_length = length(value);
    if (value_length <= EPSILON) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    return value / value_length;
}

fn transform_material_uv(uv: vec2<f32>, transform: vec4<f32>) -> vec2<f32> {
    return uv * transform.xy + transform.zw;
}

fn material_uv_channel(channel: f32) -> u32 {
    return select(0u, 1u, channel >= 0.5);
}

fn select_material_uv(uv0: vec2<f32>, uv1: vec2<f32>, channel: f32) -> vec2<f32> {
    if (material_uv_channel(channel) == 1u) {
        return uv1;
    }
    return uv0;
}

fn transform_material_uv_channel(uv0: vec2<f32>, uv1: vec2<f32>, transform: vec4<f32>, channel: f32) -> vec2<f32> {
    return transform_material_uv(select_material_uv(uv0, uv1, channel), transform);
}

@vertex
fn vs_main(input: VertexInput, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var output: VertexOutput;
    let world_from_local = zr_world_from_local(instance_index);
    let motion_params = zr_gpu_scene_motion_params(instance_index);
    let local_position = skin_vertex_position(input.position, input.joint_indices, input.joint_weights, motion_params);
    let local_normal = skin_vertex_normal(input.normal, input.joint_indices, input.joint_weights, motion_params);
    let local_tangent = skin_vertex_tangent(input.tangent.xyz, input.joint_indices, input.joint_weights, motion_params);
    let world = world_from_local * vec4<f32>(local_position, 1.0);
    output.clip_position = scene.view_proj * world;
    output.world_normal = normalize_or_zero((world_from_local * vec4<f32>(local_normal, 0.0)).xyz);
    output.uv = input.uv;
    output.world_tangent = normalize_or_zero((world_from_local * vec4<f32>(local_tangent, 0.0)).xyz);
    output.tangent_handedness = select(-1.0, 1.0, input.tangent.w >= 0.0);
    output.uv1 = input.uv1;
    output.motion_params = motion_params;
    return output;
}

fn sampled_world_normal(input: VertexOutput) -> vec3<f32> {
    let geometric_normal = normalize_or_zero(input.world_normal);
    if (input.motion_params.w <= 0.5 || length(geometric_normal) <= EPSILON) {
        return geometric_normal;
    }

    let tangent = normalize_or_zero(input.world_tangent - geometric_normal * dot(input.world_tangent, geometric_normal));
    if (length(tangent) <= EPSILON) {
        return geometric_normal;
    }
    let bitangent = normalize_or_zero(cross(geometric_normal, tangent) * input.tangent_handedness);
    if (length(bitangent) <= EPSILON) {
        return geometric_normal;
    }

    let normal_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data3, material_properties.data7.y);
    let tangent_normal = normalize_or_zero(textureSample(normal_tex, normal_sampler, normal_uv).xyz * 2.0 - vec3<f32>(1.0, 1.0, 1.0));
    if (length(tangent_normal) <= EPSILON) {
        return geometric_normal;
    }
    let world_normal = normalize_or_zero(tangent * tangent_normal.x + bitangent * tangent_normal.y + geometric_normal * tangent_normal.z);
    if (length(world_normal) <= EPSILON) {
        return geometric_normal;
    }
    return world_normal;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let encoded = sampled_world_normal(input) * 0.5 + vec3<f32>(0.5, 0.5, 0.5);
    return vec4<f32>(encoded, 1.0);
}
