struct SceneUniform {
    view_proj: mat4x4<f32>,
    view_proj_unjittered: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    ambient_color: vec4<f32>,
    lightmapped_ambient_color: vec4<f32>,
    previous_view_proj_unjittered: mat4x4<f32>,
    motion_params: vec4<f32>,
    jitter_params: vec4<f32>,
    camera_world_position: vec4<f32>,
    camera_view_direction: vec4<f32>,
    sky_horizon_color: vec4<f32>,
    sky_zenith_color: vec4<f32>,
    sky_ground_color: vec4<f32>,
    sky_sun_direction: vec4<f32>,
    sky_sun_color_radius: vec4<f32>,
    sky_sun_params: vec4<f32>,
    environment_params: vec4<f32>,
    environment_sample_params: vec4<f32>,
    environment_rotation_sin_cos: vec4<f32>,
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
    data9: vec4<f32>,
    data10: vec4<f32>,
    data11: vec4<f32>,
    data12: vec4<f32>,
    data13: vec4<f32>,
    data14: vec4<f32>,
    data15: vec4<f32>,
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
    @location(10) @interpolate(flat) instance_index: u32,
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
    dielectric_f0: vec3<f32>,
};

const EPSILON: f32 = 0.000001;
const ZR_SHADING_MODEL_UNLIT_ID: u32 = 0u;
const ZR_SHADING_MODEL_BLINN_PHONG_ID: u32 = 1u;
const ZR_SHADING_MODEL_STANDARD_PBR_ID: u32 = 2u;
const ZR_STANDARD_MATERIAL_MIN_ROUGHNESS: f32 = 0.001;

fn zr_environment_capture_roughness(roughness: f32) -> f32 {
    if (scene.sky_sun_params.w > 0.5) {
        return 1.0;
    }
    return roughness;
}

fn zr_scene_ambient_color(lightmapped: bool) -> vec3<f32> {
    return select(scene.ambient_color.rgb, scene.lightmapped_ambient_color.rgb, lightmapped);
}

fn decode_shading_model_id(encoded: f32) -> u32 {
    return u32(round(clamp(encoded, 0.0, 1.0) * 255.0));
}

fn skin_weight(instance_index: u32, joint_index: u32, weight: f32) -> f32 {
    if (weight <= EPSILON || joint_index >= zr_skinned_joint_count(instance_index)) {
        return 0.0;
    }
    return weight;
}

fn skin_weight_sum(instance_index: u32, joint_indices: vec4<u32>, joint_weights: vec4<f32>) -> f32 {
    return skin_weight(instance_index, joint_indices.x, joint_weights.x)
        + skin_weight(instance_index, joint_indices.y, joint_weights.y)
        + skin_weight(instance_index, joint_indices.z, joint_weights.z)
        + skin_weight(instance_index, joint_indices.w, joint_weights.w);
}

fn skin_vertex_position(instance_index: u32, position: vec3<f32>, joint_indices: vec4<u32>, joint_weights: vec4<f32>, motion_params: vec4<f32>) -> vec3<f32> {
    if (motion_params.y <= 0.5 || zr_skinned_joint_count(instance_index) == 0u) {
        return position;
    }

    let weight_sum = skin_weight_sum(instance_index, joint_indices, joint_weights);
    if (weight_sum <= EPSILON) {
        return position;
    }

    var skinned = vec3<f32>(0.0, 0.0, 0.0);
    let weight_x = skin_weight(instance_index, joint_indices.x, joint_weights.x);
    let weight_y = skin_weight(instance_index, joint_indices.y, joint_weights.y);
    let weight_z = skin_weight(instance_index, joint_indices.z, joint_weights.z);
    let weight_w = skin_weight(instance_index, joint_indices.w, joint_weights.w);
    if (weight_x > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(instance_index, joint_indices.x) * vec4<f32>(position, 1.0)).xyz * (weight_x / weight_sum);
    }
    if (weight_y > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(instance_index, joint_indices.y) * vec4<f32>(position, 1.0)).xyz * (weight_y / weight_sum);
    }
    if (weight_z > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(instance_index, joint_indices.z) * vec4<f32>(position, 1.0)).xyz * (weight_z / weight_sum);
    }
    if (weight_w > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(instance_index, joint_indices.w) * vec4<f32>(position, 1.0)).xyz * (weight_w / weight_sum);
    }
    return skinned;
}

