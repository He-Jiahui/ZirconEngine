struct SceneUniform {
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    ambient_color: vec4<f32>,
    previous_view_proj: mat4x4<f32>,
    motion_params: vec4<f32>,
    point_light_position_range: array<vec4<f32>, 8>,
    point_light_color_intensity: array<vec4<f32>, 8>,
    point_light_params: vec4<f32>,
};
struct ShadowReceiverUniform {
    light_view_proj: mat4x4<f32>,
    params: vec4<f32>,
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
@group(1) @binding(0) var shadow_map_tex: texture_depth_2d;
@group(1) @binding(1) var<uniform> shadow_receiver: ShadowReceiverUniform;
@group(1) @binding(2) var shadow_compare_sampler: sampler_comparison;

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
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) vertex_color: vec4<f32>,
    @location(4) world_tangent: vec3<f32>,
    @location(5) tangent_handedness: f32,
    @location(6) uv1: vec2<f32>,
    @location(7) tint: vec4<f32>,
    @location(8) shadow_params: vec4<f32>,
    @location(9) motion_params: vec4<f32>,
};

struct MotionVectorVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) current_clip_position: vec4<f32>,
    @location(2) previous_clip_position: vec4<f32>,
    @location(3) uv1: vec2<f32>,
    @location(4) tint: vec4<f32>,
    @location(5) shadow_params: vec4<f32>,
    @location(6) motion_params: vec4<f32>,
};

struct SampledMaterial {
    albedo: vec4<f32>,
    metallic: f32,
    roughness: f32,
    occlusion: f32,
    emissive: vec3<f32>,
    unlit: f32,
};

const EPSILON: f32 = 0.000001;
const POINT_LIGHT_UNIFORM_LIMIT: u32 = 8u;

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

fn previous_skin_weight(joint_index: u32, weight: f32) -> f32 {
    if (weight <= EPSILON || joint_index >= zr_previous_skinned_joint_count()) {
        return 0.0;
    }
    return weight;
}

fn previous_skin_weight_sum(joint_indices: vec4<u32>, joint_weights: vec4<f32>) -> f32 {
    return previous_skin_weight(joint_indices.x, joint_weights.x)
        + previous_skin_weight(joint_indices.y, joint_weights.y)
        + previous_skin_weight(joint_indices.z, joint_weights.z)
        + previous_skin_weight(joint_indices.w, joint_weights.w);
}

fn skin_previous_vertex_position(position: vec3<f32>, joint_indices: vec4<u32>, joint_weights: vec4<f32>, motion_params: vec4<f32>) -> vec3<f32> {
    if (motion_params.z <= 0.5 || zr_previous_skinned_joint_count() == 0u) {
        return position;
    }

    let weight_sum = previous_skin_weight_sum(joint_indices, joint_weights);
    if (weight_sum <= EPSILON) {
        return position;
    }

    var skinned = vec3<f32>(0.0, 0.0, 0.0);
    let weight_x = previous_skin_weight(joint_indices.x, joint_weights.x);
    let weight_y = previous_skin_weight(joint_indices.y, joint_weights.y);
    let weight_z = previous_skin_weight(joint_indices.z, joint_weights.z);
    let weight_w = previous_skin_weight(joint_indices.w, joint_weights.w);
    if (weight_x > 0.0) {
        skinned = skinned + (zr_previous_skinned_joint_matrix(joint_indices.x) * vec4<f32>(position, 1.0)).xyz * (weight_x / weight_sum);
    }
    if (weight_y > 0.0) {
        skinned = skinned + (zr_previous_skinned_joint_matrix(joint_indices.y) * vec4<f32>(position, 1.0)).xyz * (weight_y / weight_sum);
    }
    if (weight_z > 0.0) {
        skinned = skinned + (zr_previous_skinned_joint_matrix(joint_indices.z) * vec4<f32>(position, 1.0)).xyz * (weight_z / weight_sum);
    }
    if (weight_w > 0.0) {
        skinned = skinned + (zr_previous_skinned_joint_matrix(joint_indices.w) * vec4<f32>(position, 1.0)).xyz * (weight_w / weight_sum);
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
    output.uv1 = input.uv1;
    output.world_position = world.xyz;
    output.vertex_color = input.color;
    output.world_tangent = normalize_or_zero((world_from_local * vec4<f32>(local_tangent, 0.0)).xyz);
    output.tangent_handedness = select(-1.0, 1.0, input.tangent.w >= 0.0);
    output.tint = zr_gpu_scene_tint(instance_index);
    output.shadow_params = zr_gpu_scene_shadow_params(instance_index);
    output.motion_params = motion_params;
    return output;
}

fn world_to_shadow_coord(world_position: vec3<f32>) -> vec4<f32> {
    let light_clip = shadow_receiver.light_view_proj * vec4<f32>(world_position, 1.0);
    if (abs(light_clip.w) <= EPSILON) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let light_ndc = light_clip.xyz / light_clip.w;
    if (any(light_ndc.xy < vec2<f32>(-1.0, -1.0)) || light_ndc.z < 0.0 || any(light_ndc > vec3<f32>(1.0, 1.0, 1.0))) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let shadow_uv = light_ndc.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return vec4<f32>(shadow_uv, light_ndc.z, 1.0);
}

fn sample_shadow_visibility(shadow_uv: vec2<f32>, receiver_depth: f32, offset: vec2<i32>) -> f32 {
    let shadow_size = max(textureDimensions(shadow_map_tex), vec2<u32>(1u, 1u));
    let shadow_texel = vec2<f32>(1.0, 1.0) / vec2<f32>(shadow_size);
    let sample_uv = clamp(shadow_uv + vec2<f32>(offset) * shadow_texel, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));
    return textureSampleCompare(shadow_map_tex, shadow_compare_sampler, sample_uv, receiver_depth);
}

