struct DepthOfFieldPrepareParams {
    viewport: vec4<u32>,
    depth: vec4<f32>,
    lens: vec4<f32>,
    coc_output: vec4<f32>,
};

@group(0) @binding(0) var scene_depth_tex: texture_depth_2d;
@group(0) @binding(1) var<uniform> params: DepthOfFieldPrepareParams;
@group(0) @binding(2) var scene_color_tex: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct FragmentOutput {
    @location(0) coc: vec4<f32>,
    @location(1) bokeh: vec4<f32>,
};

const EPSILON: f32 = 0.000001;

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

fn load_scene_depth(coord: vec2<u32>) -> f32 {
    let viewport_size = max(params.viewport.xy, vec2<u32>(1u, 1u));
    let clamped = min(coord, viewport_size - vec2<u32>(1u, 1u));
    return clamp(textureLoad(scene_depth_tex, clamped, 0), 0.0, 1.0);
}

fn load_scene_color(coord: vec2<u32>) -> vec3<f32> {
    let viewport_size = max(params.viewport.xy, vec2<u32>(1u, 1u));
    let clamped = min(coord, viewport_size - vec2<u32>(1u, 1u));
    return textureLoad(scene_color_tex, clamped, 0).rgb;
}

fn linearize_scene_depth(raw_depth: f32) -> f32 {
    let near_plane = max(params.depth.x, 0.001);
    let far_plane = max(params.depth.y, near_plane + 0.001);
    if (params.depth.w > 0.5) {
        return (near_plane * far_plane)
            / max(far_plane - raw_depth * (far_plane - near_plane), 0.001);
    }
    return mix(near_plane, far_plane, raw_depth);
}

fn signed_circle_of_confusion_radius(view_depth: f32) -> f32 {
    let max_radius = max(params.coc_output.x, 0.0);
    let aperture = max(params.lens.z, 0.0);
    if (max_radius <= EPSILON || aperture <= EPSILON) {
        return 0.0;
    }

    let focus_depth = max(params.lens.x, params.depth.x);
    let focus_range = max(params.lens.y, 0.001);
    let focal_length_scale = clamp(params.lens.w / 50.0, 0.1, 6.0);
    let focus_delta = (view_depth - focus_depth) / focus_range;
    let magnitude = clamp(
        abs(focus_delta) * aperture * focal_length_scale * max_radius,
        0.0,
        max_radius
    );

    if (focus_delta < 0.0) {
        return -magnitude;
    }
    return magnitude;
}

fn circle_of_confusion_layers(coord: vec2<u32>) -> vec2<f32> {
    let view_depth = linearize_scene_depth(load_scene_depth(coord));
    let signed_radius = signed_circle_of_confusion_radius(view_depth);
    let normalized_radius = clamp(signed_radius * params.coc_output.y, -1.0, 1.0);
    return vec2<f32>(
        max(normalized_radius, 0.0),
        max(-normalized_radius, 0.0)
    );
}

fn bokeh_prefilter_weight(far_coc: f32, near_coc: f32) -> f32 {
    return smoothstep(0.0, 0.05, max(far_coc, near_coc));
}

fn clamp_prepare_coord(coord: vec2<i32>, viewport_size: vec2<u32>) -> vec2<u32> {
    let max_coord = vec2<i32>(viewport_size) - vec2<i32>(1, 1);
    return vec2<u32>(clamp(coord, vec2<i32>(0, 0), max_coord));
}

fn bokeh_prefilter_sample(sample_coord: vec2<u32>, kernel_weight: f32) -> vec4<f32> {
    let sample_coc = circle_of_confusion_layers(sample_coord);
    let sample_weight = bokeh_prefilter_weight(sample_coc.x, sample_coc.y) * kernel_weight;
    return vec4<f32>(load_scene_color(sample_coord) * sample_weight, sample_weight);
}

fn prefiltered_bokeh_seed(coord: vec2<u32>) -> vec4<f32> {
    let viewport_size = max(params.viewport.xy, vec2<u32>(1u, 1u));
    let coord_i32 = vec2<i32>(coord);
    var accumulated = bokeh_prefilter_sample(coord, 1.0);
    accumulated += bokeh_prefilter_sample(
        clamp_prepare_coord(coord_i32 + vec2<i32>(0, -1), viewport_size),
        0.5
    );
    accumulated += bokeh_prefilter_sample(
        clamp_prepare_coord(coord_i32 + vec2<i32>(0, 1), viewport_size),
        0.5
    );
    accumulated += bokeh_prefilter_sample(
        clamp_prepare_coord(coord_i32 + vec2<i32>(-1, 0), viewport_size),
        0.5
    );
    accumulated += bokeh_prefilter_sample(
        clamp_prepare_coord(coord_i32 + vec2<i32>(1, 0), viewport_size),
        0.5
    );

    let total_kernel_weight = 3.0;
    let coverage = clamp(accumulated.a / total_kernel_weight, 0.0, 1.0);
    if (accumulated.a <= EPSILON) {
        return vec4<f32>(load_scene_color(coord), 0.0);
    }
    return vec4<f32>(accumulated.rgb / accumulated.a, coverage);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> FragmentOutput {
    let viewport_size = max(params.viewport.xy, vec2<u32>(1u, 1u));
    let coord = min(vec2<u32>(position.xy), viewport_size - vec2<u32>(1u, 1u));
    let coc_layers = circle_of_confusion_layers(coord);
    let far_coc = coc_layers.x;
    let near_coc = coc_layers.y;
    let normalized_radius = far_coc - near_coc;
    let bokeh_seed = prefiltered_bokeh_seed(coord);

    var output: FragmentOutput;
    output.coc = vec4<f32>(
        far_coc,
        near_coc,
        normalized_radius * 0.5 + 0.5,
        params.coc_output.w
    );
    output.bokeh = bokeh_seed;
    return output;
}