fn previous_skin_weight(instance_index: u32, joint_index: u32, weight: f32) -> f32 {
    if (weight <= EPSILON || joint_index >= zr_previous_skinned_joint_count(instance_index)) {
        return 0.0;
    }
    return weight;
}

fn previous_skin_weight_sum(instance_index: u32, joint_indices: vec4<u32>, joint_weights: vec4<f32>) -> f32 {
    return previous_skin_weight(instance_index, joint_indices.x, joint_weights.x)
        + previous_skin_weight(instance_index, joint_indices.y, joint_weights.y)
        + previous_skin_weight(instance_index, joint_indices.z, joint_weights.z)
        + previous_skin_weight(instance_index, joint_indices.w, joint_weights.w);
}

fn skin_previous_vertex_position(instance_index: u32, position: vec3<f32>, joint_indices: vec4<u32>, joint_weights: vec4<f32>, motion_params: vec4<f32>) -> vec3<f32> {
    if (motion_params.z <= 0.5 || zr_previous_skinned_joint_count(instance_index) == 0u) {
        return position;
    }

    let weight_sum = previous_skin_weight_sum(instance_index, joint_indices, joint_weights);
    if (weight_sum <= EPSILON) {
        return position;
    }

    var skinned = vec3<f32>(0.0, 0.0, 0.0);
    let weight_x = previous_skin_weight(instance_index, joint_indices.x, joint_weights.x);
    let weight_y = previous_skin_weight(instance_index, joint_indices.y, joint_weights.y);
    let weight_z = previous_skin_weight(instance_index, joint_indices.z, joint_weights.z);
    let weight_w = previous_skin_weight(instance_index, joint_indices.w, joint_weights.w);
    if (weight_x > 0.0) {
        skinned = skinned + (zr_previous_skinned_joint_matrix(instance_index, joint_indices.x) * vec4<f32>(position, 1.0)).xyz * (weight_x / weight_sum);
    }
    if (weight_y > 0.0) {
        skinned = skinned + (zr_previous_skinned_joint_matrix(instance_index, joint_indices.y) * vec4<f32>(position, 1.0)).xyz * (weight_y / weight_sum);
    }
    if (weight_z > 0.0) {
        skinned = skinned + (zr_previous_skinned_joint_matrix(instance_index, joint_indices.z) * vec4<f32>(position, 1.0)).xyz * (weight_z / weight_sum);
    }
    if (weight_w > 0.0) {
        skinned = skinned + (zr_previous_skinned_joint_matrix(instance_index, joint_indices.w) * vec4<f32>(position, 1.0)).xyz * (weight_w / weight_sum);
    }
    return skinned;
}

fn skin_vertex_normal(instance_index: u32, normal: vec3<f32>, joint_indices: vec4<u32>, joint_weights: vec4<f32>, motion_params: vec4<f32>) -> vec3<f32> {
    if (motion_params.y <= 0.5 || zr_skinned_joint_count(instance_index) == 0u) {
        return normal;
    }

    let weight_sum = skin_weight_sum(instance_index, joint_indices, joint_weights);
    if (weight_sum <= EPSILON) {
        return normal;
    }

    var skinned = vec3<f32>(0.0, 0.0, 0.0);
    let weight_x = skin_weight(instance_index, joint_indices.x, joint_weights.x);
    let weight_y = skin_weight(instance_index, joint_indices.y, joint_weights.y);
    let weight_z = skin_weight(instance_index, joint_indices.z, joint_weights.z);
    let weight_w = skin_weight(instance_index, joint_indices.w, joint_weights.w);
    if (weight_x > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(instance_index, joint_indices.x) * vec4<f32>(normal, 0.0)).xyz * (weight_x / weight_sum);
    }
    if (weight_y > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(instance_index, joint_indices.y) * vec4<f32>(normal, 0.0)).xyz * (weight_y / weight_sum);
    }
    if (weight_z > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(instance_index, joint_indices.z) * vec4<f32>(normal, 0.0)).xyz * (weight_z / weight_sum);
    }
    if (weight_w > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(instance_index, joint_indices.w) * vec4<f32>(normal, 0.0)).xyz * (weight_w / weight_sum);
    }

    return zr_pbr_common_normalize_or_zero(skinned);
}

