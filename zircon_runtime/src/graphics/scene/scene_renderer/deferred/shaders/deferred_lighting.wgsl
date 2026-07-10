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

const ZR_STANDARD_MATERIAL_MIN_ROUGHNESS: f32 = 0.001;

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var gbuffer_albedo_tex: texture_2d<f32>;
@group(1) @binding(1) var normal_tex: texture_2d<f32>;
@group(1) @binding(2) var background_tex: texture_2d<f32>;
@group(1) @binding(3) var gbuffer_material_tex: texture_2d<f32>;
@group(1) @binding(4) var scene_depth_tex: texture_depth_2d;
@group(1) @binding(5) var gbuffer_emissive_tex: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

const EPSILON: f32 = 0.000001;
const ZR_SHADING_MODEL_UNLIT_ID: u32 = 0u;
const ZR_SHADING_MODEL_BLINN_PHONG_ID: u32 = 1u;
const ZR_SHADING_MODEL_STANDARD_PBR_ID: u32 = 2u;
const ZR_DEFERRED_MATERIAL_SHADING_MODEL_MASK: u32 = 0x7Fu;
const ZR_DEFERRED_MATERIAL_RECEIVE_SHADOWS_FLAG: u32 = 0x80u;

fn deferred_material_flags(encoded: f32) -> u32 {
    return u32(round(clamp(encoded, 0.0, 1.0) * 255.0));
}

fn decode_shading_model_id(encoded: f32) -> u32 {
    return deferred_material_flags(encoded) & ZR_DEFERRED_MATERIAL_SHADING_MODEL_MASK;
}

fn decode_receive_shadows(encoded: f32) -> bool {
    return (deferred_material_flags(encoded) & ZR_DEFERRED_MATERIAL_RECEIVE_SHADOWS_FLAG) != 0u;
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    var output: VertexOutput;
    output.clip_position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

fn screen_uv_to_clip(uv: vec2<f32>, depth: f32) -> vec4<f32> {
    return vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, depth, 1.0);
}

fn reconstruct_world_position(coord: vec2<i32>, depth: f32) -> vec3<f32> {
    let viewport_size = max(textureDimensions(scene_depth_tex), vec2<u32>(1u, 1u));
    let uv = (vec2<f32>(coord) + vec2<f32>(0.5, 0.5)) / vec2<f32>(viewport_size);
    let world = scene.inverse_view_proj * screen_uv_to_clip(uv, depth);
    if (abs(world.w) <= EPSILON) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    return world.xyz / world.w;
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

fn light_radiance(light: ZrGpuLightData) -> vec3<f32> {
    return max(light.color_intensity.rgb, vec3<f32>(0.0, 0.0, 0.0)) * max(light.color_intensity.w, 0.0);
}

fn shade_standard_pbr_light_vector(light_vector: vec3<f32>, radiance: vec3<f32>, normal: vec3<f32>, roughness: f32, metallic: f32, diffuse_color: vec3<f32>, view_dir: vec3<f32>) -> vec3<f32> {
    let lambert = max(dot(normal, light_vector), 0.0);
    let half_dir = normalize_or_zero(light_vector + view_dir);
    let specular_power = mix(96.0, 8.0, roughness);
    let specular_strength = (1.0 - roughness) * mix(0.04, 1.0, metallic);
    let specular = pow(max(dot(normal, half_dir), 0.0), specular_power) * specular_strength;
    return diffuse_color * radiance * lambert + radiance * specular;
}

fn shade_blinn_phong_light_vector(light_vector: vec3<f32>, radiance: vec3<f32>, normal: vec3<f32>, roughness: f32, diffuse_color: vec3<f32>, view_dir: vec3<f32>) -> vec3<f32> {
    let lambert = max(dot(normal, light_vector), 0.0);
    let half_dir = normalize_or_zero(light_vector + view_dir);
    let specular_power = mix(96.0, 12.0, roughness);
    let specular = pow(max(dot(normal, half_dir), 0.0), specular_power) * (1.0 - roughness) * 0.5;
    return diffuse_color * radiance * lambert + radiance * specular;
}

fn shade_light_vector(light_vector: vec3<f32>, radiance: vec3<f32>, normal: vec3<f32>, roughness: f32, metallic: f32, diffuse_color: vec3<f32>, view_dir: vec3<f32>, shading_model_id: u32) -> vec3<f32> {
    if (shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID) {
        return shade_blinn_phong_light_vector(light_vector, radiance, normal, roughness, diffuse_color, view_dir);
    }
    return shade_standard_pbr_light_vector(light_vector, radiance, normal, roughness, metallic, diffuse_color, view_dir);
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

fn shade_gpu_light_index(light_index: u32, world_position: vec3<f32>, normal: vec3<f32>, roughness: f32, metallic: f32, occlusion: f32, diffuse_color: vec3<f32>, view_dir: vec3<f32>, view_z: f32, shading_model_id: u32, receive_shadows: bool) -> vec3<f32> {
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
        if (receive_shadows) {
            direct_visibility = zr_gpu_light_shadow_visibility(light, light_type, world_position, view_z);
        }
        return shade_light_vector(
            light_vector,
            base_radiance * direct_visibility * occlusion,
            normal,
            roughness,
            metallic,
            diffuse_color,
            view_dir,
            shading_model_id,
        );
    }

    let to_light = light.position_range.xyz - world_position;
    let distance_to_light = length(to_light);
    let visibility = punctual_light_visibility(light, light_type, world_position, distance_to_light);
    if (visibility <= 0.0) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    var shadow_visibility = 1.0;
    if (receive_shadows) {
        shadow_visibility = zr_gpu_light_shadow_visibility(light, light_type, world_position, view_z);
    }
    return shade_light_vector(
        to_light / max(distance_to_light, EPSILON),
        base_radiance * visibility * shadow_visibility * occlusion,
        normal,
        roughness,
        metallic,
        diffuse_color,
        view_dir,
        shading_model_id,
    );
}

fn gpu_light_lighting(frag_coord: vec2<f32>, world_position: vec3<f32>, normal: vec3<f32>, roughness: f32, metallic: f32, occlusion: f32, diffuse_color: vec3<f32>, view_dir: vec3<f32>, shading_model_id: u32, receive_shadows: bool) -> vec3<f32> {
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
                normal,
                roughness,
                metallic,
                occlusion,
                diffuse_color,
                view_dir,
                view_z,
                shading_model_id,
                receive_shadows,
            );
            mask = mask & (mask - 1u);
        }
    }

    return accumulated;
}

