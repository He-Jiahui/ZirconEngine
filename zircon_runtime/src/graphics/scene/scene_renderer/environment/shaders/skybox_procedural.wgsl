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
};
@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(0) @binding(1) var zr_environment_source_cube: texture_cube<f32>;
@group(0) @binding(2) var zr_environment_sampler: sampler;

const SKYBOX_SOURCE_CUBEMAP_KIND: f32 = 3.0;
const SKYBOX_EPSILON: f32 = 0.000001;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    let position = positions[vertex_index];
    var output: VertexOutput;
    output.clip_position = vec4<f32>(position, 1.0, 1.0);
    output.uv = vec2<f32>(position.x * 0.5 + 0.5, position.y * 0.5 + 0.5);
    return output;
}

fn skybox_rotated_direction_normalized(direction: vec3<f32>) -> vec3<f32> {
    let rotation = scene.environment_params.z;
    if (rotation == 0.0) {
        return direction;
    }
    let s = sin(rotation);
    let c = cos(rotation);
    return vec3<f32>(
        direction.x * c - direction.z * s,
        direction.y,
        direction.x * s + direction.z * c,
    );
}

fn skybox_normalize_or_fallback(value: vec3<f32>, fallback: vec3<f32>) -> vec3<f32> {
    let value_length = length(value);
    if (value_length <= SKYBOX_EPSILON) {
        return normalize(fallback);
    }
    return value / value_length;
}

fn skybox_fix_cube_lookup(direction: vec3<f32>, _lod: f32) -> vec3<f32> {
    // WGPU cube sampling filters across face edges natively. Do not apply the
    // legacy cmft/OpenGL edge warp to a direction before lookup.
    return direction;
}

fn skybox_world_direction_from_ndc(ndc: vec2<f32>) -> vec3<f32> {
    let fallback = normalize(vec3<f32>(ndc.x, ndc.y, -1.0));
    let far_world = scene.inverse_view_proj * vec4<f32>(ndc.x, ndc.y, 1.0, 1.0);
    let center_far_world = scene.inverse_view_proj * vec4<f32>(0.0, 0.0, 1.0, 1.0);
    let right_far_world = scene.inverse_view_proj * vec4<f32>(1.0, 0.0, 1.0, 1.0);
    let up_far_world = scene.inverse_view_proj * vec4<f32>(0.0, 1.0, 1.0, 1.0);
    if (abs(far_world.w) <= SKYBOX_EPSILON) {
        return fallback;
    }

    let far_position = far_world.xyz / far_world.w;
    let perspective_direction = skybox_normalize_or_fallback(
        far_position - scene.camera_world_position.xyz,
        fallback,
    );
    var orthographic_direction = perspective_direction;
    if (
        abs(center_far_world.w) > SKYBOX_EPSILON &&
        abs(right_far_world.w) > SKYBOX_EPSILON &&
        abs(up_far_world.w) > SKYBOX_EPSILON
    ) {
        let center_position = center_far_world.xyz / center_far_world.w;
        let right_position = right_far_world.xyz / right_far_world.w;
        let up_position = up_far_world.xyz / up_far_world.w;
        let camera_forward = skybox_normalize_or_fallback(
            center_position - scene.camera_world_position.xyz,
            -scene.camera_view_direction.xyz,
        );
        let camera_right = skybox_normalize_or_fallback(
            right_position - center_position,
            vec3<f32>(1.0, 0.0, 0.0),
        );
        let camera_up = skybox_normalize_or_fallback(
            up_position - center_position,
            vec3<f32>(0.0, 1.0, 0.0),
        );
        orthographic_direction = skybox_normalize_or_fallback(
            camera_forward + ndc.x * camera_right + ndc.y * camera_up,
            perspective_direction,
        );
    }
    return skybox_normalize_or_fallback(
        mix(
            perspective_direction,
            orthographic_direction,
            clamp(scene.camera_view_direction.w, 0.0, 1.0),
        ),
        perspective_direction,
    );
}

fn source_cubemap_sky_color(direction: vec3<f32>) -> vec3<f32> {
    let rotated = skybox_rotated_direction_normalized(direction);
    return textureSampleLevel(
        zr_environment_source_cube,
        zr_environment_sampler,
        skybox_fix_cube_lookup(rotated, 0.0),
        0.0,
    ).rgb * max(scene.environment_params.y, 0.0);
}

fn procedural_sun_radiance(direction: vec3<f32>) -> vec3<f32> {
    if (scene.sky_sun_direction.w < 0.5 || scene.sky_sun_params.x <= 0.0) {
        return vec3<f32>(0.0);
    }
    let sun_mask = smoothstep(
        scene.sky_sun_params.y,
        scene.sky_sun_params.z,
        dot(direction, scene.sky_sun_direction.xyz),
    );
    return scene.sky_sun_color_radius.rgb * scene.sky_sun_params.x * sun_mask;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let ndc = input.uv * 2.0 - vec2<f32>(1.0, 1.0);
    let direction = skybox_world_direction_from_ndc(ndc);
    var color: vec3<f32>;
    if (scene.environment_sample_params.x >= SKYBOX_SOURCE_CUBEMAP_KIND - 0.5) {
        color = source_cubemap_sky_color(direction);
    } else {
        let sky_t = clamp(direction.y * 0.5 + 0.5, 0.0, 1.0);
        let ground_t = clamp(direction.y + 1.0, 0.0, 1.0);
        let intensity = max(scene.environment_params.y, 0.0);
        let sky = mix(scene.sky_horizon_color.rgb, scene.sky_zenith_color.rgb, sky_t);
        let ground = mix(scene.sky_ground_color.rgb, scene.sky_horizon_color.rgb, ground_t);
        color = (select(ground, sky, direction.y >= 0.0)
            + procedural_sun_radiance(direction))
            * intensity;
    }
    return vec4<f32>(zr_volumetric_apply(color, input.clip_position.xy, 1.0), 1.0);
}