fn skin_vertex_tangent(instance_index: u32, tangent: vec3<f32>, joint_indices: vec4<u32>, joint_weights: vec4<f32>, motion_params: vec4<f32>) -> vec3<f32> {
    if (motion_params.y <= 0.5 || zr_skinned_joint_count(instance_index) == 0u) {
        return tangent;
    }

    let weight_sum = skin_weight_sum(instance_index, joint_indices, joint_weights);
    if (weight_sum <= EPSILON) {
        return tangent;
    }

    var skinned = vec3<f32>(0.0, 0.0, 0.0);
    let weight_x = skin_weight(instance_index, joint_indices.x, joint_weights.x);
    let weight_y = skin_weight(instance_index, joint_indices.y, joint_weights.y);
    let weight_z = skin_weight(instance_index, joint_indices.z, joint_weights.z);
    let weight_w = skin_weight(instance_index, joint_indices.w, joint_weights.w);
    if (weight_x > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(instance_index, joint_indices.x) * vec4<f32>(tangent, 0.0)).xyz * (weight_x / weight_sum);
    }
    if (weight_y > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(instance_index, joint_indices.y) * vec4<f32>(tangent, 0.0)).xyz * (weight_y / weight_sum);
    }
    if (weight_z > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(instance_index, joint_indices.z) * vec4<f32>(tangent, 0.0)).xyz * (weight_z / weight_sum);
    }
    if (weight_w > 0.0) {
        skinned = skinned + (zr_skinned_joint_matrix(instance_index, joint_indices.w) * vec4<f32>(tangent, 0.0)).xyz * (weight_w / weight_sum);
    }

    return zr_pbr_common_normalize_or_zero(skinned);
}

fn normalize_or_zero(value: vec3<f32>) -> vec3<f32> {
    return zr_pbr_common_normalize_or_zero(value);
}

fn fallback_pbr_material_f0(
    dielectric_f0: vec3<f32>,
    base_color: vec3<f32>,
    metallic: f32,
) -> vec3<f32> {
    return zr_pbr_material_f0(dielectric_f0, base_color, metallic);
}

fn transform_material_uv(
    uv: vec2<f32>,
    transform: vec4<f32>,
    rotation_sin_cos: vec2<f32>,
) -> vec2<f32> {
    let scaled = uv * transform.xy;
    let rotated = vec2<f32>(
        scaled.x * rotation_sin_cos.x - scaled.y * rotation_sin_cos.y,
        scaled.x * rotation_sin_cos.y + scaled.y * rotation_sin_cos.x,
    );
    return rotated + transform.zw;
}

fn material_uv_mask() -> u32 {
    return u32(round(clamp(material_properties.data7.x, 0.0, 63.0)));
}

fn select_material_uv(
    uv0: vec2<f32>,
    uv1: vec2<f32>,
    uv_mask: u32,
    slot: u32,
) -> vec2<f32> {
    if ((uv_mask & (1u << slot)) != 0u) {
        return uv1;
    }
    return uv0;
}

fn transform_material_uv_channel(
    uv0: vec2<f32>,
    uv1: vec2<f32>,
    transform: vec4<f32>,
    rotation_sin_cos: vec2<f32>,
    uv_mask: u32,
    slot: u32,
) -> vec2<f32> {
    return transform_material_uv(
        select_material_uv(uv0, uv1, uv_mask, slot),
        transform,
        rotation_sin_cos,
    );
}