fn deferred_diffuse_color(albedo: vec4<f32>, metallic: f32, shading_model_id: u32) -> vec3<f32> {
    if (shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID) {
        return albedo.rgb;
    }
    return albedo.rgb * mix(1.0, 0.55, metallic);
}

fn add_deferred_emissive(shaded: vec4<f32>, emissive: vec3<f32>) -> vec4<f32> {
    return vec4<f32>(shaded.rgb + max(emissive, vec3<f32>(0.0)), shaded.a);
}

fn shade_deferred_lit(position: vec4<f32>, coord: vec2<i32>, albedo: vec4<f32>, material: vec4<f32>, normal: vec3<f32>, shading_model_id: u32) -> vec4<f32> {
    let metallic = clamp(material.r, 0.0, 1.0);
    let roughness = clamp(max(material.g, ZR_STANDARD_MATERIAL_MIN_ROUGHNESS), ZR_STANDARD_MATERIAL_MIN_ROUGHNESS, 1.0);
    let occlusion = clamp(max(material.b, 0.0), 0.0, 1.0);
    let receive_shadows = decode_receive_shadows(material.a);
    let depth = clamp(textureLoad(scene_depth_tex, coord, 0), 0.0, 1.0);
    let world_position = reconstruct_world_position(coord, depth);
    let view_dir = scene_view_dir_ws(world_position);
    let ambient = scene.ambient_color.rgb * occlusion;
    let diffuse_color = deferred_diffuse_color(albedo, metallic, shading_model_id);
    let direct_lights = gpu_light_lighting(position.xy, world_position, normal, roughness, metallic, occlusion, diffuse_color, view_dir, shading_model_id, receive_shadows);
    let environment_lights = zr_environment_pbr_indirect(
        world_position,
        normal,
        view_dir,
        roughness,
        metallic,
        diffuse_color,
        albedo.rgb,
        occlusion,
        shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID,
    );
    let color = diffuse_color * ambient + direct_lights + environment_lights;
    return vec4<f32>(color, albedo.a);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let coord = vec2<i32>(position.xy);
    let albedo = textureLoad(gbuffer_albedo_tex, coord, 0);
    let background = textureLoad(background_tex, coord, 0);
    if (albedo.a <= 0.001) {
        return background;
    }

    let encoded_normal = textureLoad(normal_tex, coord, 0).xyz;
    let normal = normalize(encoded_normal * 2.0 - vec3<f32>(1.0, 1.0, 1.0));
    let material = textureLoad(gbuffer_material_tex, coord, 0);
    let emissive = textureLoad(gbuffer_emissive_tex, coord, 0).rgb;
    let shading_model_id = decode_shading_model_id(material.a);
    if (shading_model_id == ZR_SHADING_MODEL_UNLIT_ID) {
        return add_deferred_emissive(shade_deferred_unlit(albedo), emissive);
    }
    if (shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID) {
        return add_deferred_emissive(
            shade_deferred_blinn_phong(position, coord, albedo, material, normal),
            emissive,
        );
    }
    // zr-deferred-lighting-custom-shading-model-dispatch
    return add_deferred_emissive(
        shade_deferred_standard_pbr(position, coord, albedo, material, normal),
        emissive,
    );
}
