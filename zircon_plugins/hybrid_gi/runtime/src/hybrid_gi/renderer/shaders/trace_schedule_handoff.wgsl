@group(0) @binding(0)
var<storage, read> hybrid_gi_scene_words: array<u32>;

@group(0) @binding(1)
var<storage, read_write> hybrid_gi_trace_words: array<u32>;

@group(0) @binding(2)
var scene_hzb_tex: texture_2d<f32>;

const HYBRID_GI_SCENE_DEPTH_HANDOFF_MAGIC: u32 = 0x48474944u;
const HYBRID_GI_TRACE_SCHEDULE_MAGIC: u32 = 0x48474954u;
const HYBRID_GI_HZB_TRACE_MAGIC: u32 = 0x48475a42u;
const SCENE_HZB_CAMERA_PACKET_MAGIC: u32 = 0x48474943u;
const SCENE_TRACE_INPUT_MAGIC: u32 = 0x48474949u;
const DEPTH_Q24_MAX: u32 = 16777215u;
const TRACE_DEPTH_SOURCE_VALID_FLAG: u32 = 1u;
const SCENE_HZB_TILE_GRID_EXTENT: u32 = 8u;
const SCENE_HZB_TILE_WORD_OFFSET: u32 = 16u;
const SCENE_HZB_TILE_WORD_COUNT: u32 = 4u;
const SCENE_HZB_CAMERA_WORD_OFFSET: u32 = 272u;
const SCENE_HZB_VALID_FLAG: u32 = 1u << 31u;
const SCENE_TRACE_INPUT_WORD_OFFSET: u32 = 294u;
const SURFACE_CACHE_PAGE_WORD_OFFSET: u32 = 302u;
const SURFACE_CACHE_PAGE_WORD_COUNT: u32 = 8u;
const SURFACE_CACHE_PAGE_CAPACITY: u32 = 16u;
const VOXEL_CLIPMAP_WORD_OFFSET: u32 = 430u;
const VOXEL_CLIPMAP_WORD_COUNT: u32 = 6u;
const VOXEL_CLIPMAP_CAPACITY: u32 = 4u;
const VOXEL_CELL_WORD_OFFSET: u32 = 454u;
const VOXEL_CELL_WORD_COUNT: u32 = 4u;
const VOXEL_CELL_CAPACITY: u32 = 64u;
const VOXEL_CELL_RESOLUTION: u32 = 4u;
const TRACE_HZB_TILE_WORD_OFFSET: u32 = 64u;
const TRACE_HZB_TILE_WORD_COUNT: u32 = 8u;
const SCENE_NORMAL_CODE_SHIFT: u32 = 8u;
const SCENE_NORMAL_CODE_MASK: u32 = 63u;
const TRACE_HZB_TILE_HIT_FLAG: u32 = 1u << 8u;
const TRACE_HZB_TILE_RANGE_VALID_FLAG: u32 = 1u << 9u;
const TRACE_SURFACE_CACHE_HIT_FLAG: u32 = 1u << 10u;
const TRACE_VOXEL_FALLBACK_FLAG: u32 = 1u << 11u;
const TRACE_RADIANCE_VALID_FLAG: u32 = 1u << 12u;

struct HzbDepthRange {
    furthest_depth_q24: u32,
    closest_depth_q24: u32,
    valid: u32,
};

struct ScreenTraceResult {
    hit: u32,
    tile_coord: vec2<u32>,
    depth_q24: u32,
    furthest_depth_q24: u32,
    closest_depth_q24: u32,
    mip_level: u32,
    step_count: u32,
    coarse_skip_count: u32,
    range_valid: u32,
};

struct RadianceLookup {
    packed_rgba8: u32,
    valid: u32,
    source: u32,
    support_signature: u32,
};

fn depth_unorm8_from_q24(depth_q24: u32) -> u32 {
    if (depth_q24 >= DEPTH_Q24_MAX) {
        return 255u;
    }
    return min(254u, (depth_q24 * 255u + (DEPTH_Q24_MAX / 2u)) / DEPTH_Q24_MAX);
}

fn quantize_depth_q24(depth: f32) -> u32 {
    return u32(clamp(depth, 0.0, 1.0) * f32(DEPTH_Q24_MAX) + 0.5);
}

