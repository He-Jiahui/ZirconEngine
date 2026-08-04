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
    sky_sun_direction: vec4<f32>,
    sky_sun_color_radius: vec4<f32>,
    sky_sun_params: vec4<f32>,
    environment_params: vec4<f32>,
    environment_sample_params: vec4<f32>,
    environment_rotation_sin_cos: vec4<f32>,
};

const ZR_STANDARD_MATERIAL_MIN_ROUGHNESS: f32 = 0.001;
const EPSILON: f32 = 0.000001;

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var gbuffer_albedo_tex: texture_2d<f32>;
@group(1) @binding(1) var normal_tex: texture_2d<f32>;
@group(1) @binding(3) var gbuffer_material_tex: texture_2d<f32>;
@group(1) @binding(4) var scene_depth_tex: texture_depth_2d;
@group(1) @binding(5) var gbuffer_emissive_tex: texture_2d<f32>;

fn screen_uv_to_clip(uv: vec2<f32>, depth: f32) -> vec4<f32> {
    return vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, depth, 1.0);
}

fn reconstruct_world_position(coord: vec2<i32>, depth: f32) -> vec3<f32> {
    let viewport_size = max(textureDimensions(scene_depth_tex), vec2<u32>(1u, 1u));
    let uv = (vec2<f32>(coord) + vec2<f32>(0.5, 0.5)) / vec2<f32>(viewport_size);
    let world = scene.inverse_view_proj * screen_uv_to_clip(uv, depth);
    if (abs(world.w) <= EPSILON) {
        return vec3<f32>(0.0);
    }
    return world.xyz / world.w;
}

fn normalize_or_zero(value: vec3<f32>) -> vec3<f32> {
    let value_length = length(value);
    if (value_length <= EPSILON) {
        return vec3<f32>(0.0);
    }
    return value / value_length;
}

fn scene_view_dir_ws(world_position: vec3<f32>) -> vec3<f32> {
    let camera_direction_weight = clamp(scene.camera_view_direction.w, 0.0, 1.0);
    if (camera_direction_weight <= 0.0) {
        return normalize_or_zero(scene.camera_world_position.xyz - world_position);
    }
    if (camera_direction_weight >= 1.0) {
        return normalize_or_zero(scene.camera_view_direction.xyz);
    }
    let perspective_view_dir = normalize_or_zero(scene.camera_world_position.xyz - world_position);
    return normalize_or_zero(mix(
        perspective_view_dir,
        scene.camera_view_direction.xyz,
        camera_direction_weight,
    ));
}

fn shade_deferred_environment_only_pbr(
    coord: vec2<i32>,
    albedo: vec4<f32>,
    material: vec4<f32>,
    normal: vec3<f32>,
    emissive: vec3<f32>,
) -> vec4<f32> {
    let metallic = clamp(material.r, 0.0, 1.0);
    let roughness = clamp(
        max(material.g, ZR_STANDARD_MATERIAL_MIN_ROUGHNESS),
        ZR_STANDARD_MATERIAL_MIN_ROUGHNESS,
        1.0,
    );
    let occlusion = clamp(max(material.b, 0.0), 0.0, 1.0);
    let depth = clamp(textureLoad(scene_depth_tex, coord, 0), 0.0, 1.0);
    let world_position = reconstruct_world_position(coord, depth);
    let view_dir = scene_view_dir_ws(world_position);
    let diffuse_color = albedo.rgb;
    let ambient = scene.ambient_color.rgb * occlusion;
    let environment_lights = zr_environment_pbr_indirect(
        world_position,
        normal,
        view_dir,
        roughness,
        metallic,
        diffuse_color,
        albedo.rgb,
        occlusion,
        true,
    );
    let color = diffuse_color * (1.0 - metallic) * ambient
        + environment_lights
        + max(emissive, vec3<f32>(0.0));
    return vec4<f32>(color, albedo.a);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let coord = vec2<i32>(position.xy);
    let albedo = textureLoad(gbuffer_albedo_tex, coord, 0);
    if (albedo.a <= 0.001) {
        discard;
    }
    let encoded_normal = textureLoad(normal_tex, coord, 0).xyz;
    let normal = normalize_or_zero(encoded_normal * 2.0 - vec3<f32>(1.0));
    let material = textureLoad(gbuffer_material_tex, coord, 0);
    let emissive = textureLoad(gbuffer_emissive_tex, coord, 0).rgb;
    return shade_deferred_environment_only_pbr(coord, albedo, material, normal, emissive);
}
