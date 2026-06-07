@group(0) @binding(0) var motion_vector_tile_max_coarse_tex: texture_2d<f32>;

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

fn motion_vector_tile_texture_size() -> vec2<u32> {
    return max(textureDimensions(motion_vector_tile_max_coarse_tex), vec2<u32>(1u, 1u));
}

fn clamp_motion_vector_tile_coord(coord: vec2<i32>, tile_size: vec2<u32>) -> vec2<u32> {
    let max_coord = vec2<i32>(tile_size) - vec2<i32>(1, 1);
    return vec2<u32>(clamp(coord, vec2<i32>(0, 0), max_coord));
}

fn load_motion_vector_neighbor_candidate(
    coord: vec2<i32>,
    tile_size: vec2<u32>
) -> vec2<f32> {
    let clamped = clamp_motion_vector_tile_coord(coord, tile_size);
    return clamp(
        textureLoad(motion_vector_tile_max_coarse_tex, clamped, 0).rg,
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, 1.0)
    );
}

fn choose_motion_vector_neighbor_max(current: vec2<f32>, candidate: vec2<f32>) -> vec2<f32> {
    if (dot(candidate, candidate) > dot(current, current)) {
        return candidate;
    }

    return current;
}

fn motion_vector_neighbor_max(full_res_coord: vec2<u32>, tile_size: vec2<u32>) -> vec2<f32> {
    let coord_i32 = vec2<i32>(full_res_coord / vec2<u32>(4u, 4u));
    var neighbor_max = load_motion_vector_neighbor_candidate(coord_i32, tile_size);
    neighbor_max = choose_motion_vector_neighbor_max(
        neighbor_max,
        load_motion_vector_neighbor_candidate(coord_i32 + vec2<i32>(-1, -1), tile_size)
    );
    neighbor_max = choose_motion_vector_neighbor_max(
        neighbor_max,
        load_motion_vector_neighbor_candidate(coord_i32 + vec2<i32>(0, -1), tile_size)
    );
    neighbor_max = choose_motion_vector_neighbor_max(
        neighbor_max,
        load_motion_vector_neighbor_candidate(coord_i32 + vec2<i32>(1, -1), tile_size)
    );
    neighbor_max = choose_motion_vector_neighbor_max(
        neighbor_max,
        load_motion_vector_neighbor_candidate(coord_i32 + vec2<i32>(-1, 0), tile_size)
    );
    neighbor_max = choose_motion_vector_neighbor_max(
        neighbor_max,
        load_motion_vector_neighbor_candidate(coord_i32 + vec2<i32>(1, 0), tile_size)
    );
    neighbor_max = choose_motion_vector_neighbor_max(
        neighbor_max,
        load_motion_vector_neighbor_candidate(coord_i32 + vec2<i32>(-1, 1), tile_size)
    );
    neighbor_max = choose_motion_vector_neighbor_max(
        neighbor_max,
        load_motion_vector_neighbor_candidate(coord_i32 + vec2<i32>(0, 1), tile_size)
    );
    return choose_motion_vector_neighbor_max(
        neighbor_max,
        load_motion_vector_neighbor_candidate(coord_i32 + vec2<i32>(1, 1), tile_size)
    );
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let tile_size = motion_vector_tile_texture_size();
    let full_res_coord = vec2<u32>(position.xy);
    let neighbor_max = motion_vector_neighbor_max(full_res_coord, tile_size);
    return vec4<f32>(neighbor_max, 0.0, 1.0);
}
