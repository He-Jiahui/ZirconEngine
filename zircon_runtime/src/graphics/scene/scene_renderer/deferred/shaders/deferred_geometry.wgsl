struct SceneUniform {
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    ambient_color: vec4<f32>,
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
@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;
@group(2) @binding(2) var normal_tex: texture_2d<f32>;
@group(2) @binding(3) var normal_sampler: sampler;
@group(2) @binding(4) var metallic_roughness_tex: texture_2d<f32>;
@group(2) @binding(5) var metallic_roughness_sampler: sampler;
@group(2) @binding(6) var occlusion_tex: texture_2d<f32>;
@group(2) @binding(7) var occlusion_sampler: sampler;
@group(2) @binding(8) var emissive_tex: texture_2d<f32>;
@group(2) @binding(9) var emissive_sampler: sampler;
@group(2) @binding(10) var<uniform> material_properties: MaterialPropertyUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) joint_indices: vec4<u32>,
    @location(4) joint_weights: vec4<f32>,
    @location(5) tangent: vec4<f32>,
    @location(6) color: vec4<f32>,
    @location(7) uv1: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) vertex_color: vec4<f32>,
    @location(2) uv1: vec2<f32>,
    @location(3) tint: vec4<f32>,
};

struct DeferredGeometryOutput {
    @location(0) albedo: vec4<f32>,
    @location(1) material: vec4<f32>,
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
    let local_position = skin_vertex_position(
        input.position,
        input.joint_indices,
        input.joint_weights,
        zr_gpu_scene_motion_params(instance_index)
    );
    let world = zr_world_from_local(instance_index) * vec4<f32>(local_position, 1.0);
    output.clip_position = scene.view_proj * world;
    output.uv = input.uv;
    output.vertex_color = input.color;
    output.uv1 = input.uv1;
    output.tint = zr_gpu_scene_tint(instance_index);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> DeferredGeometryOutput {
    let base_color_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data2, material_properties.data7.x);
    let metallic_roughness_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data4, material_properties.data7.z);
    let occlusion_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data5, material_properties.data7.w);
    let albedo = textureSample(albedo_tex, albedo_sampler, base_color_uv) * input.tint * input.vertex_color;
    let metallic_roughness = textureSample(metallic_roughness_tex, metallic_roughness_sampler, metallic_roughness_uv);
    let metallic = clamp(material_properties.data0.x * metallic_roughness.b, 0.0, 1.0);
    var roughness = material_properties.data0.y;
    if (roughness <= 0.0) {
        roughness = 1.0;
    }
    roughness = roughness * metallic_roughness.g;
    var occlusion = material_properties.data0.z;
    if (occlusion <= 0.0) {
        occlusion = 1.0;
    }
    occlusion = occlusion * textureSample(occlusion_tex, occlusion_sampler, occlusion_uv).r;
    return DeferredGeometryOutput(
        albedo,
        vec4<f32>(metallic, clamp(roughness, 0.04, 1.0), clamp(occlusion, 0.0, 1.0), 1.0)
    );
}
