@group(0) @binding(0) var motion_vector_source_tex: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
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

fn motion_vector_source_texture_size() -> vec2<u32> {
    return max(textureDimensions(motion_vector_source_tex), vec2<u32>(1u, 1u));
}

fn clamp_motion_vector_source_coord(coord: vec2<i32>, source_size: vec2<u32>) -> vec2<u32> {
    let max_coord = vec2<i32>(source_size) - vec2<i32>(1, 1);
    return vec2<u32>(clamp(coord, vec2<i32>(0, 0), max_coord));
}

fn load_motion_vector_source_tile_candidate(
    coord: vec2<i32>,
    source_size: vec2<u32>
) -> vec2<f32> {
    let clamped = clamp_motion_vector_source_coord(coord, source_size);
    return clamp(
        textureLoad(motion_vector_source_tex, clamped, 0).rg,
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, 1.0)
    );
}

fn choose_motion_vector_tile_max(current: vec2<f32>, candidate: vec2<f32>) -> vec2<f32> {
    if (dot(candidate, candidate) > dot(current, current)) {
        return candidate;
    }

    return current;
}

fn motion_vector_tile_max(tile_coord: vec2<u32>, source_size: vec2<u32>) -> vec2<f32> {
    let base_coord = vec2<i32>(tile_coord * vec2<u32>(2u, 2u));
    var tile_max = load_motion_vector_source_tile_candidate(base_coord, source_size);
    tile_max = choose_motion_vector_tile_max(
        tile_max,
        load_motion_vector_source_tile_candidate(base_coord + vec2<i32>(1, 0), source_size)
    );
    tile_max = choose_motion_vector_tile_max(
        tile_max,
        load_motion_vector_source_tile_candidate(base_coord + vec2<i32>(0, 1), source_size)
    );
    return choose_motion_vector_tile_max(
        tile_max,
        load_motion_vector_source_tile_candidate(base_coord + vec2<i32>(1, 1), source_size)
    );
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let source_size = motion_vector_source_texture_size();
    let tile_coord = vec2<u32>(position.xy);
    let tile_max = motion_vector_tile_max(tile_coord, source_size);
    return vec4<f32>(tile_max, 0.0, 1.0);
}
