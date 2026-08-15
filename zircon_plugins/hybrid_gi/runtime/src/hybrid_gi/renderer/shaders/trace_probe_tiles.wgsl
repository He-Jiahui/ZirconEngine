struct GlobalSdfTraceClipmap {
    page_coordinate_origin_and_padding: vec4<i32>,
    page_world_size_and_padding: vec4<f32>,
};

struct ProbeTraceTileDispatchParams {
    resident_probe_count: u32,
    completed_probe_count: u32,
    tile_count: u32,
    surface_cache_texture_available: u32,
    surface_cache_atlas_width: u32,
    surface_cache_atlas_height: u32,
    surface_cache_atlas_columns: u32,
    surface_cache_tile_extent: u32,
    voxel_cell_descriptor_offset: u32,
    global_sdf_page_count: u32,
    intersection_backend_mask: u32,
    global_sdf_lighting_source: u32,
    fallback_reason: u32,
    voxel_cell_descriptor_count: u32,
    voxel_cell_lookup_clipmap_count: u32,
    _pad2: u32,
    global_sdf_clipmaps: array<GlobalSdfTraceClipmap, 4>,
};

struct ResidentProbeInput {
    probe_id: u32,
    slot: u32,
    ray_budget: u32,
    lineage_trace_support_q: u32,
    position_x_q: u32,
    position_y_q: u32,
    position_z_q: u32,
    radius_q: u32,
    previous_irradiance_rgb: u32,
    runtime_hierarchy_irradiance_rgb: u32,
    runtime_hierarchy_irradiance_weight_q: u32,
    skip_scene_prepare_for_irradiance_q: u32,
    lineage_trace_lighting_rgb: u32,
    skip_scene_prepare_for_trace_q: u32,
    parent_probe_id: u32,
    resident_ancestor_probe_id: u32,
    resident_ancestor_depth: u32,
    resident_secondary_ancestor_probe_id: u32,
    resident_secondary_ancestor_depth: u32,
    resident_tertiary_ancestor_probe_id: u32,
    resident_tertiary_ancestor_depth: u32,
    resident_quaternary_ancestor_probe_id: u32,
    resident_quaternary_ancestor_depth: u32,
};

struct PendingProbeInput {
    probe_id: u32,
    logical_index: u32,
    ray_budget: u32,
    lineage_trace_support_q: u32,
    position_x_q: u32,
    position_y_q: u32,
    position_z_q: u32,
    radius_q: u32,
    runtime_hierarchy_irradiance_rgb: u32,
    runtime_hierarchy_irradiance_weight_q: u32,
    skip_scene_prepare_for_irradiance_q: u32,
    lineage_trace_lighting_rgb: u32,
    skip_scene_prepare_for_trace_q: u32,
    parent_probe_id: u32,
    resident_ancestor_probe_id: u32,
    resident_ancestor_depth: u32,
    resident_secondary_ancestor_probe_id: u32,
    resident_secondary_ancestor_depth: u32,
    resident_tertiary_ancestor_probe_id: u32,
    resident_tertiary_ancestor_depth: u32,
    resident_quaternary_ancestor_probe_id: u32,
    resident_quaternary_ancestor_depth: u32,
};

struct ScenePrepareDescriptor {
    descriptor_kind: u32,
    primary_id: u32,
    secondary_id: u32,
    tertiary_id: u32,
    quaternary_id: u32,
    scalar0: u32,
    scalar1: u32,
    scalar2: u32,
    scalar3: u32,
    padding0: u32,
    padding1: u32,
    padding2: u32,
};

struct TraceRgbSample {
    rgb: vec3<u32>,
    valid: u32,
    intersection_source: u32,
    lighting_source: u32,
    distance: f32,
    confidence: f32,
    texture_samples: u32,
    page_tests: u32,
    sdf_steps: u32,
    voxel_candidates: u32,
};

struct ProbeTraceResult {
    rgb: u32,
    intersection_source: u32,
    lighting_source: u32,
    intersection_backend_mask: u32,
    lighting_source_mask: u32,
    distance: f32,
    confidence: f32,
    fallback_reason: u32,
    texture_samples: u32,
    page_tests: u32,
    sdf_steps: u32,
    voxel_candidates: u32,
};

struct SurfaceCacheTexelSample {
    rgb: vec3<u32>,
    depth_q: u32,
    valid: u32,
};

struct SurfaceCacheHzbDepthRange {
    min_depth_q: u32,
    max_depth_q: u32,
    valid: u32,
};