@vertex
fn vs_main(input: VertexInput, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var output: VertexOutput;
    let instance = zr_gpu_scene_instance(instance_index);
    let world_from_local = instance.world_from_local;
    let instance_flags = instance.flags;
    let motion_params = zr_gpu_scene_motion_params(instance_index);
    let local_position = skin_vertex_position(instance_index, input.position, input.joint_indices, input.joint_weights, motion_params);
    let local_normal = skin_vertex_normal(instance_index, input.normal, input.joint_indices, input.joint_weights, motion_params);
    let local_tangent = skin_vertex_tangent(instance_index, input.tangent.xyz, input.joint_indices, input.joint_weights, motion_params);
    let world = world_from_local * vec4<f32>(local_position, 1.0);
    output.clip_position = scene.view_proj * world;
    output.world_normal = normalize_or_zero(zr_gpu_scene_normal_to_world_direction(world_from_local, instance_flags, local_normal));
    output.uv = input.uv;
    output.uv1 = input.uv1;
    output.world_position = world.xyz;
    output.vertex_color = input.color;
    output.world_tangent = normalize_or_zero(zr_gpu_scene_tangent_to_world_direction(world_from_local, instance_flags, local_tangent));
    output.tangent_handedness = select(-1.0, 1.0, input.tangent.w >= 0.0) * zr_gpu_scene_tangent_handedness_scale(instance_flags);
    output.tint = zr_gpu_scene_tint(instance_index);
    output.shadow_params = zr_gpu_scene_shadow_params(instance_index);
    output.motion_params = motion_params;
    output.instance_index = instance_index;
    return output;
}

fn sampled_world_normal(input: VertexOutput, uv_mask: u32) -> vec3<f32> {
    let geometric_normal = input.world_normal;
    if (input.motion_params.w <= 0.5 || dot(geometric_normal, geometric_normal) <= EPSILON * EPSILON) {
        return normalize_or_zero(geometric_normal);
    }

    let tangent = input.world_tangent;
    if (dot(tangent, tangent) <= EPSILON * EPSILON) {
        return normalize_or_zero(geometric_normal);
    }
    let bitangent = cross(geometric_normal, tangent) * input.tangent_handedness;
    if (dot(bitangent, bitangent) <= EPSILON * EPSILON) {
        return normalize_or_zero(geometric_normal);
    }

    let normal_uv = transform_material_uv_channel(
        input.uv,
        input.uv1,
        material_properties.data3,
        material_properties.data13.zw,
        uv_mask,
        1u,
    );
    let encoded_normal_xy = textureSampleBias(
        normal_tex,
        normal_sampler,
        normal_uv,
        scene.camera_world_position.w,
    ).xy;
    let sampled_normal = zr_reconstruct_bc5_normal(encoded_normal_xy);
    let tangent_normal = vec3<f32>(
        sampled_normal.xy * material_properties.data12.x,
        sampled_normal.z,
    );
    if (dot(tangent_normal, tangent_normal) <= EPSILON * EPSILON) {
        return normalize_or_zero(geometric_normal);
    }
    let world_normal = tangent * tangent_normal.x + bitangent * tangent_normal.y + geometric_normal * tangent_normal.z;
    if (dot(world_normal, world_normal) <= EPSILON * EPSILON) {
        return normalize_or_zero(geometric_normal);
    }
    return normalize_or_zero(world_normal);
}

fn sampled_base_color(input: VertexOutput, uv_mask: u32) -> vec4<f32> {
    let base_color_uv = transform_material_uv_channel(
        input.uv,
        input.uv1,
        material_properties.data2,
        material_properties.data13.xy,
        uv_mask,
        0u,
    );
    return textureSampleBias(albedo_tex, albedo_sampler, base_color_uv, scene.camera_world_position.w).rgba * input.tint * input.vertex_color;
}

