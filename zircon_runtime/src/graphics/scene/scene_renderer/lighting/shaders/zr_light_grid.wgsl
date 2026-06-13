struct ZrLightGridParams {
    world_to_view: mat4x4<f32>,
    zbin_scale: f32,
    zbin_offset: f32,
    bin_count: u32,
    words_per_tile: u32,
    tile_resolution: vec2<u32>,
    tile_size_px: u32,
    light_count: u32,
    projection_mode: u32,
    _padding: vec3<u32>,
}

@group(1) @binding(20) var<uniform> zr_light_grid_params: ZrLightGridParams;
@group(1) @binding(21) var<storage, read> zr_light_zbins: array<u32>;
@group(1) @binding(22) var<storage, read> zr_light_tile_masks: array<u32>;

fn zr_light_view_z(world_position: vec3<f32>, p: ZrLightGridParams) -> f32 {
    let view_position = p.world_to_view * vec4<f32>(world_position, 1.0);
    return max(-view_position.z, 0.0001);
}

fn zr_light_zbin_index(view_z: f32, p: ZrLightGridParams) -> u32 {
    if (p.bin_count == 0u) {
        return 0u;
    }
    let scaled_z = select(log2(max(view_z, 0.0001)), view_z, p.projection_mode == 1u);
    let raw = scaled_z * p.zbin_scale + p.zbin_offset;
    return min(u32(max(raw, 0.0)), p.bin_count - 1u);
}

fn zr_light_tile_base(frag_coord: vec2<f32>, p: ZrLightGridParams) -> u32 {
    let resolution = max(p.tile_resolution, vec2<u32>(1u, 1u));
    let tile_size = max(p.tile_size_px, 1u);
    let clamped_frag = max(frag_coord, vec2<f32>(0.0, 0.0));
    let pixel = vec2<u32>(u32(clamped_frag.x), u32(clamped_frag.y));
    let tile = min(pixel / vec2<u32>(tile_size), resolution - vec2<u32>(1u, 1u));
    return (tile.y * resolution.x + tile.x) * max(p.words_per_tile, 1u);
}

fn zr_light_zbin_header(bin: u32, p: ZrLightGridParams) -> vec2<u32> {
    let stride = 2u + max(p.words_per_tile, 1u);
    let header = zr_light_zbins[bin * stride];
    let min_index = header & 0xFFFFu;
    let max_index = header >> 16u;
    return vec2<u32>(min_index, max_index);
}

fn zr_light_mask_word(tile_base: u32, bin: u32, word: u32, p: ZrLightGridParams) -> u32 {
    let stride = 2u + max(p.words_per_tile, 1u);
    let zbin_word = zr_light_zbins[bin * stride + 2u + word];
    let tile_word = zr_light_tile_masks[tile_base + word];
    return zbin_word & tile_word;
}

fn zr_light_count(frag_coord: vec2<f32>, view_z: f32, p: ZrLightGridParams) -> u32 {
    if (p.light_count == 0u || p.bin_count == 0u) {
        return 0u;
    }
    let bin = zr_light_zbin_index(view_z, p);
    let header = zr_light_zbin_header(bin, p);
    if (header.x == 0xFFFFu || header.x > header.y) {
        return 0u;
    }

    let tile_base = zr_light_tile_base(frag_coord, p);
    var count = 0u;
    for (var word = header.x / 32u; word <= header.y / 32u; word = word + 1u) {
        count = count + countOneBits(zr_light_mask_word(tile_base, bin, word, p));
    }
    return count;
}
