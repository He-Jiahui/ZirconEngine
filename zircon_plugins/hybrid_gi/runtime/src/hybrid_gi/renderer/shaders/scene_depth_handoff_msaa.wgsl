@group(0) @binding(0)
var scene_depth_tex: texture_depth_multisampled_2d;

@group(0) @binding(1)
var scene_hzb_tex: texture_2d<f32>;

@group(0) @binding(2)
var<storage, read_write> hybrid_gi_scene_words: array<u32>;

const HYBRID_GI_SCENE_DEPTH_HANDOFF_MAGIC: u32 = 0x48474944u;
const DEPTH_Q24_SCALE: f32 = 16777215.0;
const SCENE_HZB_TILE_GRID_EXTENT: u32 = 8u;
const SCENE_HZB_TILE_WORD_OFFSET: u32 = 16u;
const SCENE_HZB_TILE_WORD_COUNT: u32 = 4u;
const SCENE_HZB_VALID_FLAG: u32 = 1u << 31u;

fn conservative_resolved_depth(coord: vec2<i32>) -> f32 {
    let sample_count = textureNumSamples(scene_depth_tex);
    var resolved = 1.0;
    for (var sample_index = 0u; sample_index < sample_count; sample_index = sample_index + 1u) {
        resolved = min(
            resolved,
            clamp(textureLoad(scene_depth_tex, coord, sample_index), 0.0, 1.0),
        );
    }
    return resolved;
}

fn quantize_depth_q24(depth: f32) -> u32 {
    return u32((clamp(depth, 0.0, 1.0) * DEPTH_Q24_SCALE) + 0.5);
}

fn hzb_mip_for_tile(scene_size: vec2<u32>, mip_count: u32) -> u32 {
    let tile_extent = max(
        (scene_size.x + SCENE_HZB_TILE_GRID_EXTENT - 1u) / SCENE_HZB_TILE_GRID_EXTENT,
        (scene_size.y + SCENE_HZB_TILE_GRID_EXTENT - 1u) / SCENE_HZB_TILE_GRID_EXTENT,
    );
    var mip_level = 0u;
    var covered_scene_extent = 2u;
    while (covered_scene_extent < tile_extent && mip_level + 1u < mip_count) {
        covered_scene_extent = covered_scene_extent * 2u;
        mip_level = mip_level + 1u;
    }
    return mip_level;
}

fn valid_hzb_extent(scene_size: vec2<u32>, mip_level: u32) -> vec2<u32> {
    let shift = mip_level + 1u;
    let divisor = 1u << shift;
    return max(
        vec2<u32>(1u),
        (scene_size + vec2<u32>(divisor - 1u)) / divisor,
    );
}

fn hzb_coord_for_tile(
    tile_coord: vec2<u32>,
    scene_size: vec2<u32>,
    mip_level: u32,
) -> vec2<i32> {
    let texture_size = textureDimensions(scene_hzb_tex, mip_level);
    let valid_extent = min(valid_hzb_extent(scene_size, mip_level), texture_size);
    let coord = min(
        ((tile_coord * 2u + vec2<u32>(1u)) * valid_extent) /
            (SCENE_HZB_TILE_GRID_EXTENT * 2u),
        valid_extent - vec2<u32>(1u),
    );
    return vec2<i32>(coord);
}

fn scene_coord_for_tile(tile_coord: vec2<u32>, scene_size: vec2<u32>) -> vec2<i32> {
    let coord = min(
        ((tile_coord * 2u + vec2<u32>(1u)) * scene_size) /
            (SCENE_HZB_TILE_GRID_EXTENT * 2u),
        scene_size - vec2<u32>(1u),
    );
    return vec2<i32>(coord);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= SCENE_HZB_TILE_GRID_EXTENT ||
        global_id.y >= SCENE_HZB_TILE_GRID_EXTENT ||
        global_id.z != 0u) {
        return;
    }

    let dimensions = textureDimensions(scene_depth_tex);
    let hzb_dimensions = textureDimensions(scene_hzb_tex, 0);
    let hzb_mip_count = textureNumLevels(scene_hzb_tex);
    let tile_coord = global_id.xy;
    let mip_level = hzb_mip_for_tile(dimensions, hzb_mip_count);
    let scene_coord = scene_coord_for_tile(tile_coord, dimensions);
    let hzb_coord = hzb_coord_for_tile(tile_coord, dimensions, mip_level);
    let depth = conservative_resolved_depth(scene_coord);
    let hzb_range = textureLoad(scene_hzb_tex, hzb_coord, mip_level);

    let tile_index = tile_coord.y * SCENE_HZB_TILE_GRID_EXTENT + tile_coord.x;
    let tile_word_offset =
        SCENE_HZB_TILE_WORD_OFFSET + tile_index * SCENE_HZB_TILE_WORD_COUNT;
    hybrid_gi_scene_words[tile_word_offset] = quantize_depth_q24(depth);
    hybrid_gi_scene_words[tile_word_offset + 1u] = quantize_depth_q24(hzb_range.x);
    hybrid_gi_scene_words[tile_word_offset + 2u] = quantize_depth_q24(hzb_range.y);
    hybrid_gi_scene_words[tile_word_offset + 3u] = mip_level | SCENE_HZB_VALID_FLAG;

    if (tile_index != 0u) {
        return;
    }

    let center = vec2<i32>(i32(dimensions.x / 2u), i32(dimensions.y / 2u));
    let center_hzb_coord = min(
        center / 2,
        vec2<i32>(hzb_dimensions) - vec2<i32>(1),
    );
    let center_depth = conservative_resolved_depth(center);
    let center_hzb_range = textureLoad(scene_hzb_tex, center_hzb_coord, 0);
    let center_furthest_depth = center_hzb_range.x;
    let center_closest_depth = center_hzb_range.y;
    let coarsest_mip_level = hzb_mip_count - 1u;
    let coarsest_hzb_range =
        textureLoad(scene_hzb_tex, vec2<i32>(0), coarsest_mip_level);

    hybrid_gi_scene_words[0] = HYBRID_GI_SCENE_DEPTH_HANDOFF_MAGIC;
    hybrid_gi_scene_words[1] = dimensions.x;
    hybrid_gi_scene_words[2] = dimensions.y;
    hybrid_gi_scene_words[3] = quantize_depth_q24(center_depth);
    hybrid_gi_scene_words[4] = textureNumSamples(scene_depth_tex);
    hybrid_gi_scene_words[5] = hzb_dimensions.x;
    hybrid_gi_scene_words[6] = hzb_dimensions.y;
    hybrid_gi_scene_words[7] = hzb_mip_count;
    hybrid_gi_scene_words[8] = quantize_depth_q24(center_furthest_depth);
    hybrid_gi_scene_words[9] = quantize_depth_q24(center_closest_depth);
    hybrid_gi_scene_words[10] = quantize_depth_q24(coarsest_hzb_range.x);
    hybrid_gi_scene_words[11] = quantize_depth_q24(coarsest_hzb_range.y);
    hybrid_gi_scene_words[12] = SCENE_HZB_VALID_FLAG;
    hybrid_gi_scene_words[13] = 0u;
    hybrid_gi_scene_words[14] = SCENE_HZB_TILE_GRID_EXTENT;
    hybrid_gi_scene_words[15] =
        SCENE_HZB_TILE_GRID_EXTENT * SCENE_HZB_TILE_GRID_EXTENT;
}