fn sampled_material(input: VertexOutput, uv_mask: u32) -> SampledMaterial {
    let metallic_roughness_uv = transform_material_uv_channel(
        input.uv,
        input.uv1,
        material_properties.data4,
        material_properties.data14.xy,
        uv_mask,
        2u,
    );
    let occlusion_uv = transform_material_uv_channel(
        input.uv,
        input.uv1,
        material_properties.data5,
        material_properties.data14.zw,
        uv_mask,
        3u,
    );
    let emissive_uv = transform_material_uv_channel(
        input.uv,
        input.uv1,
        material_properties.data6,
        material_properties.data15.xy,
        uv_mask,
        4u,
    );
    let albedo = sampled_base_color(input, uv_mask);
    let metallic_roughness = textureSampleBias(metallic_roughness_tex, metallic_roughness_sampler, metallic_roughness_uv, scene.camera_world_position.w);
    let metallic = clamp(material_properties.data0.x * metallic_roughness.b, 0.0, 1.0);
    let roughness = clamp(
        material_properties.data0.y * metallic_roughness.g,
        ZR_STANDARD_MATERIAL_MIN_ROUGHNESS,
        1.0,
    );
    let occlusion_sample = textureSampleBias(occlusion_tex, occlusion_sampler, occlusion_uv, scene.camera_world_position.w).r;
    let occlusion = clamp(
        mix(1.0, occlusion_sample, clamp(material_properties.data0.z, 0.0, 1.0)),
        0.0,
        1.0,
    );
    let emissive = max(material_properties.data1.rgb, vec3<f32>(0.0, 0.0, 0.0)) * textureSampleBias(emissive_tex, emissive_sampler, emissive_uv, scene.camera_world_position.w).rgb;
    let shading_model_id = select(
        decode_shading_model_id(material_properties.data8.y),
        ZR_SHADING_MODEL_UNLIT_ID,
        material_properties.data0.w >= 0.5,
    );
    return SampledMaterial(
        albedo,
        metallic,
        roughness,
        occlusion,
        emissive,
        material_properties.data0.w,
        shading_model_id,
        vec3<f32>(clamp(material_properties.data12.y, 0.0, 1.0)),
    );
}

fn light_radiance(light: ZrGpuLightData) -> vec3<f32> {
    return max(light.color_intensity.rgb, vec3<f32>(0.0, 0.0, 0.0)) * max(light.color_intensity.w, 0.0);
}

fn material_diffuse_color(material: SampledMaterial) -> vec3<f32> {
    if (material.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID) {
        return zr_pbr_base_color(material.albedo.rgb);
    }
    return material.albedo.rgb;
}

fn shade_standard_pbr_light_vector_normalized(light_vector: vec3<f32>, radiance: vec3<f32>, normalized_world_normal: vec3<f32>, material: SampledMaterial, direct_diffuse_brdf: vec3<f32>, direct_f0: vec3<f32>, world_view: vec3<f32>) -> vec3<f32> {
    let lambert = max(dot(normalized_world_normal, light_vector), 0.0);
    let specular = zr_pbr_isotropic_ggx(
        normalized_world_normal,
        world_view,
        light_vector,
        material.roughness,
        direct_f0,
    );
    return (direct_diffuse_brdf + specular) * radiance * lambert;
}

fn shade_blinn_phong_light_vector(light_vector: vec3<f32>, radiance: vec3<f32>, world_normal: vec3<f32>, material: SampledMaterial, diffuse_color: vec3<f32>, world_view: vec3<f32>) -> vec3<f32> {
    let lambert = max(dot(world_normal, light_vector), 0.0);
    let half_dir = normalize_or_zero(light_vector + world_view);
    let specular_power = mix(96.0, 12.0, material.roughness);
    let specular_intensity = pow(max(dot(world_normal, half_dir), 0.0), specular_power) * (1.0 - material.roughness) * 0.5;
    return diffuse_color * radiance * lambert + radiance * specular_intensity;
}

fn shade_light_vector_normalized(light_vector: vec3<f32>, radiance: vec3<f32>, normalized_world_normal: vec3<f32>, material: SampledMaterial, diffuse_color: vec3<f32>, direct_diffuse_brdf: vec3<f32>, direct_f0: vec3<f32>, world_view: vec3<f32>) -> vec3<f32> {
    if (material.shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID) {
        return shade_blinn_phong_light_vector(light_vector, radiance, normalized_world_normal, material, diffuse_color, world_view);
    }
    return shade_standard_pbr_light_vector_normalized(light_vector, radiance, normalized_world_normal, material, direct_diffuse_brdf, direct_f0, world_view);
}