fn shadow_visibility(world_position: vec3<f32>, shadow_params: vec4<f32>) -> f32 {
    if (shadow_receiver.params.x <= 0.5) {
        return 1.0;
    }
    if (shadow_params.z <= 0.5) {
        return 1.0;
    }

    let shadow_coord = world_to_shadow_coord(world_position);
    if (shadow_coord.w <= 0.0) {
        return 1.0;
    }

    let receiver_depth = clamp(shadow_coord.z - shadow_receiver.params.y, 0.0, 1.0);
    let offsets = array<vec2<i32>, 9>(
        vec2<i32>(-1, -1),
        vec2<i32>(0, -1),
        vec2<i32>(1, -1),
        vec2<i32>(-1, 0),
        vec2<i32>(0, 0),
        vec2<i32>(1, 0),
        vec2<i32>(-1, 1),
        vec2<i32>(0, 1),
        vec2<i32>(1, 1),
    );
    var lit = 0.0;
    for (var i = 0u; i < 9u; i = i + 1u) {
        lit = lit + sample_shadow_visibility(shadow_coord.xy, receiver_depth, offsets[i]);
    }

    return mix(clamp(shadow_receiver.params.z, 0.0, 1.0), 1.0, lit / 9.0);
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

fn sampled_material(input: VertexOutput) -> SampledMaterial {
    let base_color_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data2, material_properties.data7.x);
    let metallic_roughness_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data4, material_properties.data7.z);
    let occlusion_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data5, material_properties.data7.w);
    let emissive_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data6, material_properties.data1.w);
    let albedo = textureSample(albedo_tex, albedo_sampler, base_color_uv).rgba * input.tint * input.vertex_color;
    let metallic_roughness = textureSample(metallic_roughness_tex, metallic_roughness_sampler, metallic_roughness_uv);
    let metallic = clamp(material_properties.data0.x * metallic_roughness.b, 0.0, 1.0);
    var roughness = material_properties.data0.y;
    if (roughness <= 0.0) {
        roughness = 1.0;
    }
    roughness = clamp(roughness * metallic_roughness.g, 0.04, 1.0);
    var occlusion = material_properties.data0.z;
    if (occlusion <= 0.0) {
        occlusion = 1.0;
    }
    occlusion = clamp(occlusion * textureSample(occlusion_tex, occlusion_sampler, occlusion_uv).r, 0.0, 1.0);
    let emissive = max(material_properties.data1.rgb, vec3<f32>(0.0, 0.0, 0.0)) * textureSample(emissive_tex, emissive_sampler, emissive_uv).rgb;
    return SampledMaterial(albedo, metallic, roughness, occlusion, emissive, material_properties.data0.w);
}