fn depth_span_q24(closest_depth_q24: u32, furthest_depth_q24: u32) -> u32 {
    return max(furthest_depth_q24, closest_depth_q24) - closest_depth_q24;
}

fn pack_depth_rgba(depth_q24: u32, valid: bool) -> u32 {
    if (!valid) {
        return 0u;
    }
    let depth_u8 = depth_unorm8_from_q24(depth_q24);
    return depth_u8 | (depth_u8 << 8u) | (depth_u8 << 16u) | (255u << 24u);
}

fn pack_hzb_trace_rgb(closest_depth_q24: u32, furthest_depth_q24: u32) -> u32 {
    let closest_u8 = depth_unorm8_from_q24(closest_depth_q24);
    let furthest_u8 = depth_unorm8_from_q24(furthest_depth_q24);
    let span_q24 = depth_span_q24(closest_depth_q24, furthest_depth_q24);
    let span_u8 = depth_unorm8_from_q24(span_q24);
    return closest_u8 |
        (furthest_u8 << 8u) |
        (span_u8 << 16u) |
        (255u << 24u);
}

fn scene_inverse_view_projection() -> mat4x4<f32> {
    let offset = SCENE_HZB_CAMERA_WORD_OFFSET + 1u;
    return mat4x4<f32>(
        vec4<f32>(
            bitcast<f32>(hybrid_gi_scene_words[offset]),
            bitcast<f32>(hybrid_gi_scene_words[offset + 1u]),
            bitcast<f32>(hybrid_gi_scene_words[offset + 2u]),
            bitcast<f32>(hybrid_gi_scene_words[offset + 3u]),
        ),
        vec4<f32>(
            bitcast<f32>(hybrid_gi_scene_words[offset + 4u]),
            bitcast<f32>(hybrid_gi_scene_words[offset + 5u]),
            bitcast<f32>(hybrid_gi_scene_words[offset + 6u]),
            bitcast<f32>(hybrid_gi_scene_words[offset + 7u]),
        ),
        vec4<f32>(
            bitcast<f32>(hybrid_gi_scene_words[offset + 8u]),
            bitcast<f32>(hybrid_gi_scene_words[offset + 9u]),
            bitcast<f32>(hybrid_gi_scene_words[offset + 10u]),
            bitcast<f32>(hybrid_gi_scene_words[offset + 11u]),
        ),
        vec4<f32>(
            bitcast<f32>(hybrid_gi_scene_words[offset + 12u]),
            bitcast<f32>(hybrid_gi_scene_words[offset + 13u]),
            bitcast<f32>(hybrid_gi_scene_words[offset + 14u]),
            bitcast<f32>(hybrid_gi_scene_words[offset + 15u]),
        ),
    );
}

fn scene_camera_position() -> vec3<f32> {
    let offset = SCENE_HZB_CAMERA_WORD_OFFSET + 17u;
    return vec3<f32>(
        bitcast<f32>(hybrid_gi_scene_words[offset]),
        bitcast<f32>(hybrid_gi_scene_words[offset + 1u]),
        bitcast<f32>(hybrid_gi_scene_words[offset + 2u]),
    );
}

fn reconstruct_world_position(tile_coord: vec2<u32>, depth_q24: u32) -> vec3<f32> {
    let uv =
        (vec2<f32>(tile_coord) + vec2<f32>(0.5)) /
        f32(SCENE_HZB_TILE_GRID_EXTENT);
    let clip_depth = f32(depth_q24) / f32(DEPTH_Q24_MAX);
    let clip = vec4<f32>(
        uv.x * 2.0 - 1.0,
        1.0 - uv.y * 2.0,
        clip_depth,
        1.0,
    );
    let world_h = scene_inverse_view_projection() * clip;
    var safe_w = world_h.w;
    if (abs(safe_w) < 0.00001) {
        safe_w = select(-0.00001, 0.00001, safe_w >= 0.0);
    }
    return world_h.xyz / safe_w;
}

