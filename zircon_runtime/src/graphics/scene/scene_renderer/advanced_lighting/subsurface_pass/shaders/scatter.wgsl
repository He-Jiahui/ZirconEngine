const SSS_SHADING_MODEL_ID: u32 = 16u;
const BURLEY_SAMPLE_COUNT: u32 = 64u;
const GOLDEN_ANGLE: f32 = 2.39996323;
const SSS_EPSILON: f32 = 0.000001;

struct SubsurfaceParams {
    viewport_width: u32,
    viewport_height: u32,
    profile_count: u32,
    active_profile_mask: u32,
    inverse_view_projection: mat4x4<f32>,
};

struct SubsurfaceProfile {
    scatter_radius_and_world_scale: vec4<f32>,
    falloff_and_profile_id: vec4<f32>,
};

struct ChannelScatter {
    value: f32,
    accepted: f32,
};

@group(0) @binding(0) var sss_diffuse: texture_2d<f32>;
@group(0) @binding(1) var scene_depth: texture_depth_2d;
@group(0) @binding(2) var gbuffer_material: texture_2d<f32>;
@group(0) @binding(3) var gbuffer_normal: texture_2d<f32>;
@group(0) @binding(4) var<storage, read> tile_list: array<vec2<u32>>;
@group(0) @binding(5) var<uniform> profiles: array<SubsurfaceProfile, 16>;
@group(0) @binding(6) var<uniform> params: SubsurfaceParams;
@group(0) @binding(7) var sss_scattered: texture_storage_2d<rgba16float, write>;

fn decode_shading_model(material_sample: vec4<f32>) -> u32 {
    return u32(round(material_sample.a * 255.0)) & 0x7fu;
}

fn profile_index_at(pixel: vec2<i32>) -> u32 {
    return u32(round(textureLoad(gbuffer_normal, pixel, 0).a * 255.0));
}

fn profile_is_active(profile_index: u32) -> bool {
    return profile_index < params.profile_count
        && (params.active_profile_mask & (1u << profile_index)) != 0u;
}

fn reconstruct_world_position(pixel: vec2<i32>, device_depth: f32) -> vec3<f32> {
    let viewport = vec2<f32>(f32(params.viewport_width), f32(params.viewport_height));
    let uv = (vec2<f32>(pixel) + vec2<f32>(0.5)) / max(viewport, vec2<f32>(1.0));
    let clip = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, device_depth, 1.0);
    let world = params.inverse_view_projection * clip;
    let safe_w = select(-SSS_EPSILON, SSS_EPSILON, world.w >= 0.0);
    return world.xyz / select(safe_w, world.w, abs(world.w) > SSS_EPSILON);
}

fn profile_matches(
    pixel: vec2<i32>,
    profile_index: u32,
    center_depth: f32,
    center_normal: vec3<f32>,
    world_thickness: f32,
) -> bool {
    if (decode_shading_model(textureLoad(gbuffer_material, pixel, 0)) != SSS_SHADING_MODEL_ID
        || profile_index_at(pixel) != profile_index) {
        return false;
    }
    let sample_normal = normalize(textureLoad(gbuffer_normal, pixel, 0).xyz * 2.0 - vec3<f32>(1.0));
    if (dot(center_normal, sample_normal) < 0.55) {
        return false;
    }
    let sample_depth = textureLoad(scene_depth, pixel, 0);
    let sample_world = reconstruct_world_position(pixel, sample_depth);
    let center_plane_world = reconstruct_world_position(pixel, center_depth);
    return distance(sample_world, center_plane_world) <= world_thickness;
}

fn burley_radius(sample_index: u32, scatter_distance: f32) -> f32 {
    let component_index = sample_index % 4u;
    let stratum = f32(sample_index / 4u) + 0.5;
    let unit_sample = clamp(stratum / 16.0, 0.0001, 0.9999);
    let component_scale = select(scatter_distance * 3.0, scatter_distance, component_index == 0u);
    return -component_scale * log(1.0 - unit_sample);
}

