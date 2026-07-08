struct SceneUniform {
    view_proj: mat4x4<f32>,
    view_proj_unjittered: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    ambient_color: vec4<f32>,
    previous_view_proj_unjittered: mat4x4<f32>,
    motion_params: vec4<f32>,
    jitter_params: vec4<f32>,
    camera_world_position: vec4<f32>,
    camera_view_direction: vec4<f32>,
    sky_horizon_color: vec4<f32>,
    sky_zenith_color: vec4<f32>,
    sky_ground_color: vec4<f32>,
    environment_params: vec4<f32>,
    environment_sample_params: vec4<f32>,
    environment_sh9: array<vec4<f32>, 9>,
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
    data8: vec4<f32>,
};
@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(2) @binding(0) var<uniform> material_properties: MaterialPropertyUniform;
@group(2) @binding(1) var albedo_tex: texture_2d<f32>;
@group(2) @binding(2) var albedo_sampler: sampler;
@group(2) @binding(3) var normal_tex: texture_2d<f32>;
@group(2) @binding(4) var normal_sampler: sampler;
@group(2) @binding(5) var metallic_roughness_tex: texture_2d<f32>;
@group(2) @binding(6) var metallic_roughness_sampler: sampler;
@group(2) @binding(7) var occlusion_tex: texture_2d<f32>;
@group(2) @binding(8) var occlusion_sampler: sampler;
@group(2) @binding(9) var emissive_tex: texture_2d<f32>;
@group(2) @binding(10) var emissive_sampler: sampler;

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

struct VelocityVertexInput {
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) joint_indices: vec4<u32>,
    @location(4) joint_weights: vec4<f32>,
    @location(7) uv1: vec2<f32>,
    @location(8) previous_position: vec3<f32>,
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

struct VelocityVertexOutput {
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
    shading_model_id: u32,
};

const EPSILON: f32 = 0.000001;
const ZR_SHADING_MODEL_UNLIT_ID: u32 = 0u;
const ZR_SHADING_MODEL_BLINN_PHONG_ID: u32 = 1u;
const ZR_SHADING_MODEL_STANDARD_PBR_ID: u32 = 2u;
const ZR_STANDARD_MATERIAL_MIN_ROUGHNESS: f32 = 0.001;

fn decode_shading_model_id(encoded: f32) -> u32 {
    return u32(round(clamp(encoded, 0.0, 1.0) * 255.0));
}

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

fn scene_view_dir_ws(world_position: vec3<f32>) -> vec3<f32> {
    let perspective_view_dir = normalize_or_zero(scene.camera_world_position.xyz - world_position);
    return normalize_or_zero(mix(
        perspective_view_dir,
        scene.camera_view_direction.xyz,
        clamp(scene.camera_view_direction.w, 0.0, 1.0),
    ));
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

fn sampled_base_color(input: VertexOutput) -> vec4<f32> {
    let base_color_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data2, material_properties.data7.x);
    return textureSample(albedo_tex, albedo_sampler, base_color_uv).rgba * input.tint * input.vertex_color;
}

fn sampled_material(input: VertexOutput) -> SampledMaterial {
    let metallic_roughness_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data4, material_properties.data7.z);
    let occlusion_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data5, material_properties.data7.w);
    let emissive_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data6, material_properties.data1.w);
    let albedo = sampled_base_color(input);
    let metallic_roughness = textureSample(metallic_roughness_tex, metallic_roughness_sampler, metallic_roughness_uv);
    let metallic = clamp(material_properties.data0.x * metallic_roughness.b, 0.0, 1.0);
    var roughness = material_properties.data0.y;
    if (roughness <= 0.0) {
        roughness = 1.0;
    }
    roughness = clamp(roughness * metallic_roughness.g, ZR_STANDARD_MATERIAL_MIN_ROUGHNESS, 1.0);
    var occlusion = material_properties.data0.z;
    if (occlusion <= 0.0) {
        occlusion = 1.0;
    }
    occlusion = clamp(occlusion * textureSample(occlusion_tex, occlusion_sampler, occlusion_uv).r, 0.0, 1.0);
    let emissive = max(material_properties.data1.rgb, vec3<f32>(0.0, 0.0, 0.0)) * textureSample(emissive_tex, emissive_sampler, emissive_uv).rgb;
    let shading_model_id = select(
        decode_shading_model_id(material_properties.data8.y),
        ZR_SHADING_MODEL_UNLIT_ID,
        material_properties.data0.w >= 0.5,
    );
    return SampledMaterial(albedo, metallic, roughness, occlusion, emissive, material_properties.data0.w, shading_model_id);
}

fn light_radiance(light: ZrGpuLightData) -> vec3<f32> {
    return max(light.color_intensity.rgb, vec3<f32>(0.0, 0.0, 0.0)) * max(light.color_intensity.w, 0.0);
}