fn hzb_range_overlaps_depth(
    depth_q24: u32,
    closest_depth_q24: u32,
    furthest_depth_q24: u32,
) -> bool {
    if (closest_depth_q24 > furthest_depth_q24) {
        return false;
    }
    let span_q24 = depth_span_q24(closest_depth_q24, furthest_depth_q24);
    let thickness_q24 = max(DEPTH_Q24_MAX / 128u, span_q24 / 8u);
    let lower_depth_q24 = max(depth_q24, thickness_q24) - thickness_q24;
    let upper_depth_q24 = min(DEPTH_Q24_MAX, depth_q24 + thickness_q24);
    return furthest_depth_q24 >= lower_depth_q24 &&
        closest_depth_q24 <= upper_depth_q24;
}

fn hzb_range_overlaps_ray_segment(
    start_depth_q24: u32,
    end_depth_q24: u32,
    closest_depth_q24: u32,
    furthest_depth_q24: u32,
) -> bool {
    let segment_closest_q24 = min(start_depth_q24, end_depth_q24);
    let segment_furthest_q24 = max(start_depth_q24, end_depth_q24);
    return hzb_range_overlaps_depth(
            segment_closest_q24,
            closest_depth_q24,
            furthest_depth_q24,
        ) ||
        hzb_range_overlaps_depth(
            segment_furthest_q24,
            closest_depth_q24,
            furthest_depth_q24,
        ) ||
        (closest_depth_q24 <= segment_closest_q24 &&
            furthest_depth_q24 >= segment_furthest_q24) ||
        (segment_closest_q24 <= closest_depth_q24 &&
            segment_furthest_q24 >= furthest_depth_q24);
}

fn scene_tile_depth_q24(tile_coord: vec2<u32>) -> u32 {
    let tile_index = tile_coord.y * SCENE_HZB_TILE_GRID_EXTENT + tile_coord.x;
    return hybrid_gi_scene_words[
        SCENE_HZB_TILE_WORD_OFFSET + tile_index * SCENE_HZB_TILE_WORD_COUNT
    ];
}

fn screen_ray_direction(tile_index: u32) -> vec2<i32> {
    let direction = tile_index & 7u;
    if (direction == 1u) {
        return vec2<i32>(0, 1);
    }
    if (direction == 2u) {
        return vec2<i32>(-1, 0);
    }
    if (direction == 3u) {
        return vec2<i32>(0, -1);
    }
    if (direction == 4u) {
        return vec2<i32>(1, 1);
    }
    if (direction == 5u) {
        return vec2<i32>(-1, 1);
    }
    if (direction == 6u) {
        return vec2<i32>(-1, -1);
    }
    if (direction == 7u) {
        return vec2<i32>(1, -1);
    }
    return vec2<i32>(1, 0);
}

fn steps_to_screen_edge(tile_coord: vec2<u32>, direction: vec2<i32>) -> u32 {
    var x_steps = SCENE_HZB_TILE_GRID_EXTENT - 1u;
    var y_steps = SCENE_HZB_TILE_GRID_EXTENT - 1u;
    if (direction.x > 0) {
        x_steps = SCENE_HZB_TILE_GRID_EXTENT - 1u - tile_coord.x;
    } else if (direction.x < 0) {
        x_steps = tile_coord.x;
    }
    if (direction.y > 0) {
        y_steps = SCENE_HZB_TILE_GRID_EXTENT - 1u - tile_coord.y;
    } else if (direction.y < 0) {
        y_steps = tile_coord.y;
    }
    if (direction.x == 0) {
        return y_steps;
    }
    if (direction.y == 0) {
        return x_steps;
    }
    return min(x_steps, y_steps);
}

fn screen_ray_step_coord(
    tile_coord: vec2<u32>,
    direction: vec2<i32>,
    step: u32,
) -> vec2<u32> {
    return vec2<u32>(vec2<i32>(tile_coord) + direction * i32(step));
}

fn component_is_stride_aligned(coord: u32, direction: i32, stride: u32) -> bool {
    if (direction == 0) {
        return true;
    }
    let remainder = coord % stride;
    if (direction > 0) {
        return remainder == 0u;
    }
    return remainder + 1u == stride;
}

fn hzb_skip_level_for_step(
    step_coord: vec2<u32>,
    direction: vec2<i32>,
    remaining_steps: u32,
    max_skip_level: u32,
) -> u32 {
    var skip_level = 0u;
    var stride = 1u;
    loop {
        let next_stride = stride * 2u;
        if (skip_level >= max_skip_level || next_stride > remaining_steps) {
            break;
        }
        if (!component_is_stride_aligned(step_coord.x, direction.x, next_stride) ||
            !component_is_stride_aligned(step_coord.y, direction.y, next_stride)) {
            break;
        }
        skip_level = skip_level + 1u;
        stride = next_stride;
    }
    return skip_level;
}