fn scatter_channel(
    pixel: vec2<i32>,
    channel: u32,
    pixel_radius: f32,
    profile_index: u32,
    center_depth: f32,
    center_normal: vec3<f32>,
    world_thickness: f32,
) -> ChannelScatter {
    var accumulated = 0.0;
    var accepted = 0.0;
    for (var sample_index = 0u; sample_index < BURLEY_SAMPLE_COUNT; sample_index += 1u) {
        let angle = f32(sample_index) * GOLDEN_ANGLE;
        let radius = burley_radius(sample_index, pixel_radius);
        let offset = vec2<i32>(round(vec2<f32>(cos(angle), sin(angle)) * radius));
        let sample_pixel = clamp(
            pixel + offset,
            vec2<i32>(0),
            vec2<i32>(i32(params.viewport_width) - 1, i32(params.viewport_height) - 1),
        );
        if (profile_matches(
            sample_pixel,
            profile_index,
            center_depth,
            center_normal,
            world_thickness,
        )) {
            accumulated += textureLoad(sss_diffuse, sample_pixel, 0)[channel];
            accepted += 1.0;
        }
    }
    return ChannelScatter(accumulated / max(accepted, 1.0), accepted);
}

@compute @workgroup_size(8, 8, 1)
fn main(
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
    let tile = tile_list[workgroup_id.x];
    let pixel_u = tile * 8u + local_id.xy;
    if (pixel_u.x >= params.viewport_width || pixel_u.y >= params.viewport_height) {
        return;
    }

    let pixel = vec2<i32>(pixel_u);
    let profile_index = profile_index_at(pixel);
    if (decode_shading_model(textureLoad(gbuffer_material, pixel, 0)) != SSS_SHADING_MODEL_ID
        || !profile_is_active(profile_index)) {
        textureStore(sss_scattered, pixel, vec4<f32>(0.0));
        return;
    }

    let profile = profiles[profile_index];
    let center_depth = textureLoad(scene_depth, pixel, 0);
    let center_world = reconstruct_world_position(pixel, center_depth);
    let adjacent_pixel = vec2<i32>(min(pixel.x + 1, i32(params.viewport_width) - 1), pixel.y);
    let world_per_pixel = max(
        distance(center_world, reconstruct_world_position(adjacent_pixel, center_depth)),
        SSS_EPSILON,
    );
    let world_radii = max(
        profile.scatter_radius_and_world_scale.rgb * profile.scatter_radius_and_world_scale.w,
        vec3<f32>(0.0),
    );
    let pixel_radii = max(world_radii / world_per_pixel, vec3<f32>(0.5));
    let world_thickness = max(max(world_radii.x, max(world_radii.y, world_radii.z)) * 0.5, world_per_pixel * 2.0);
    let center_normal = normalize(textureLoad(gbuffer_normal, pixel, 0).xyz * 2.0 - vec3<f32>(1.0));
    let red = scatter_channel(pixel, 0u, pixel_radii.x, profile_index, center_depth, center_normal, world_thickness);
    let green = scatter_channel(pixel, 1u, pixel_radii.y, profile_index, center_depth, center_normal, world_thickness);
    let blue = scatter_channel(pixel, 2u, pixel_radii.z, profile_index, center_depth, center_normal, world_thickness);
    let center_diffuse = textureLoad(sss_diffuse, pixel, 0).rgb;
    let scattered = vec3<f32>(
        select(center_diffuse.r, red.value, red.accepted > 0.0),
        select(center_diffuse.g, green.value, green.accepted > 0.0),
        select(center_diffuse.b, blue.value, blue.accepted > 0.0),
    );
    let falloff = clamp(profile.falloff_and_profile_id.rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    textureStore(sss_scattered, pixel, vec4<f32>(mix(center_diffuse, scattered, falloff), 1.0));
}