fn material_diffuse_color(material: SampledMaterial) -> vec3<f32> {
    if (material.shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID) {
        return material.albedo.rgb;
    }
    return material.albedo.rgb * (1.0 - material.metallic * 0.45);
}

fn shade_standard_pbr_light_vector(light_vector: vec3<f32>, radiance: vec3<f32>, world_normal: vec3<f32>, material: SampledMaterial, diffuse_color: vec3<f32>) -> vec3<f32> {
    let lambert = max(dot(world_normal, light_vector), 0.0);
    let half_dir = normalize_or_zero(light_vector + vec3<f32>(0.0, 0.0, 1.0));
    let specular_power = mix(64.0, 4.0, material.roughness);
    let specular_intensity = pow(max(dot(world_normal, half_dir), 0.0), specular_power) * mix(0.04, 1.0, material.metallic);
    return diffuse_color * radiance * lambert + radiance * specular_intensity;
}

fn shade_blinn_phong_light_vector(light_vector: vec3<f32>, radiance: vec3<f32>, world_normal: vec3<f32>, material: SampledMaterial, diffuse_color: vec3<f32>) -> vec3<f32> {
    let lambert = max(dot(world_normal, light_vector), 0.0);
    let half_dir = normalize_or_zero(light_vector + vec3<f32>(0.0, 0.0, 1.0));
    let specular_power = mix(96.0, 12.0, material.roughness);
    let specular_intensity = pow(max(dot(world_normal, half_dir), 0.0), specular_power) * (1.0 - material.roughness) * 0.5;
    return diffuse_color * radiance * lambert + radiance * specular_intensity;
}

fn shade_light_vector(light_vector: vec3<f32>, radiance: vec3<f32>, world_normal: vec3<f32>, material: SampledMaterial, diffuse_color: vec3<f32>) -> vec3<f32> {
    if (material.shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID) {
        return shade_blinn_phong_light_vector(light_vector, radiance, world_normal, material, diffuse_color);
    }
    return shade_standard_pbr_light_vector(light_vector, radiance, world_normal, material, diffuse_color);
}

fn punctual_light_visibility(light: ZrGpuLightData, light_type: u32, world_position: vec3<f32>, distance_to_light: f32) -> f32 {
    let range = max(light.position_range.w, EPSILON);
    if (distance_to_light >= range) {
        return 0.0;
    }

    var visibility = pow(clamp(1.0 - distance_to_light / range, 0.0, 1.0), 2.0);
    if (light_type == ZR_GPU_LIGHT_TYPE_SPOT) {
        let light_to_surface = normalize_or_zero(world_position - light.position_range.xyz);
        let cone = dot(normalize_or_zero(light.direction_type.xyz), light_to_surface);
        let inner = light.spot_angles_size.x;
        let outer = light.spot_angles_size.y;
        visibility = visibility * clamp((cone - outer) / max(inner - outer, EPSILON), 0.0, 1.0);
    } else if (light_type == ZR_GPU_LIGHT_TYPE_RECT) {
        let light_to_surface = normalize_or_zero(world_position - light.position_range.xyz);
        visibility = visibility * max(dot(normalize_or_zero(light.direction_type.xyz), light_to_surface), 0.0);
    }
    return visibility;
}

fn shade_gpu_light_index(light_index: u32, world_position: vec3<f32>, world_normal: vec3<f32>, material: SampledMaterial, diffuse_color: vec3<f32>, shadow_params: vec4<f32>, view_z: f32) -> vec3<f32> {
    if (light_index >= zr_gpu_scene_light_count()) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    let light = zr_gpu_light(light_index);
    let light_type = zr_gpu_light_type(light);
    let base_radiance = light_radiance(light);
    if (length(base_radiance) <= EPSILON) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    if (light_type == ZR_GPU_LIGHT_TYPE_DIRECTIONAL) {
        let light_vector = normalize_or_zero(-light.direction_type.xyz);
        var direct_visibility = 1.0;
        if (shadow_params.z > 0.5) {
            direct_visibility = zr_gpu_light_shadow_visibility(light, light_type, world_position, view_z);
        }
        let radiance = base_radiance * direct_visibility * material.occlusion;
        return shade_light_vector(light_vector, radiance, world_normal, material, diffuse_color);
    }

    let to_light = light.position_range.xyz - world_position;
    let distance_to_light = length(to_light);
    let light_vector = to_light / max(distance_to_light, EPSILON);
    let visibility = punctual_light_visibility(light, light_type, world_position, distance_to_light);
    if (visibility <= 0.0) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    var shadow_visibility = 1.0;
    if (shadow_params.z > 0.5) {
        shadow_visibility = zr_gpu_light_shadow_visibility(light, light_type, world_position, view_z);
    }
    return shade_light_vector(
        light_vector,
        base_radiance * visibility * shadow_visibility * material.occlusion,
        world_normal,
        material,
        diffuse_color,
    );
}

