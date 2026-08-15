struct TaaResolveParams {
    viewport_and_flags: vec4<u32>,
    blend_and_clamp: vec4<f32>,
    responsive_and_reactive: vec4<f32>,
};

@group(0) @binding(0) var scene_color_tex: texture_2d<f32>;
@group(0) @binding(1) var scene_depth_tex: texture_depth_2d;
@group(0) @binding(2) var scene_velocity_tex: texture_2d<f32>;
@group(0) @binding(3) var taa_history_previous_tex: texture_2d<f32>;
@group(0) @binding(4) var<uniform> params: TaaResolveParams;
@group(0) @binding(5) var taa_reactive_mask_tex: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct TaaResolveOutput {
    @location(0) resolved_scene_color: vec4<f32>,
    @location(1) current_history: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    var output: VertexOutput;
    output.clip_position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

fn viewport_size() -> vec2<u32> {
    return max(params.viewport_and_flags.xy, vec2<u32>(1u, 1u));
}

fn clamp_coord(coord: vec2<i32>, size: vec2<u32>) -> vec2<i32> {
    let max_coord = vec2<i32>(i32(size.x) - 1, i32(size.y) - 1);
    return clamp(coord, vec2<i32>(0, 0), max_coord);
}

fn max3(value: vec3<f32>) -> f32 {
    return max(max(value.x, value.y), value.z);
}

fn luminance(rgb: vec3<f32>) -> f32 {
    return dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn rgb_to_ycocg(rgb: vec3<f32>) -> vec3<f32> {
    let y = dot(rgb, vec3<f32>(0.25, 0.5, 0.25));
    let co = dot(rgb, vec3<f32>(0.5, 0.0, -0.5));
    let cg = dot(rgb, vec3<f32>(-0.25, 0.5, -0.25));
    return vec3<f32>(y, co, cg);
}

fn ycocg_to_rgb(ycocg: vec3<f32>) -> vec3<f32> {
    let r = ycocg.x + ycocg.y - ycocg.z;
    let g = ycocg.x + ycocg.z;
    let b = ycocg.x - ycocg.y - ycocg.z;
    return max(vec3<f32>(r, g, b), vec3<f32>(0.0, 0.0, 0.0));
}

fn clip_towards_aabb_center(history_color: vec3<f32>, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> vec3<f32> {
    let center = 0.5 * (aabb_max + aabb_min);
    let extent = 0.5 * (aabb_max - aabb_min) + vec3<f32>(0.00000001, 0.00000001, 0.00000001);
    let offset = history_color - center;
    let unit = abs(offset / extent);
    let max_unit = max3(unit);
    if (max_unit > 1.0) {
        return center + offset / max_unit;
    }
    return history_color;
}

fn load_scene_depth_coord(coord: vec2<i32>, size: vec2<u32>) -> f32 {
    let clamped = vec2<u32>(clamp_coord(coord, size));
    return clamp(textureLoad(scene_depth_tex, clamped, 0), 0.0, 1.0);
}

fn load_scene_depth(coord: vec2<u32>) -> f32 {
    let size = viewport_size();
    return load_scene_depth_coord(vec2<i32>(coord), size);
}

fn load_scene_color(coord: vec2<i32>, size: vec2<u32>) -> vec4<f32> {
    return textureLoad(scene_color_tex, clamp_coord(coord, size), 0);
}

fn closest_depth_coord(coord: vec2<i32>, size: vec2<u32>) -> vec2<i32> {
    var closest_coord = clamp_coord(coord, size);
    var closest_depth = load_scene_depth_coord(closest_coord, size);
    for (var y = -1; y <= 1; y = y + 1) {
        for (var x = -1; x <= 1; x = x + 1) {
            let sample_coord = clamp_coord(coord + vec2<i32>(x, y), size);
            let sample_depth = load_scene_depth_coord(sample_coord, size);
            if (sample_depth > 0.0 && (closest_depth <= 0.0 || sample_depth < closest_depth)) {
                closest_depth = sample_depth;
                closest_coord = sample_coord;
            }
        }
    }
    return closest_coord;
}

fn scene_color_neighborhood_ycocg_bounds(coord: vec2<i32>, size: vec2<u32>) -> array<vec3<f32>, 2> {
    var moment_1 = vec3<f32>(0.0, 0.0, 0.0);
    var moment_2 = vec3<f32>(0.0, 0.0, 0.0);
    for (var y = -1; y <= 1; y = y + 1) {
        for (var x = -1; x <= 1; x = x + 1) {
            let sample_color = rgb_to_ycocg(load_scene_color(coord + vec2<i32>(x, y), size).rgb);
            moment_1 += sample_color;
            moment_2 += sample_color * sample_color;
        }
    }
    let mean = moment_1 / 9.0;
    let variance = max((moment_2 / 9.0) - (mean * mean), vec3<f32>(0.0, 0.0, 0.0));
    let deviation = sqrt(variance) * max(params.blend_and_clamp.z, 0.0);
    return array<vec3<f32>, 2>(mean - deviation, mean + deviation);
}

fn reproject_history_coord(coord: vec2<u32>, velocity: vec2<f32>, size: vec2<u32>) -> vec2<f32> {
    return (vec2<f32>(coord) + vec2<f32>(0.5, 0.5)) - velocity * vec2<f32>(size);
}

fn sample_reprojected_history(history_pixel: vec2<f32>, size: vec2<u32>) -> vec4<f32> {
    let rounded = vec2<i32>(round(history_pixel - vec2<f32>(0.5, 0.5)));
    return textureLoad(taa_history_previous_tex, clamp_coord(rounded, size), 0);
}

fn load_authored_reactive_mask(coord: vec2<i32>) -> f32 {
    let mask_size = vec2<i32>(textureDimensions(taa_reactive_mask_tex));
    let mask_coord = clamp(coord, vec2<i32>(0), mask_size - vec2<i32>(1));
    return clamp(textureLoad(taa_reactive_mask_tex, mask_coord, 0).r, 0.0, 1.0);
}

fn responsive_rejection(current_color: vec3<f32>, history_color: vec3<f32>, velocity: vec2<f32>) -> f32 {
    let luma_delta = abs(luminance(current_color) - luminance(history_color));
    let luma_threshold = max(params.responsive_and_reactive.x, 0.000001);
    let luma_rejection = smoothstep(luma_threshold, luma_threshold * 4.0, luma_delta);
    let velocity_rejection = clamp(length(velocity) * params.responsive_and_reactive.y, 0.0, 1.0);
    return max(luma_rejection, velocity_rejection);
}

fn history_weight(history_pixel: vec2<f32>, velocity: vec2<f32>, depth: f32, depth_delta: f32, history_confidence: f32, responsive: f32, size: vec2<u32>) -> f32 {
    if (params.viewport_and_flags.z == 0u) {
        return 0.0;
    }
    let inside =
        history_pixel.x >= 0.5 &&
        history_pixel.y >= 0.5 &&
        history_pixel.x < f32(size.x) - 0.5 &&
        history_pixel.y < f32(size.y) - 0.5;
    if (!inside || depth <= 0.0 || depth >= 1.0) {
        return 0.0;
    }
    let motion_rejection = clamp(1.0 - length(velocity) * params.blend_and_clamp.y, 0.0, 1.0);
    let depth_threshold = max(params.blend_and_clamp.w, 0.000001);
    let depth_rejection = clamp(depth_delta / depth_threshold, 0.0, 1.0);
    let responsive_multiplier = mix(
        1.0,
        clamp(params.responsive_and_reactive.z, 0.0, 1.0),
        clamp(responsive, 0.0, 1.0)
    );
    return clamp(
        params.blend_and_clamp.x *
            motion_rejection *
            (1.0 - depth_rejection) *
            responsive_multiplier *
            clamp(history_confidence, 0.0, 1.0),
        0.0,
        0.98
    );
}

fn next_history_confidence(previous_confidence: f32, weight: f32, responsive: f32) -> f32 {
    if (weight <= 0.0) {
        return 0.25;
    }
    let recovered = clamp(previous_confidence + 0.15, 0.25, 1.0);
    let responsive_cap = clamp(params.responsive_and_reactive.w, 0.25, 1.0);
    let confidence_cap = mix(1.0, responsive_cap, clamp(responsive, 0.0, 1.0));
    return min(recovered, confidence_cap);
}

@fragment
fn fs_taa_resolve(@builtin(position) position: vec4<f32>) -> TaaResolveOutput {
    let size = viewport_size();
    let coord = min(vec2<u32>(u32(position.x), u32(position.y)), size - vec2<u32>(1u, 1u));
    let coord_i32 = vec2<i32>(coord);
    let current = textureLoad(scene_color_tex, coord_i32, 0);
    let depth = load_scene_depth(coord);
    let velocity_coord = closest_depth_coord(coord_i32, size);
    let velocity = textureLoad(scene_velocity_tex, velocity_coord, 0).xy;
    let foreground_depth = load_scene_depth_coord(velocity_coord, size);
    let depth_delta = abs(depth - foreground_depth);
    let history_pixel = reproject_history_coord(coord, velocity, size);
    let history = sample_reprojected_history(history_pixel, size);
    let neighborhood = scene_color_neighborhood_ycocg_bounds(coord_i32, size);
    let clamped_history_ycocg = clip_towards_aabb_center(
        rgb_to_ycocg(history.rgb),
        neighborhood[0],
        neighborhood[1]
    );
    let clamped_history = ycocg_to_rgb(clamped_history_ycocg);
    let authored_reactive = load_authored_reactive_mask(coord_i32);
    let responsive = max(responsive_rejection(current.rgb, clamped_history, velocity), authored_reactive);
    let weight = history_weight(history_pixel, velocity, depth, depth_delta, history.a, responsive, size);
    let resolved = vec4<f32>(mix(current.rgb, clamped_history, weight), current.a);

    var output: TaaResolveOutput;
    output.resolved_scene_color = resolved;
    output.current_history = vec4<f32>(resolved.rgb, next_history_confidence(history.a, weight, responsive));
    return output;
}
