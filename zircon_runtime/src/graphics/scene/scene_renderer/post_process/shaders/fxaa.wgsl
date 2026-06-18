struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@group(0) @binding(0) var terminal_input_tex: texture_2d<f32>;

struct TerminalRegionParams {
    viewport_origin: vec4<u32>,
};

@group(0) @binding(1) var<uniform> params: TerminalRegionParams;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(3.0, 1.0),
        vec2<f32>(-1.0, 1.0),
    );

    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

fn luma(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.299, 0.587, 0.114));
}

fn load_rgb(coord: vec2<i32>, extent: vec2<u32>) -> vec3<f32> {
    let max_coord = vec2<i32>(extent - vec2<u32>(1u, 1u));
    let clamped = clamp(coord, vec2<i32>(0, 0), max_coord);
    return textureLoad(terminal_input_tex, clamped, 0).rgb;
}

fn apply_fxaa(coord: vec2<u32>, color: vec3<f32>) -> vec3<f32> {
    let extent = textureDimensions(terminal_input_tex);
    let coord_i32 = vec2<i32>(coord);
    let north = load_rgb(coord_i32 + vec2<i32>(0, -1), extent);
    let south = load_rgb(coord_i32 + vec2<i32>(0, 1), extent);
    let west = load_rgb(coord_i32 + vec2<i32>(-1, 0), extent);
    let east = load_rgb(coord_i32 + vec2<i32>(1, 0), extent);

    let luma_center = luma(color);
    let luma_north = luma(north);
    let luma_south = luma(south);
    let luma_west = luma(west);
    let luma_east = luma(east);
    let luma_min = min(luma_center, min(min(luma_north, luma_south), min(luma_west, luma_east)));
    let luma_max = max(luma_center, max(max(luma_north, luma_south), max(luma_west, luma_east)));
    let luma_range = luma_max - luma_min;
    if (luma_range < 0.03125) {
        return color;
    }

    let horizontal_edge = abs(luma_north + luma_south - 2.0 * luma_center);
    let vertical_edge = abs(luma_east + luma_west - 2.0 * luma_center);
    var neighbor_average = (east + west) * 0.5;
    if (horizontal_edge > vertical_edge) {
        neighbor_average = (north + south) * 0.5;
    }

    let blend = clamp(luma_range * 1.5, 0.0, 0.75);
    return mix(color, neighbor_average, blend);
}

fn local_terminal_coord(position: vec4<f32>) -> vec2<u32> {
    let extent = textureDimensions(terminal_input_tex);
    let max_coord = vec2<i32>(extent - vec2<u32>(1u, 1u));
    return vec2<u32>(clamp(
        vec2<i32>(position.xy) - vec2<i32>(params.viewport_origin.xy),
        vec2<i32>(0, 0),
        max_coord
    ));
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let coord = local_terminal_coord(position);
    let center = textureLoad(terminal_input_tex, vec2<i32>(coord), 0);
    return vec4<f32>(apply_fxaa(coord, center.rgb), center.a);
}