struct GlobalSdfTraceSample {
    distance: f32,
    cell_size: f32,
    valid: u32,
    page_tests: u32,
};

@group(0) @binding(0)
var<uniform> params: ProbeTraceTileDispatchParams;

@group(0) @binding(1)
var<storage, read> resident_probe_inputs: array<ResidentProbeInput>;

@group(0) @binding(2)
var<storage, read> pending_probe_updates: array<PendingProbeInput>;

@group(0) @binding(3)
var<storage, read> probe_trace_tiles: array<u32>;

@group(0) @binding(4)
var<storage, read_write> probe_trace_lighting_updates: array<u32>;

@group(0) @binding(5)
var surface_cache_atlas: texture_2d<f32>;

@group(0) @binding(6)
var surface_cache_depth: texture_2d<f32>;

@group(0) @binding(7)
var<storage, read> scene_prepare_descriptors: array<ScenePrepareDescriptor>;

@group(0) @binding(8)
var<storage, read> global_sdf_page_table: array<u32>;

@group(0) @binding(9)
var<storage, read> global_sdf_atlas: array<u32>;

@group(0) @binding(10)
var<storage, read_write> probe_trace_diagnostics: array<u32>;

@group(0) @binding(11)
var<storage, read> voxel_cell_lookup: array<u32>;

const WORDS_PER_TILE: u32 = 4u;
const SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CELL: u32 = 3u;
const VOXEL_CLIPMAP_CELL_RESOLUTION: u32 = 4u;
const VOXEL_CLIPMAP_CELL_COUNT: u32 = 64u;
const VOXEL_CELL_LOOKUP_MAX_CLIPMAPS: u32 = 8u;
const VOXEL_CELL_LOOKUP_WORDS_PER_CLIPMAP: u32 = 65u;
const VOXEL_CELL_LOOKUP_INVALID_DESCRIPTOR_INDEX: u32 = 0xffffffffu;
const GLOBAL_SDF_PAGE_CELLS_PER_EDGE: u32 = 8u;
const GLOBAL_SDF_PAGE_VOXEL_COUNT: u32 = 512u;
const GLOBAL_SDF_CLIPMAP_COUNT: u32 = 4u;
const GLOBAL_SDF_PAGES_PER_EDGE: u32 = 8u;
const GLOBAL_SDF_PAGES_PER_CLIPMAP: u32 = 512u;
const GLOBAL_SDF_PAGE_UNAVAILABLE_SLOT: u32 = 0xffffffffu;
const GLOBAL_SDF_MAX_TRACE_STEPS: u32 = 16u;
const TRACE_BACKEND_SURFACE_CACHE: u32 = 1u;
const TRACE_BACKEND_GLOBAL_SDF: u32 = 2u;
const TRACE_BACKEND_VOXEL_CLIPMAP: u32 = 4u;
const TRACE_SOURCE_SURFACE_CACHE: u32 = 1u;
const TRACE_SOURCE_GLOBAL_SDF: u32 = 2u;
const TRACE_SOURCE_VOXEL_CLIPMAP: u32 = 3u;
const TRACE_LIGHTING_SURFACE_CACHE: u32 = 1u;
const TRACE_LIGHTING_PROBE_LINEAGE: u32 = 2u;
const TRACE_LIGHTING_VOXEL_RADIANCE: u32 = 3u;
const TRACE_FALLBACK_INTERSECTION_MISS: u32 = 4u;
const TRACE_DIAGNOSTIC_WORDS_PER_ENTRY: u32 = 13u;
const SCENE_PREPARE_POSITION_QUANTIZATION_SCALE: f32 = 64.0;
const SCENE_PREPARE_SIGNED_POSITION_BIAS: f32 = 2048.0;
const SCENE_VOXEL_CELL_HALF_EXTENT_QUANTIZATION_SCALE: f32 = 64.0;

fn pack_rgb8(rgb: vec3<u32>) -> u32 {
    return min(rgb.x, 255u) | (min(rgb.y, 255u) << 8u) | (min(rgb.z, 255u) << 16u);
}

fn unpack_rgb8(packed: u32) -> vec3<u32> {
    return vec3<u32>(
        packed & 0xffu,
        (packed >> 8u) & 0xffu,
        (packed >> 16u) & 0xffu,
    );
}

fn quantize_unorm8(value: f32) -> u32 {
    return u32(clamp(value, 0.0, 1.0) * 255.0 + 0.5);
}

