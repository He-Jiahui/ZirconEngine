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

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var gbuffer_albedo_tex: texture_2d<f32>;
@group(1) @binding(1) var normal_tex: texture_2d<f32>;
@group(1) @binding(2) var background_tex: texture_2d<f32>;
@group(1) @binding(3) var gbuffer_material_tex: texture_2d<f32>;
@group(1) @binding(4) var scene_depth_tex: texture_depth_2d;
@group(1) @binding(5) var shadow_map_tex: texture_depth_2d;
@group(1) @binding(6) var<uniform> shadow_receiver: ShadowReceiverUniform;
@group(1) @binding(7) var shadow_compare_sampler: sampler_comparison;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

const EPSILON: f32 = 0.000001;
const POINT_LIGHT_UNIFORM_LIMIT: u32 = 8u;

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

fn shadow_visibility(world_position: vec3<f32>) -> f32 {
    if (shadow_receiver.params.x <= 0.5) {
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

fn point_light_lighting(world_position: vec3<f32>, normal: vec3<f32>, roughness: f32, metallic: f32, occlusion: f32, diffuse_color: vec3<f32>) -> vec3<f32> {
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
        let lambert = max(dot(normal, light_vector), 0.0);
        let radiance = color_intensity.rgb * intensity * attenuation * occlusion;
        let half_dir = normalize(light_vector + vec3<f32>(0.0, 0.0, 1.0));
        let specular_power = mix(96.0, 8.0, roughness);
        let specular_strength = (1.0 - roughness) * mix(0.04, 1.0, metallic);
        let specular = pow(max(dot(normal, half_dir), 0.0), specular_power) * specular_strength;
        accumulated = accumulated + diffuse_color * radiance * lambert + radiance * specular;
    }

    return accumulated;
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
    let metallic = clamp(material.r, 0.0, 1.0);
    let roughness = clamp(max(material.g, 0.04), 0.04, 1.0);
    let occlusion = clamp(max(material.b, 0.0), 0.0, 1.0);
    let light_dir = normalize(-scene.light_dir.xyz);
    let view_dir = vec3<f32>(0.0, 0.0, 1.0);
    let half_dir = normalize(light_dir + view_dir);
    let lambert = max(dot(light_dir, normal), 0.0);
    let depth = clamp(textureLoad(scene_depth_tex, coord, 0), 0.0, 1.0);
    let world_position = reconstruct_world_position(coord, depth);
    let direct_visibility = shadow_visibility(world_position);
    let specular_power = mix(96.0, 8.0, roughness);
    let specular_strength = (1.0 - roughness) * mix(0.04, 1.0, metallic);
    let specular = pow(max(dot(normal, half_dir), 0.0), specular_power) * specular_strength;
    let lighting = (scene.ambient_color.rgb + scene.light_color.rgb * lambert * direct_visibility) * occlusion;
    let diffuse_color = albedo.rgb * mix(1.0, 0.55, metallic);
    let point_lights = point_light_lighting(world_position, normal, roughness, metallic, occlusion, diffuse_color);
    let color = diffuse_color * lighting + scene.light_color.rgb * specular * direct_visibility * occlusion + point_lights;
    return vec4<f32>(color, albedo.a);
}