fn punctual_light_visibility(light: ZrGpuLightData, light_type: u32, light_vector_to_light: vec3<f32>, distance_to_light: f32, range: f32) -> f32 {
    var visibility = pow(clamp(1.0 - distance_to_light / range, 0.0, 1.0), 2.0);
    let light_to_surface = select(
        vec3<f32>(0.0, 0.0, 0.0),
        -light_vector_to_light,
        distance_to_light > EPSILON,
    );
    if (light_type == ZR_GPU_LIGHT_TYPE_SPOT) {
        let cone = dot(normalize_or_zero(light.direction_type.xyz), light_to_surface);
        let inner = light.spot_angles_size.x;
        let outer = light.spot_angles_size.y;
        visibility = visibility * clamp((cone - outer) / max(inner - outer, EPSILON), 0.0, 1.0);
    } else if (light_type == ZR_GPU_LIGHT_TYPE_RECT) {
        visibility = visibility * max(dot(normalize_or_zero(light.direction_type.xyz), light_to_surface), 0.0);
    }
    return visibility;
}

fn shade_gpu_light_index(light_index: u32, world_position: vec3<f32>, normalized_world_normal: vec3<f32>, material: SampledMaterial, diffuse_color: vec3<f32>, direct_diffuse_brdf: vec3<f32>, direct_f0: vec3<f32>, shadow_params: vec4<f32>, view_z: f32, world_view: vec3<f32>) -> vec3<f32> {
    if (light_index >= zr_gpu_scene_light_count()) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    let light = zr_gpu_light(light_index);
    let light_type = zr_gpu_light_type(light);
    let base_radiance = light_radiance(light) * zr_light_cookie_factor(light, world_position);
    if (length(base_radiance) <= EPSILON) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    if (light_type == ZR_GPU_LIGHT_TYPE_DIRECTIONAL) {
        let light_vector = normalize_or_zero(-light.direction_type.xyz);
        var direct_visibility = 1.0;
        if (shadow_params.z > 0.5) {
            direct_visibility = zr_gpu_light_shadow_visibility(light, light_type, world_position, view_z);
        }
        let radiance = base_radiance * direct_visibility;
        return shade_light_vector_normalized(light_vector, radiance, normalized_world_normal, material, diffuse_color, direct_diffuse_brdf, direct_f0, world_view);
    }

    let to_light = light.position_range.xyz - world_position;
    let distance_to_light = length(to_light);
    let range = max(light.position_range.w, EPSILON);
    if (distance_to_light >= range) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    let light_vector = to_light / max(distance_to_light, EPSILON);
    let visibility = punctual_light_visibility(light, light_type, light_vector, distance_to_light, range);
    if (visibility <= 0.0) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    var shadow_visibility = 1.0;
    if (shadow_params.z > 0.5) {
        shadow_visibility = zr_gpu_light_shadow_visibility(light, light_type, world_position, view_z);
    }
    return shade_light_vector_normalized(
        light_vector,
        base_radiance * visibility * shadow_visibility,
        normalized_world_normal,
        material,
        diffuse_color,
        direct_diffuse_brdf,
        direct_f0,
        world_view,
    );
}

fn gpu_light_lighting(frag_coord: vec2<f32>, world_position: vec3<f32>, world_normal_normalized: vec3<f32>, material: SampledMaterial, diffuse_color: vec3<f32>, shadow_params: vec4<f32>, view_dir_normalized: vec3<f32>) -> vec3<f32> {
    if (zr_light_grid_params.light_count == 0u || zr_light_grid_params.bin_count == 0u) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    let view_z = zr_light_view_z(world_position, zr_light_grid_params);
    let bin = zr_light_zbin_index(view_z, zr_light_grid_params);
    let header = zr_light_zbin_header(bin, zr_light_grid_params);
    if (header.x == 0xFFFFu || header.x > header.y) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    let normalized_world_normal = world_normal_normalized;
    let world_view = view_dir_normalized;
    var direct_f0 = vec3<f32>(0.0);
    var direct_diffuse_brdf = diffuse_color / ZR_PBR_EXTRAS_PI;
    if (material.shading_model_id != ZR_SHADING_MODEL_BLINN_PHONG_ID) {
        direct_f0 = fallback_pbr_material_f0(
            material.dielectric_f0,
            diffuse_color,
            material.metallic,
        );
        direct_diffuse_brdf = diffuse_color
            * zr_surface_metallic_diffuse_energy_scale(material.metallic)
            / ZR_PBR_EXTRAS_PI;
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
                normalized_world_normal,
                material,
                diffuse_color,
                direct_diffuse_brdf,
                direct_f0,
                shadow_params,
                view_z,
                world_view,
            );
            mask = mask & (mask - 1u);
        }
    }

    return accumulated;
}