fn point_light_lighting(world_position: vec3<f32>, world_normal: vec3<f32>, material: SampledMaterial, diffuse_color: vec3<f32>) -> vec3<f32> {
    let light_count = min(u32(max(scene.point_light_params.x, 0.0)), POINT_LIGHT_UNIFORM_LIMIT);
    var accumulated = vec3<f32>(0.0, 0.0, 0.0);

    for (var i = 0u; i < POINT_LIGHT_UNIFORM_LIMIT; i = i + 1u) {
        if (i >= light_count) {
            break;
        }
        let position_range = scene.point_light_position_range[i];
        let color_intensity = scene.point_light_color_intensity[i];
        let range = max(position_range.w, EPSILON);
        let intensity = max(color_intensity.w, 0.0);
        if (intensity <= 0.0) {
            continue;
        }

        let to_light = position_range.xyz - world_position;
        let distance_to_light = length(to_light);
        if (distance_to_light >= range) {
            continue;
        }

        let light_vector = to_light / max(distance_to_light, EPSILON);
        let attenuation = pow(clamp(1.0 - distance_to_light / range, 0.0, 1.0), 2.0);
        let lambert = max(dot(world_normal, light_vector), 0.0);
        let radiance = color_intensity.rgb * intensity * attenuation * material.occlusion;
        let half_dir = normalize_or_zero(light_vector + vec3<f32>(0.0, 0.0, 1.0));
        let specular_power = mix(64.0, 4.0, material.roughness);
        let specular_intensity = pow(max(dot(world_normal, half_dir), 0.0), specular_power) * mix(0.04, 1.0, material.metallic);
        accumulated = accumulated + diffuse_color * radiance * lambert + radiance * specular_intensity;
    }

    return accumulated;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(-scene.light_dir.xyz);
    let world_normal = sampled_world_normal(input);
    let lambert = max(dot(light_dir, world_normal), 0.0);
    let direct_visibility = shadow_visibility(input.world_position, input.shadow_params);
    let material = sampled_material(input);
    let ambient = scene.ambient_color.rgb * material.occlusion;
    let direct = scene.light_color.rgb * lambert * direct_visibility * material.occlusion;
    let half_dir = normalize_or_zero(light_dir + vec3<f32>(0.0, 0.0, 1.0));
    let specular_power = mix(64.0, 4.0, material.roughness);
    let specular_intensity = pow(max(dot(world_normal, half_dir), 0.0), specular_power) * mix(0.04, 1.0, material.metallic);
    let diffuse_color = material.albedo.rgb * (1.0 - material.metallic * 0.45);
    let point_lights = point_light_lighting(input.world_position, world_normal, material, diffuse_color);
    let lit = diffuse_color * (ambient + direct) + scene.light_color.rgb * specular_intensity * direct_visibility * material.occlusion + point_lights;
    let shaded = mix(lit, material.albedo.rgb, clamp(material.unlit, 0.0, 1.0)) + material.emissive;
    return vec4<f32>(shaded, material.albedo.a);
}

@vertex
fn vs_motion_vector(input: VertexInput, @builtin(instance_index) instance_index: u32) -> MotionVectorVertexOutput {
    var output: MotionVectorVertexOutput;
    let motion_params = zr_gpu_scene_motion_params(instance_index);
    let current_local_position = skin_vertex_position(input.position, input.joint_indices, input.joint_weights, motion_params);
    let previous_local_position = skin_previous_vertex_position(input.position, input.joint_indices, input.joint_weights, motion_params);
    let current_world = zr_world_from_local(instance_index) * vec4<f32>(current_local_position, 1.0);
    let previous_world = zr_previous_world_from_local(instance_index) * vec4<f32>(previous_local_position, 1.0);
    let current_clip = scene.view_proj * current_world;
    let previous_clip = scene.previous_view_proj * previous_world;
    output.clip_position = current_clip;
    output.uv = input.uv;
    output.uv1 = input.uv1;
    output.current_clip_position = current_clip;
    output.previous_clip_position = previous_clip;
    output.tint = zr_gpu_scene_tint(instance_index);
    output.shadow_params = zr_gpu_scene_shadow_params(instance_index);
    output.motion_params = motion_params;
    return output;
}

fn clip_to_motion_uv(clip_position: vec4<f32>) -> vec2<f32> {
    if (abs(clip_position.w) <= EPSILON) {
        return vec2<f32>(0.5, 0.5);
    }
    let ndc = clip_position.xy / clip_position.w;
    return vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
}

@fragment
fn fs_motion_vector(input: MotionVectorVertexOutput) -> @location(0) vec4<f32> {
    if (scene.motion_params.x <= 0.5 || input.motion_params.x <= 0.5) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    if (input.shadow_params.x > 0.5) {
        let base_color_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data2, material_properties.data7.x);
        let albedo = textureSample(albedo_tex, albedo_sampler, base_color_uv).rgba * input.tint;
        if (albedo.a < input.shadow_params.y) {
            discard;
        }
    }

    let current_uv = clip_to_motion_uv(input.current_clip_position);
    let previous_uv = clip_to_motion_uv(input.previous_clip_position);
    let velocity = clamp(current_uv - previous_uv, vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0));
    return vec4<f32>(velocity, 0.0, 1.0);
}