fn tile_footprint_hzb_mip(mip_count: u32) -> u32 {
    let scene_size = vec2<u32>(hybrid_gi_scene_words[1], hybrid_gi_scene_words[2]);
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
    let divisor = 1u << (mip_level + 1u);
    return max(
        vec2<u32>(1u),
        (scene_size + vec2<u32>(divisor - 1u)) / divisor,
    );
}

fn hzb_range_for_tile(tile_coord: vec2<u32>, mip_level: u32) -> HzbDepthRange {
    let scene_size = vec2<u32>(hybrid_gi_scene_words[1], hybrid_gi_scene_words[2]);
    let texture_size = textureDimensions(scene_hzb_tex, mip_level);
    let valid_extent = min(valid_hzb_extent(scene_size, mip_level), texture_size);
    let coord = min(
        (tile_coord * valid_extent + vec2<u32>(SCENE_HZB_TILE_GRID_EXTENT / 2u)) /
            SCENE_HZB_TILE_GRID_EXTENT,
        valid_extent - vec2<u32>(1u),
    );
    let range = textureLoad(scene_hzb_tex, vec2<i32>(coord), mip_level);
    let furthest_depth_q24 = quantize_depth_q24(range.x);
    let closest_depth_q24 = quantize_depth_q24(range.y);
    return HzbDepthRange(
        furthest_depth_q24,
        closest_depth_q24,
        select(0u, 1u, range.a > 0.5 && closest_depth_q24 <= furthest_depth_q24),
    );
}

fn interpolated_ray_depth_q24(
    start_depth_q24: u32,
    end_depth_q24: u32,
    step: u32,
    max_steps: u32,
) -> u32 {
    if (max_steps == 0u) {
        return start_depth_q24;
    }
    return (
        start_depth_q24 * (max_steps - step) +
        end_depth_q24 * step +
        max_steps / 2u
    ) / max_steps;
}

fn empty_screen_trace(tile_coord: vec2<u32>, depth_q24: u32) -> ScreenTraceResult {
    return ScreenTraceResult(0u, tile_coord, depth_q24, 0u, 0u, 0u, 0u, 0u, 0u);
}

fn trace_main_scene_hzb_ray(
    tile_coord: vec2<u32>,
    tile_index: u32,
    start_depth_q24: u32,
) -> ScreenTraceResult {
    let mip_count = textureNumLevels(scene_hzb_tex);
    if (mip_count == 0u || start_depth_q24 >= DEPTH_Q24_MAX) {
        return empty_screen_trace(tile_coord, start_depth_q24);
    }
    let direction = screen_ray_direction(tile_index);
    let max_steps = steps_to_screen_edge(tile_coord, direction);
    if (max_steps == 0u) {
        return empty_screen_trace(tile_coord, start_depth_q24);
    }
    let target_coord = screen_ray_step_coord(tile_coord, direction, max_steps);
    let target_depth_q24 = scene_tile_depth_q24(target_coord);
    let base_mip_level = tile_footprint_hzb_mip(mip_count);
    let max_skip_level = mip_count - 1u - base_mip_level;
    var ray_step = 1u;
    var step_count = 0u;
    var coarse_skip_count = 0u;
    var any_valid_range = 0u;
    var last_result = empty_screen_trace(target_coord, target_depth_q24);

    while (ray_step <= max_steps) {
        let step_coord = screen_ray_step_coord(tile_coord, direction, ray_step);
        let remaining_steps = max_steps - ray_step + 1u;
        let skip_level = hzb_skip_level_for_step(
            step_coord,
            direction,
            remaining_steps,
            max_skip_level,
        );
        var mip_level = base_mip_level + skip_level;
        loop {
            let depth_range = hzb_range_for_tile(step_coord, mip_level);
            any_valid_range = max(any_valid_range, depth_range.valid);
            let ray_depth_q24 = interpolated_ray_depth_q24(
                start_depth_q24,
                target_depth_q24,
                ray_step,
                max_steps,
            );
            var stride = 1u;
            if (mip_level > base_mip_level) {
                stride = 1u << (mip_level - base_mip_level);
            }
            let segment_end_step = min(max_steps, ray_step + stride - 1u);
            let segment_end_depth_q24 = interpolated_ray_depth_q24(
                start_depth_q24,
                target_depth_q24,
                segment_end_step,
                max_steps,
            );
            step_count = step_count + 1u;
            last_result = ScreenTraceResult(
                0u,
                step_coord,
                ray_depth_q24,
                depth_range.furthest_depth_q24,
                depth_range.closest_depth_q24,
                mip_level,
                step_count,
                coarse_skip_count,
                any_valid_range,
            );
            if (depth_range.valid != 0u && hzb_range_overlaps_ray_segment(
                    ray_depth_q24,
                    segment_end_depth_q24,
                    depth_range.closest_depth_q24,
                    depth_range.furthest_depth_q24,
                )) {
                if (mip_level > 0u) {
                    mip_level = mip_level - 1u;
                    continue;
                }
                last_result.hit = 1u;
                return last_result;
            }
            if (mip_level > base_mip_level) {
                coarse_skip_count = coarse_skip_count + 1u;
            }
            ray_step = ray_step + stride;
            break;
        }
    }
    last_result.coarse_skip_count = coarse_skip_count;
    return last_result;
}