fn invalid_trace_sample() -> TraceRgbSample {
    return TraceRgbSample(
        vec3<u32>(0u),
        0u,
        0u,
        0u,
        3.402823466e+38,
        0.0,
        0u,
        0u,
        0u,
        0u,
    );
}

fn surface_trace_sample(
    rgb: vec3<u32>,
    distance: f32,
    confidence: f32,
    texture_samples: u32,
) -> TraceRgbSample {
    return TraceRgbSample(
        rgb,
        1u,
        TRACE_SOURCE_SURFACE_CACHE,
        TRACE_LIGHTING_SURFACE_CACHE,
        distance,
        confidence,
        texture_samples,
        0u,
        0u,
        0u,
    );
}

fn global_sdf_trace_sample(
    rgb: vec3<u32>,
    lighting_source: u32,
    distance: f32,
    confidence: f32,
    page_tests: u32,
    sdf_steps: u32,
) -> TraceRgbSample {
    return TraceRgbSample(
        rgb,
        1u,
        TRACE_SOURCE_GLOBAL_SDF,
        lighting_source,
        distance,
        confidence,
        0u,
        page_tests,
        sdf_steps,
        0u,
    );
}

fn voxel_trace_sample(
    rgb: vec3<u32>,
    distance: f32,
    confidence: f32,
    voxel_candidates: u32,
) -> TraceRgbSample {
    return TraceRgbSample(
        rgb,
        1u,
        TRACE_SOURCE_VOXEL_CLIPMAP,
        TRACE_LIGHTING_VOXEL_RADIANCE,
        distance,
        confidence,
        0u,
        0u,
        0u,
        voxel_candidates,
    );
}

fn invalid_trace_sample_with_cost(
    texture_samples: u32,
    page_tests: u32,
    sdf_steps: u32,
    voxel_candidates: u32,
) -> TraceRgbSample {
    var sample = invalid_trace_sample();
    sample.texture_samples = texture_samples;
    sample.page_tests = page_tests;
    sample.sdf_steps = sdf_steps;
    sample.voxel_candidates = voxel_candidates;
    return sample;
}

fn valid_surface_cache_texel_sample(rgb: vec3<u32>, depth_q: u32) -> SurfaceCacheTexelSample {
    return SurfaceCacheTexelSample(rgb, depth_q, 1u);
}

fn invalid_surface_cache_texel_sample() -> SurfaceCacheTexelSample {
    return SurfaceCacheTexelSample(vec3<u32>(0u), 0u, 0u);
}

fn invalid_surface_cache_hzb_depth_range() -> SurfaceCacheHzbDepthRange {
    return SurfaceCacheHzbDepthRange(0u, 0u, 0u);
}

fn fallback_tile_rgb(
    probe_id: u32,
    position_hash: u32,
    tile_id: u32,
    tile_probe_id: u32,
    tile_sample_id: u32,
    ray_count: u32,
) -> vec3<u32> {
    let tile_hash = tile_id * 17u + tile_probe_id * 31u + tile_sample_id * 43u + position_hash;
    return vec3<u32>(
        36u + ((tile_hash + probe_id * 11u) % 180u),
        28u + ((tile_hash + ray_count * 13u) % 164u),
        24u + ((tile_hash + tile_sample_id * 19u) % 156u),
    );
}

fn clamp_surface_cache_coord(coord: vec2<i32>) -> vec2<i32> {
    return vec2<i32>(
        clamp(coord.x, 0, i32(params.surface_cache_atlas_width) - 1),
        clamp(coord.y, 0, i32(params.surface_cache_atlas_height) - 1),
    );
}

fn surface_cache_sample_coord(tile_sample_id: u32) -> vec2<i32> {
    let tile_extent = max(params.surface_cache_tile_extent, 1u);
    let atlas_columns = max(params.surface_cache_atlas_columns, 1u);
    let origin = vec2<u32>(
        (tile_sample_id % atlas_columns) * tile_extent,
        (tile_sample_id / atlas_columns) * tile_extent,
    );
    let center = origin + vec2<u32>(tile_extent / 2u, tile_extent / 2u);
    return vec2<i32>(
        i32(min(center.x, params.surface_cache_atlas_width - 1u)),
        i32(min(center.y, params.surface_cache_atlas_height - 1u)),
    );
}

