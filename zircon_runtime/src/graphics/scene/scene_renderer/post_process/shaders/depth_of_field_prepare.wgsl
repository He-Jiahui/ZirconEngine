// 景深预备 pass 同时写 CoC 分层与 bokeh seed；后续景深/blur 共享这两个局部目标。
// 调用方按相机投影与镜头设置上传参数，停用时清黑，让消费端自然退回无预备数据路径。
// xy 是本次有效尺寸，zw 是场景源原点；depth 描述标准设备深度的相机近远平面。
// CPU 的 GL/ANGLE 降级会按固定源码片段替换深度绑定与加载语句，保持这些片段完整。
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

// CoC 的 r/g 分别承载远景/近景归一化半径，b 保留有符号半径，a 标记有效输出；
// bokeh 的 RGB 是预滤颜色，a 是失焦覆盖度，供后续取样决定是否借用该颜色。
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
    return clamp(
        textureLoad(scene_depth_tex, params.viewport.zw + clamped, 0),
        0.0,
        1.0
    );
}

fn load_scene_color(coord: vec2<u32>) -> vec3<f32> {
    let viewport_size = max(params.viewport.xy, vec2<u32>(1u, 1u));
    let clamped = min(coord, viewport_size - vec2<u32>(1u, 1u));
    return textureLoad(scene_color_tex, params.viewport.zw + clamped, 0).rgb;
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

// 正负号保留焦平面两侧的信息，避免近景扩散与远景模糊被当作同一层处理。
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

// 只让具有失焦覆盖的邻域参与 seed，降低清晰背景被散景取样拖入前景的机会。
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
