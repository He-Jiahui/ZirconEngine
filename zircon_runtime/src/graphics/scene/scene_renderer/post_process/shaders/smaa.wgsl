struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@group(0) @binding(0) var terminal_input_tex: texture_2d<f32>;
@group(0) @binding(1) var smaa_stage_tex: texture_2d<f32>;

struct TerminalRegionParams {
    viewport_origin: vec4<u32>,
};

@group(0) @binding(2) var<uniform> params: TerminalRegionParams;

const EDGE_THRESHOLD_LOW: f32 = 0.035;
const EDGE_THRESHOLD_HIGH: f32 = 0.18;
const EDGE_BLEND_LIMIT: f32 = 0.85;

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

fn saturate(value: f32) -> f32 {
    return clamp(value, 0.0, 1.0);
}

fn safe_extent(texture_extent: vec2<u32>) -> vec2<u32> {
    return max(texture_extent, vec2<u32>(1u, 1u));
}

fn clamp_coord(coord: vec2<i32>, extent: vec2<u32>) -> vec2<i32> {
    let max_coord = vec2<i32>(safe_extent(extent) - vec2<u32>(1u, 1u));
    return clamp(coord, vec2<i32>(0, 0), max_coord);
}

fn luma(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.299, 0.587, 0.114));
}

fn load_terminal_rgb(coord: vec2<i32>, extent: vec2<u32>) -> vec3<f32> {
    return textureLoad(terminal_input_tex, clamp_coord(coord, extent), 0).rgb;
}

fn load_stage_rg(coord: vec2<i32>, extent: vec2<u32>) -> vec2<f32> {
    return textureLoad(smaa_stage_tex, clamp_coord(coord, extent), 0).rg;
}

fn local_coord(position: vec4<f32>, extent: vec2<u32>) -> vec2<u32> {
    let max_coord = vec2<i32>(safe_extent(extent) - vec2<u32>(1u, 1u));
    return vec2<u32>(clamp(
        vec2<i32>(position.xy) - vec2<i32>(params.viewport_origin.xy),
        vec2<i32>(0, 0),
        max_coord
    ));
}

fn smaa_edge_weight(center_luma: f32, neighbor_luma: f32) -> f32 {
    let contrast = abs(center_luma - neighbor_luma);
    return smoothstep(EDGE_THRESHOLD_LOW, EDGE_THRESHOLD_HIGH, contrast);
}

fn detect_smaa_edges(coord: vec2<u32>) -> vec2<f32> {
    let extent = textureDimensions(terminal_input_tex);
    let coord_i32 = vec2<i32>(coord);
    let center = load_terminal_rgb(coord_i32, extent);
    let north = load_terminal_rgb(coord_i32 + vec2<i32>(0, -1), extent);
    let south = load_terminal_rgb(coord_i32 + vec2<i32>(0, 1), extent);
    let west = load_terminal_rgb(coord_i32 + vec2<i32>(-1, 0), extent);
    let east = load_terminal_rgb(coord_i32 + vec2<i32>(1, 0), extent);

    let center_luma = luma(center);
    let horizontal_edge =
        max(smaa_edge_weight(center_luma, luma(west)), smaa_edge_weight(center_luma, luma(east)));
    let vertical_edge =
        max(smaa_edge_weight(center_luma, luma(north)), smaa_edge_weight(center_luma, luma(south)));
    return vec2<f32>(horizontal_edge, vertical_edge);
}

fn compute_smaa_blend_weights(coord: vec2<u32>) -> vec2<f32> {
    let extent = textureDimensions(smaa_stage_tex);
    let coord_i32 = vec2<i32>(coord);
    let edge = load_stage_rg(coord_i32, extent);
    let west_edge = load_stage_rg(coord_i32 + vec2<i32>(-1, 0), extent);
    let east_edge = load_stage_rg(coord_i32 + vec2<i32>(1, 0), extent);
    let north_edge = load_stage_rg(coord_i32 + vec2<i32>(0, -1), extent);
    let south_edge = load_stage_rg(coord_i32 + vec2<i32>(0, 1), extent);

    let horizontal_continuity = max(west_edge.x, east_edge.x);
    let vertical_continuity = max(north_edge.y, south_edge.y);
    let horizontal_weight = saturate(edge.x * (0.55 + horizontal_continuity * 0.45));
    let vertical_weight = saturate(edge.y * (0.55 + vertical_continuity * 0.45));
    return vec2<f32>(horizontal_weight, vertical_weight);
}

fn apply_smaa_resolve(coord: vec2<u32>, color: vec3<f32>) -> vec3<f32> {
    let terminal_extent = textureDimensions(terminal_input_tex);
    let stage_extent = textureDimensions(smaa_stage_tex);
    let coord_i32 = vec2<i32>(coord);
    let blend = load_stage_rg(coord_i32, stage_extent);
    let total_weight = min(blend.x + blend.y, EDGE_BLEND_LIMIT);

    if (total_weight <= 0.0) {
        return color;
    }

    let north = load_terminal_rgb(coord_i32 + vec2<i32>(0, -1), terminal_extent);
    let south = load_terminal_rgb(coord_i32 + vec2<i32>(0, 1), terminal_extent);
    let west = load_terminal_rgb(coord_i32 + vec2<i32>(-1, 0), terminal_extent);
    let east = load_terminal_rgb(coord_i32 + vec2<i32>(1, 0), terminal_extent);

    let horizontal_color = (west + east) * 0.5;
    let vertical_color = (north + south) * 0.5;
    let axis_sum = max(blend.x + blend.y, 0.0001);
    let axis_mix = blend.y / axis_sum;
    let blend_color = mix(horizontal_color, vertical_color, axis_mix);
    return mix(color, blend_color, total_weight * 0.75);
}

@fragment
fn fs_edge(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let coord = local_coord(position, textureDimensions(terminal_input_tex));
    let edge = detect_smaa_edges(coord);
    return vec4<f32>(edge, 0.0, 1.0);
}

@fragment
fn fs_blend(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let coord = local_coord(position, textureDimensions(smaa_stage_tex));
    let blend = compute_smaa_blend_weights(coord);
    return vec4<f32>(blend, 0.0, 1.0);
}

@fragment
fn fs_resolve(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let terminal_extent = textureDimensions(terminal_input_tex);
    let coord = local_coord(position, terminal_extent);
    let center_coord = clamp_coord(vec2<i32>(coord), terminal_extent);
    let center = textureLoad(terminal_input_tex, center_coord, 0);
    return vec4<f32>(apply_smaa_resolve(coord, center.rgb), center.a);
}