fn surface_cache_ray_direction(tile_sample_id: u32) -> vec2<i32> {
    let direction = tile_sample_id & 15u;
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
    if (direction == 8u) {
        return vec2<i32>(2, 1);
    }
    if (direction == 9u) {
        return vec2<i32>(1, 2);
    }
    if (direction == 10u) {
        return vec2<i32>(-1, 2);
    }
    if (direction == 11u) {
        return vec2<i32>(-2, 1);
    }
    if (direction == 12u) {
        return vec2<i32>(-2, -1);
    }
    if (direction == 13u) {
        return vec2<i32>(-1, -2);
    }
    if (direction == 14u) {
        return vec2<i32>(1, -2);
    }
    if (direction == 15u) {
        return vec2<i32>(2, -1);
    }
    return vec2<i32>(1, 0);
}

fn surface_cache_ray_step_coord(
    base_coord: vec2<i32>,
    tile_sample_id: u32,
    step_index: u32,
) -> vec2<i32> {
    let step = i32(step_index);
    return clamp_surface_cache_coord(
        base_coord + surface_cache_ray_direction(tile_sample_id) * step,
    );
}

fn surface_cache_texel_sample(coord: vec2<i32>) -> SurfaceCacheTexelSample {
    let atlas_sample = textureLoad(surface_cache_atlas, coord, 0);
    let depth_sample = textureLoad(surface_cache_depth, coord, 0);
    let depth_q = quantize_unorm8(depth_sample.r);
    if (depth_sample.a <= 0.5 || depth_q >= 255u) {
        return invalid_surface_cache_texel_sample();
    }

    let atlas_rgb = vec3<u32>(
        quantize_unorm8(atlas_sample.r),
        quantize_unorm8(atlas_sample.g),
        quantize_unorm8(atlas_sample.b),
    );
    let depth_gain = max(64u, 255u - depth_q / 2u);
    return valid_surface_cache_texel_sample(vec3<u32>(
        (atlas_rgb.x * depth_gain + 127u) / 255u,
        (atlas_rgb.y * depth_gain + 127u) / 255u,
        (atlas_rgb.z * depth_gain + 127u) / 255u,
    ), depth_q);
}

fn surface_cache_hzb_depth_range(
    coord: vec2<i32>,
    mip_level: u32,
) -> SurfaceCacheHzbDepthRange {
    let mip_stride = i32(1u << mip_level);
    let mip_dimensions = vec2<i32>(textureDimensions(surface_cache_depth, mip_level));
    let mip_coord = clamp(
        coord / mip_stride,
        vec2<i32>(0),
        mip_dimensions - vec2<i32>(1),
    );
    let depth_range = textureLoad(surface_cache_depth, mip_coord, mip_level);
    if (depth_range.a <= 0.5) {
        return invalid_surface_cache_hzb_depth_range();
    }

    let min_depth_q = quantize_unorm8(depth_range.r);
    let max_depth_q = max(min_depth_q, quantize_unorm8(depth_range.g));
    return SurfaceCacheHzbDepthRange(min_depth_q, max_depth_q, 1u);
}

fn surface_cache_coord_component_is_stride_aligned(
    coord: i32,
    direction: i32,
    stride: u32,
) -> bool {
    if (direction == 0) {
        return true;
    }
    let remainder = u32(coord) % stride;
    if (direction > 0) {
        return remainder == 0u;
    }
    return remainder + 1u == stride;
}

fn surface_cache_hzb_mip_for_step_coord(
    step_coord: vec2<i32>,
    tile_sample_id: u32,
    remaining_steps: u32,
    mip_count: u32,
) -> u32 {
    let direction = surface_cache_ray_direction(tile_sample_id);
    var mip_level = 0u;
    var stride = 1u;
    loop {
        let next_stride = stride * 2u;
        if (mip_level + 1u >= mip_count ||
            next_stride > remaining_steps) {
            break;
        }
        if (!surface_cache_coord_component_is_stride_aligned(
                step_coord.x,
                direction.x,
                next_stride,
            ) ||
            !surface_cache_coord_component_is_stride_aligned(
                step_coord.y,
                direction.y,
                next_stride,
            )) {
            break;
        }
        mip_level = mip_level + 1u;
        stride = next_stride;
    }
    return mip_level;
}