fn invalid_radiance_lookup() -> RadianceLookup {
    return RadianceLookup(0u, 0u, 0u, 0u);
}

fn mix_support_signature(hash: u32, value: u32) -> u32 {
    let mixed = (hash ^ value) * 16777619u;
    return mixed ^ (mixed >> 16u);
}

fn support_signature_for_words(offset: u32, word_count: u32) -> u32 {
    var hash = 2166136261u;
    for (var word_index = 0u; word_index < word_count; word_index = word_index + 1u) {
        hash = mix_support_signature(hash, word_index);
        hash = mix_support_signature(hash, hybrid_gi_scene_words[offset + word_index]);
    }
    return max(hash, 1u);
}

fn trace_geometry_support_signature(trace: ScreenTraceResult) -> u32 {
    var hash = 2166136261u;
    hash = mix_support_signature(hash, trace.tile_coord.x);
    hash = mix_support_signature(hash, trace.tile_coord.y);
    hash = mix_support_signature(hash, trace.depth_q24);
    hash = mix_support_signature(hash, trace.closest_depth_q24);
    hash = mix_support_signature(hash, trace.furthest_depth_q24);
    hash = mix_support_signature(hash, trace.range_valid);
    return max(hash, 1u);
}

fn scene_trace_input_valid() -> bool {
    return hybrid_gi_scene_words[SCENE_TRACE_INPUT_WORD_OFFSET] == SCENE_TRACE_INPUT_MAGIC;
}

fn surface_cache_radiance_for_world_position(world_position: vec3<f32>) -> RadianceLookup {
    if (!scene_trace_input_valid()) {
        return invalid_radiance_lookup();
    }
    let page_count = min(
        hybrid_gi_scene_words[SCENE_TRACE_INPUT_WORD_OFFSET + 1u],
        SURFACE_CACHE_PAGE_CAPACITY,
    );
    var best_distance_ratio = 1e30;
    var result = invalid_radiance_lookup();
    for (var page_index = 0u; page_index < page_count; page_index = page_index + 1u) {
        let offset = SURFACE_CACHE_PAGE_WORD_OFFSET + page_index * SURFACE_CACHE_PAGE_WORD_COUNT;
        let packed_radiance = hybrid_gi_scene_words[offset + 3u];
        let center = vec3<f32>(
            bitcast<f32>(hybrid_gi_scene_words[offset + 4u]),
            bitcast<f32>(hybrid_gi_scene_words[offset + 5u]),
            bitcast<f32>(hybrid_gi_scene_words[offset + 6u]),
        );
        let radius = max(0.0, bitcast<f32>(hybrid_gi_scene_words[offset + 7u]));
        if (radius <= 0.0 || ((packed_radiance >> 24u) & 0xffu) == 0u) {
            continue;
        }
        let distance_ratio = length(world_position - center) / radius;
        if (distance_ratio <= 1.25 && distance_ratio < best_distance_ratio) {
            best_distance_ratio = distance_ratio;
            result = RadianceLookup(
                packed_radiance,
                1u,
                TRACE_SURFACE_CACHE_HIT_FLAG,
                support_signature_for_words(offset, SURFACE_CACHE_PAGE_WORD_COUNT),
            );
        }
    }
    return result;
}