fn gpu_light_lighting(frag_coord: vec2<f32>, world_position: vec3<f32>, world_normal: vec3<f32>, material: SampledMaterial, diffuse_color: vec3<f32>, shadow_params: vec4<f32>) -> vec3<f32> {
    if (zr_light_grid_params.light_count == 0u || zr_light_grid_params.bin_count == 0u) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    let view_z = zr_light_view_z(world_position, zr_light_grid_params);
    let bin = zr_light_zbin_index(view_z, zr_light_grid_params);
    let header = zr_light_zbin_header(bin, zr_light_grid_params);
    if (header.x == 0xFFFFu || header.x > header.y) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    let tile_base = zr_light_tile_base(frag_coord, zr_light_grid_params);
    var accumulated = vec3<f32>(0.0, 0.0, 0.0);
    for (var word = header.x / 32u; word <= header.y / 32u; word = word + 1u) {
        var mask = zr_light_mask_word(tile_base, bin, word, zr_light_grid_params);
        while (mask != 0u) {
            let bit_index = firstTrailingBit(mask);
            let light_index = word * 32u + bit_index;
            accumulated = accumulated + shade_gpu_light_index(
                light_index,
                world_position,
                world_normal,
                material,
                diffuse_color,
                shadow_params,
                view_z,
            );
            mask = mask & (mask - 1u);
        }
    }

    return accumulated;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let world_normal = sampled_world_normal(input);
    let material = sampled_material(input);
    if (material.shading_model_id == ZR_SHADING_MODEL_UNLIT_ID) {
        return vec4<f32>(material.albedo.rgb + material.emissive, material.albedo.a);
    }
    let ambient = scene.ambient_color.rgb * material.occlusion;
    let diffuse_color = material_diffuse_color(material);
    let view_dir = scene_view_dir_ws(input.world_position);
    let direct_lights = gpu_light_lighting(input.clip_position.xy, input.world_position, world_normal, material, diffuse_color, input.shadow_params);
    let environment_lights = zr_environment_pbr_indirect(
        world_normal,
        view_dir,
        material.roughness,
        material.metallic,
        diffuse_color,
        material.albedo.rgb,
        material.occlusion,
        material.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID,
    );
    let lit = diffuse_color * ambient + direct_lights + environment_lights;
    let shaded = lit + material.emissive;
    return vec4<f32>(shaded, material.albedo.a);
}

@fragment
fn fs_taa_reactive_mask(input: VertexOutput) -> @location(0) f32 {
    let alpha = clamp(sampled_base_color(input).a, 0.0, 1.0);
    let authored_strength = clamp(material_properties.data8.x, 0.0, 1.0);
    let reactive_mask = max(alpha, authored_strength);
    if (reactive_mask <= EPSILON) {
        discard;
    }
    return reactive_mask;
}

@fragment
fn fs_taa_reactive_material_mask(_input: VertexOutput) -> @location(0) f32 {
    let reactive_mask = clamp(material_properties.data8.x, 0.0, 1.0);
    if (reactive_mask <= EPSILON) {
        discard;
    }
    return reactive_mask;
}

@vertex
fn vs_velocity_object(input: VelocityVertexInput, @builtin(instance_index) instance_index: u32) -> VelocityVertexOutput {
    var output: VelocityVertexOutput;
    let motion_params = zr_gpu_scene_motion_params(instance_index);
    let current_local_position = skin_vertex_position(input.position, input.joint_indices, input.joint_weights, motion_params);
    let previous_local_position = skin_previous_vertex_position(input.previous_position, input.joint_indices, input.joint_weights, motion_params);
    let current_world = zr_world_from_local(instance_index) * vec4<f32>(current_local_position, 1.0);
    let previous_world = zr_previous_world_from_local(instance_index) * vec4<f32>(previous_local_position, 1.0);
    let current_clip = scene.view_proj_unjittered * current_world;
    let previous_clip = scene.previous_view_proj_unjittered * previous_world;
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

fn clip_to_velocity_uv(clip_position: vec4<f32>) -> vec2<f32> {
    if (abs(clip_position.w) <= EPSILON) {
        return vec2<f32>(0.5, 0.5);
    }
    let ndc = clip_position.xy / clip_position.w;
    return vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
}

@fragment
fn fs_velocity_object(input: VelocityVertexOutput) -> @location(0) vec4<f32> {
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

    let current_uv = clip_to_velocity_uv(input.current_clip_position);
    let previous_uv = clip_to_velocity_uv(input.previous_clip_position);
    let velocity = clamp(current_uv - previous_uv, vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0));
    return vec4<f32>(velocity, 0.0, 1.0);
}