fn surface_cache_hzb_depth_range_overlaps(
    depth_range: SurfaceCacheHzbDepthRange,
    origin_depth_q: u32,
    thickness_q: u32,
) -> bool {
    if (depth_range.valid == 0u) {
        return false;
    }
    let lower_depth_q = max(origin_depth_q, thickness_q) - thickness_q;
    let upper_depth_q = min(255u, origin_depth_q + thickness_q);
    return depth_range.max_depth_q >= lower_depth_q &&
        depth_range.min_depth_q <= upper_depth_q;
}

fn surface_cache_directional_ray_sample(
    coord: vec2<i32>,
    first_sample: SurfaceCacheTexelSample,
    ray_sample_id: u32,
    ray_count: u32,
) -> TraceRgbSample {
    let max_ray_distance = max(1u, min(16u, ray_count / 4u));
    var weighted_rgb = first_sample.rgb;
    var total_weight = 1u;
    var texture_samples = 0u;

    let hierarchy_mip_count = textureNumLevels(surface_cache_depth);
    var ray_distance = 1u;
    while (ray_distance <= max_ray_distance) {
        let remaining_steps = max_ray_distance + 1u - ray_distance;
        let step_coord = surface_cache_ray_step_coord(
            coord,
            ray_sample_id,
            ray_distance,
        );
        var mip_level = surface_cache_hzb_mip_for_step_coord(
            step_coord,
            ray_sample_id,
            remaining_steps,
            hierarchy_mip_count,
        );
        loop {
            let depth_range = surface_cache_hzb_depth_range(step_coord, mip_level);
            texture_samples = texture_samples + 1u;
            let thickness_q =
                16u + min(ray_distance, 16u) * 4u + min(ray_count, 64u) / 8u;
            if (surface_cache_hzb_depth_range_overlaps(
                depth_range,
                first_sample.depth_q,
                thickness_q,
            )) {
                if (mip_level > 0u) {
                    mip_level = mip_level - 1u;
                    continue;
                }

                let step_sample = surface_cache_texel_sample(step_coord);
                texture_samples = texture_samples + 2u;
                if (step_sample.valid != 0u) {
                    let step_weight = max_ray_distance + 1u - ray_distance;
                    weighted_rgb = vec3<u32>(
                        weighted_rgb.x + step_sample.rgb.x * step_weight,
                        weighted_rgb.y + step_sample.rgb.y * step_weight,
                        weighted_rgb.z + step_sample.rgb.z * step_weight,
                    );
                    total_weight = total_weight + step_weight;
                }
                ray_distance = ray_distance + 1u;
                break;
            }

            ray_distance = ray_distance + (1u << mip_level);
            break;
        }
    }

    let depth = f32(first_sample.depth_q) / 255.0;
    return surface_trace_sample(
        vec3<u32>(
            (weighted_rgb.x + total_weight / 2u) / total_weight,
            (weighted_rgb.y + total_weight / 2u) / total_weight,
            (weighted_rgb.z + total_weight / 2u) / total_weight,
        ),
        depth,
        1.0 - depth,
        texture_samples,
    );
}

fn surface_cache_tile_sample(
    tile_sample_id: u32,
    ray_count: u32,
) -> TraceRgbSample {
    if ((params.intersection_backend_mask & TRACE_BACKEND_SURFACE_CACHE) == 0u ||
        params.surface_cache_texture_available == 0u) {
        return invalid_trace_sample();
    }

    let coord = surface_cache_sample_coord(tile_sample_id);
    let first_sample = surface_cache_texel_sample(coord);
    if (first_sample.valid == 0u) {
        return invalid_trace_sample_with_cost(2u, 0u, 0u, 0u);
    }

    let directional_ray_count = max(1u, min(16u, ray_count));
    var accumulated_rgb = vec3<u32>(0u);
    var texture_samples = 2u;
    for (var ray_index = 0u; ray_index < directional_ray_count; ray_index = ray_index + 1u) {
        let directional_sample = surface_cache_directional_ray_sample(
            coord,
            first_sample,
            tile_sample_id + ray_index,
            ray_count,
        );
        accumulated_rgb = accumulated_rgb + directional_sample.rgb;
        texture_samples = texture_samples + directional_sample.texture_samples;
    }

    let depth = f32(first_sample.depth_q) / 255.0;
    return surface_trace_sample(
        vec3<u32>(
            (accumulated_rgb.x + directional_ray_count / 2u) / directional_ray_count,
            (accumulated_rgb.y + directional_ray_count / 2u) / directional_ray_count,
            (accumulated_rgb.z + directional_ray_count / 2u) / directional_ray_count,
        ),
        depth,
        1.0 - depth,
        texture_samples,
    );
}