fn voxel_radiance_for_world_position(world_position: vec3<f32>) -> RadianceLookup {
    if (!scene_trace_input_valid()) {
        return invalid_radiance_lookup();
    }
    let clipmap_count = min(
        hybrid_gi_scene_words[SCENE_TRACE_INPUT_WORD_OFFSET + 2u],
        VOXEL_CLIPMAP_CAPACITY,
    );
    var selected_clipmap_id = 0u;
    var selected_cell_id = 0u;
    var selected_half_extent = 1e30;
    var selected_clipmap_offset = 0u;
    var selected = false;
    for (var clipmap_index = 0u; clipmap_index < clipmap_count; clipmap_index = clipmap_index + 1u) {
        let offset = VOXEL_CLIPMAP_WORD_OFFSET + clipmap_index * VOXEL_CLIPMAP_WORD_COUNT;
        let center = vec3<f32>(
            bitcast<f32>(hybrid_gi_scene_words[offset + 1u]),
            bitcast<f32>(hybrid_gi_scene_words[offset + 2u]),
            bitcast<f32>(hybrid_gi_scene_words[offset + 3u]),
        );
        let half_extent = max(0.0, bitcast<f32>(hybrid_gi_scene_words[offset + 4u]));
        if (half_extent <= 0.0 || any(abs(world_position - center) > vec3<f32>(half_extent))) {
            continue;
        }
        let normalized = clamp(
            (world_position - (center - vec3<f32>(half_extent))) / (half_extent * 2.0),
            vec3<f32>(0.0),
            vec3<f32>(0.999999),
        );
        let cell_coord = vec3<u32>(normalized * f32(VOXEL_CELL_RESOLUTION));
        let cell_id = cell_coord.x +
            cell_coord.y * VOXEL_CELL_RESOLUTION +
            cell_coord.z * VOXEL_CELL_RESOLUTION * VOXEL_CELL_RESOLUTION;
        if (half_extent < selected_half_extent) {
            selected_clipmap_id = hybrid_gi_scene_words[offset];
            selected_cell_id = cell_id;
            selected_half_extent = half_extent;
            selected_clipmap_offset = offset;
            selected = true;
        }
    }
    if (!selected) {
        return invalid_radiance_lookup();
    }

    let cell_count = min(
        hybrid_gi_scene_words[SCENE_TRACE_INPUT_WORD_OFFSET + 3u],
        VOXEL_CELL_CAPACITY,
    );
    for (var cell_index = 0u; cell_index < cell_count; cell_index = cell_index + 1u) {
        let offset = VOXEL_CELL_WORD_OFFSET + cell_index * VOXEL_CELL_WORD_COUNT;
        if (hybrid_gi_scene_words[offset] == selected_clipmap_id &&
            hybrid_gi_scene_words[offset + 1u] == selected_cell_id &&
            hybrid_gi_scene_words[offset + 3u] > 0u) {
            let packed_radiance = hybrid_gi_scene_words[offset + 2u];
            if (((packed_radiance >> 24u) & 0xffu) != 0u) {
                var support_signature = support_signature_for_words(
                    selected_clipmap_offset,
                    VOXEL_CLIPMAP_WORD_COUNT,
                );
                support_signature = mix_support_signature(
                    support_signature,
                    support_signature_for_words(offset, VOXEL_CELL_WORD_COUNT),
                );
                return RadianceLookup(
                    packed_radiance,
                    1u,
                    TRACE_VOXEL_FALLBACK_FLAG,
                    max(support_signature, 1u),
                );
            }
        }
    }
    return invalid_radiance_lookup();
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= SCENE_HZB_TILE_GRID_EXTENT ||
        global_id.y >= SCENE_HZB_TILE_GRID_EXTENT ||
        global_id.z != 0u) {
        return;
    }

    let tile_coord = global_id.xy;
    let tile_index = tile_coord.y * SCENE_HZB_TILE_GRID_EXTENT + tile_coord.x;
    let scene_tile_offset =
        SCENE_HZB_TILE_WORD_OFFSET + tile_index * SCENE_HZB_TILE_WORD_COUNT;
    let depth_q24 = hybrid_gi_scene_words[scene_tile_offset];
    let scene_tile_flags = hybrid_gi_scene_words[scene_tile_offset + 3u];
    let camera_valid =
        hybrid_gi_scene_words[SCENE_HZB_CAMERA_WORD_OFFSET] == SCENE_HZB_CAMERA_PACKET_MAGIC;
    var trace = empty_screen_trace(tile_coord, depth_q24);
    if (camera_valid && (scene_tile_flags & SCENE_HZB_VALID_FLAG) != 0u) {
        trace = trace_main_scene_hzb_ray(tile_coord, tile_index, depth_q24);
    }

    var lookup_world_position = reconstruct_world_position(tile_coord, depth_q24);
    if (trace.hit != 0u) {
        let hit_depth_q24 = clamp(
            trace.depth_q24,
            trace.closest_depth_q24,
            trace.furthest_depth_q24,
        );
        trace.depth_q24 = hit_depth_q24;
        lookup_world_position = reconstruct_world_position(trace.tile_coord, hit_depth_q24);
    } else if (trace.depth_q24 < DEPTH_Q24_MAX) {
        lookup_world_position = reconstruct_world_position(trace.tile_coord, trace.depth_q24);
    }

    var radiance = invalid_radiance_lookup();
    if (trace.hit != 0u) {
        radiance = surface_cache_radiance_for_world_position(lookup_world_position);
    }
    if (radiance.valid == 0u) {
        radiance = voxel_radiance_for_world_position(lookup_world_position);
    }
    let world_distance_q8 = u32(min(
        16777215.0,
        length(lookup_world_position - scene_camera_position()) * 256.0,
    ));

    let trace_tile_offset =
        TRACE_HZB_TILE_WORD_OFFSET + tile_index * TRACE_HZB_TILE_WORD_COUNT;
    hybrid_gi_trace_words[trace_tile_offset] = radiance.packed_rgba8;
    hybrid_gi_trace_words[trace_tile_offset + 1u] = trace.depth_q24;
    hybrid_gi_trace_words[trace_tile_offset + 2u] = world_distance_q8;
    hybrid_gi_trace_words[trace_tile_offset + 3u] =
        trace.mip_level |
        select(0u, TRACE_HZB_TILE_HIT_FLAG, trace.hit != 0u) |
        select(0u, TRACE_HZB_TILE_RANGE_VALID_FLAG, trace.range_valid != 0u) |
        radiance.source |
        select(0u, TRACE_RADIANCE_VALID_FLAG, radiance.valid != 0u) |
        ((min(trace.step_count, 255u)) << 16u) |
        ((min(trace.coarse_skip_count, 255u)) << 24u);
    hybrid_gi_trace_words[trace_tile_offset + 4u] =
        trace.tile_coord.x | (trace.tile_coord.y << 16u);
    hybrid_gi_trace_words[trace_tile_offset + 5u] = pack_hzb_trace_rgb(
        trace.closest_depth_q24,
        trace.furthest_depth_q24,
    );
    hybrid_gi_trace_words[trace_tile_offset + 6u] = select(
        trace_geometry_support_signature(trace),
        radiance.support_signature,
        radiance.support_signature != 0u,
    );
    let normal_tile_index =
        trace.tile_coord.y * SCENE_HZB_TILE_GRID_EXTENT + trace.tile_coord.x;
    let normal_tile_flags = hybrid_gi_scene_words[
        SCENE_HZB_TILE_WORD_OFFSET +
        normal_tile_index * SCENE_HZB_TILE_WORD_COUNT +
        3u
    ];
    hybrid_gi_trace_words[trace_tile_offset + 7u] =
        (normal_tile_flags >> SCENE_NORMAL_CODE_SHIFT) & SCENE_NORMAL_CODE_MASK;

    if (tile_index != 0u) {
        return;
    }

    let handoff_magic = hybrid_gi_scene_words[0];
    let width = hybrid_gi_scene_words[1];
    let height = hybrid_gi_scene_words[2];
    let center_depth_q24 = hybrid_gi_scene_words[3];
    let sample_count = hybrid_gi_scene_words[4];
    let center_furthest_depth_q24 = hybrid_gi_scene_words[8];
    let center_closest_depth_q24 = hybrid_gi_scene_words[9];
    let valid_depth_source =
        handoff_magic == HYBRID_GI_SCENE_DEPTH_HANDOFF_MAGIC &&
        width > 0u &&
        height > 0u &&
        center_depth_q24 < DEPTH_Q24_MAX;
    let valid_hzb_source =
        handoff_magic == HYBRID_GI_SCENE_DEPTH_HANDOFF_MAGIC &&
        width > 0u &&
        height > 0u &&
        hybrid_gi_scene_words[12] == SCENE_HZB_VALID_FLAG &&
        center_closest_depth_q24 <= center_furthest_depth_q24 &&
        camera_valid &&
        scene_trace_input_valid();

    hybrid_gi_trace_words[0] = HYBRID_GI_TRACE_SCHEDULE_MAGIC;
    hybrid_gi_trace_words[1] = select(
        0u,
        SCENE_HZB_TILE_GRID_EXTENT * SCENE_HZB_TILE_GRID_EXTENT,
        valid_hzb_source,
    );
    hybrid_gi_trace_words[2] = width;
    hybrid_gi_trace_words[3] = height;
    hybrid_gi_trace_words[4] = center_depth_q24;
    hybrid_gi_trace_words[5] = sample_count;
    hybrid_gi_trace_words[6] = radiance.packed_rgba8;
    hybrid_gi_trace_words[7] = select(0u, TRACE_DEPTH_SOURCE_VALID_FLAG, valid_depth_source);
    hybrid_gi_trace_words[8] =
        depth_span_q24(center_closest_depth_q24, center_furthest_depth_q24);
    hybrid_gi_trace_words[9] = handoff_magic;
    hybrid_gi_trace_words[10] = HYBRID_GI_HZB_TRACE_MAGIC;
    hybrid_gi_trace_words[11] = hybrid_gi_scene_words[5];
    hybrid_gi_trace_words[12] = hybrid_gi_scene_words[6];
    hybrid_gi_trace_words[13] = hybrid_gi_scene_words[7];
    hybrid_gi_trace_words[14] = center_furthest_depth_q24;
    hybrid_gi_trace_words[15] = center_closest_depth_q24;
    hybrid_gi_trace_words[16] = hybrid_gi_scene_words[10];
    hybrid_gi_trace_words[17] = hybrid_gi_scene_words[11];
    hybrid_gi_trace_words[18] =
        depth_span_q24(center_closest_depth_q24, center_furthest_depth_q24);
    hybrid_gi_trace_words[19] = select(0u, 1u, valid_hzb_source);
    hybrid_gi_trace_words[20] = SCENE_HZB_TILE_GRID_EXTENT;
    hybrid_gi_trace_words[21] =
        SCENE_HZB_TILE_GRID_EXTENT * SCENE_HZB_TILE_GRID_EXTENT;
    hybrid_gi_trace_words[22] =
        hybrid_gi_scene_words[SCENE_HZB_CAMERA_WORD_OFFSET];
    for (var camera_word = 1u; camera_word < 22u; camera_word = camera_word + 1u) {
        hybrid_gi_trace_words[22u + camera_word] =
            hybrid_gi_scene_words[SCENE_HZB_CAMERA_WORD_OFFSET + camera_word];
    }
    hybrid_gi_trace_words[44] = TRACE_HZB_TILE_WORD_OFFSET;
    hybrid_gi_trace_words[45] = TRACE_HZB_TILE_WORD_COUNT;
    hybrid_gi_trace_words[46] = hybrid_gi_scene_words[SCENE_TRACE_INPUT_WORD_OFFSET + 1u];
    hybrid_gi_trace_words[47] = hybrid_gi_scene_words[SCENE_TRACE_INPUT_WORD_OFFSET + 2u];
    hybrid_gi_trace_words[48] = hybrid_gi_scene_words[SCENE_TRACE_INPUT_WORD_OFFSET + 3u];
    hybrid_gi_trace_words[49] = hybrid_gi_scene_words[SCENE_TRACE_INPUT_WORD_OFFSET + 7u];
}