@fragment
fn fs_main(
    input: VertexOutput,
    @builtin(front_facing) front_facing: bool,
) -> @location(0) vec4<f32> {
    let uv_mask = material_uv_mask();
    let sampled_normal = sampled_world_normal(input, uv_mask);
    let world_normal = select(-sampled_normal, sampled_normal, front_facing);
    var material = sampled_material(input, uv_mask);
    if (material.shading_model_id == ZR_SHADING_MODEL_UNLIT_ID) {
        let shaded = material.albedo.rgb + material.emissive;
        return vec4<f32>(
            zr_volumetric_apply(shaded, input.clip_position.xy, input.clip_position.z),
            material.albedo.a,
        );
    }
    material.roughness = zr_environment_capture_roughness(material.roughness);
    let ambient = zr_scene_ambient_color(
        zr_gpu_scene_has_lightmap(input.instance_index),
    ) * material.occlusion;
    let diffuse_color = material_diffuse_color(material);
    let view_dir = zr_pbr_view_direction_ws(input.world_position);
    let direct_lights = gpu_light_lighting(input.clip_position.xy, input.world_position, world_normal, material, diffuse_color, input.shadow_params, view_dir);
    var diffuse_energy_scale = vec3<f32>(1.0);
    if (material.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID) {
        diffuse_energy_scale = vec3<f32>(
            zr_surface_metallic_diffuse_energy_scale(material.metallic),
        );
    }
    let environment_lights = zr_environment_pbr_indirect_with_dielectric_f0_normalized(
        input.world_position,
        world_normal,
        view_dir,
        material.roughness,
        material.metallic,
        diffuse_color,
        diffuse_color,
        material.dielectric_f0,
        material.occlusion,
        material.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID,
    );
    let baked_indirect = diffuse_color * diffuse_energy_scale * material.occlusion * zr_lightmap_baked_irradiance(
        input.instance_index,
        input.uv1,
        input.world_position,
        world_normal,
    );
    let lit = diffuse_color * diffuse_energy_scale * ambient + direct_lights + environment_lights + baked_indirect;
    let shaded = lit + material.emissive;
    return vec4<f32>(
        zr_volumetric_apply(shaded, input.clip_position.xy, input.clip_position.z),
        material.albedo.a,
    );
}

@fragment
fn fs_taa_reactive_mask(input: VertexOutput) -> @location(0) f32 {
    let alpha = clamp(sampled_base_color(input, material_uv_mask()).a, 0.0, 1.0);
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
    let current_local_position = skin_vertex_position(instance_index, input.position, input.joint_indices, input.joint_weights, motion_params);
    let previous_local_position = skin_previous_vertex_position(instance_index, input.previous_position, input.joint_indices, input.joint_weights, motion_params);
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
        let base_color_uv = transform_material_uv_channel(
            input.uv,
            input.uv1,
            material_properties.data2,
            material_properties.data13.xy,
            material_uv_mask(),
            0u,
        );
        let albedo = textureSampleBias(albedo_tex, albedo_sampler, base_color_uv, scene.camera_world_position.w).rgba * input.tint;
        if (albedo.a < input.shadow_params.y) {
            discard;
        }
    }

    let current_uv = clip_to_velocity_uv(input.current_clip_position);
    let previous_uv = clip_to_velocity_uv(input.previous_clip_position);
    let velocity = clamp(current_uv - previous_uv, vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0));
    return vec4<f32>(velocity, 0.0, 1.0);
}
