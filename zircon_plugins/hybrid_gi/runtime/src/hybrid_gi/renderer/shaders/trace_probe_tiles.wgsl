struct ProbeTraceTileDispatchParams {
    resident_probe_count: u32,
    completed_probe_count: u32,
    tile_count: u32,
    surface_cache_texture_available: u32,
    surface_cache_atlas_width: u32,
    surface_cache_atlas_height: u32,
    surface_cache_atlas_columns: u32,
    surface_cache_tile_extent: u32,
    scene_prepare_descriptor_count: u32,
    _pad0: u32,
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

const WORDS_PER_TILE: u32 = 4u;
const SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CELL: u32 = 3u;

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

fn valid_trace_sample(rgb: vec3<u32>) -> TraceRgbSample {
    return TraceRgbSample(rgb, 1u);
}

fn invalid_trace_sample() -> TraceRgbSample {
    return TraceRgbSample(vec3<u32>(0u), 0u);
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

    return valid_trace_sample(vec3<u32>(
        (weighted_rgb.x + total_weight / 2u) / total_weight,
        (weighted_rgb.y + total_weight / 2u) / total_weight,
        (weighted_rgb.z + total_weight / 2u) / total_weight,
    ));
}

fn surface_cache_tile_sample(
    tile_sample_id: u32,
    ray_count: u32,
) -> TraceRgbSample {
    if (params.surface_cache_texture_available == 0u) {
        return invalid_trace_sample();
    }

    let coord = surface_cache_sample_coord(tile_sample_id);
    let first_sample = surface_cache_texel_sample(coord);
    if (first_sample.valid == 0u) {
        return invalid_trace_sample();
    }

    let directional_ray_count = max(1u, min(16u, ray_count));
    var accumulated_rgb = vec3<u32>(0u);
    for (var ray_index = 0u; ray_index < directional_ray_count; ray_index = ray_index + 1u) {
        let directional_sample = surface_cache_directional_ray_sample(
            coord,
            first_sample,
            tile_sample_id + ray_index,
            ray_count,
        );
        accumulated_rgb = accumulated_rgb + directional_sample.rgb;
    }

    return valid_trace_sample(vec3<u32>(
        (accumulated_rgb.x + directional_ray_count / 2u) / directional_ray_count,
        (accumulated_rgb.y + directional_ray_count / 2u) / directional_ray_count,
        (accumulated_rgb.z + directional_ray_count / 2u) / directional_ray_count,
    ));
}

fn abs_diff_u32(a: u32, b: u32) -> u32 {
    if (a >= b) {
        return a - b;
    }
    return b - a;
}

fn voxel_cell_cone_weight(
    descriptor: ScenePrepareDescriptor,
    tile_probe_id: u32,
    tile_sample_id: u32,
) -> u32 {
    if (descriptor.descriptor_kind != SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CELL) {
        return 0u;
    }
    if (descriptor.primary_id != tile_probe_id) {
        return 0u;
    }
    if (descriptor.tertiary_id == 0u || descriptor.padding1 == 0u) {
        return 0u;
    }

    let cell_distance = abs_diff_u32(descriptor.secondary_id, tile_sample_id);
    let cone_radius = max(1u, min(8u, descriptor.scalar3 / 32u));
    if (cell_distance > cone_radius) {
        return 0u;
    }

    let occupancy_weight = min(descriptor.tertiary_id, 16u);
    let distance_weight = cone_radius + 1u - cell_distance;
    let exact_weight = select(0u, 32u, cell_distance == 0u);
    return occupancy_weight * distance_weight + exact_weight;
}

fn voxel_fallback_tile_sample(tile_probe_id: u32, tile_sample_id: u32) -> TraceRgbSample {
    var weighted_rgb = vec3<u32>(0u);
    var total_weight = 0u;

    for (var descriptor_index = 0u; descriptor_index < params.scene_prepare_descriptor_count; descriptor_index = descriptor_index + 1u) {
        let descriptor = scene_prepare_descriptors[descriptor_index];
        let contribution_weight = voxel_cell_cone_weight(
            descriptor,
            tile_probe_id,
            tile_sample_id,
        );
        if (contribution_weight == 0u) {
            continue;
        }

        let descriptor_rgb = unpack_rgb8(descriptor.quaternary_id);
        weighted_rgb = vec3<u32>(
            weighted_rgb.x + descriptor_rgb.x * contribution_weight,
            weighted_rgb.y + descriptor_rgb.y * contribution_weight,
            weighted_rgb.z + descriptor_rgb.z * contribution_weight,
        );
        total_weight = total_weight + contribution_weight;
    }

    if (total_weight > 0u) {
        return valid_trace_sample(vec3<u32>(
            (weighted_rgb.x + total_weight / 2u) / total_weight,
            (weighted_rgb.y + total_weight / 2u) / total_weight,
            (weighted_rgb.z + total_weight / 2u) / total_weight,
        ));
    }

    return invalid_trace_sample();
}

fn tile_trace_rgb(
    probe_id: u32,
    ray_budget: u32,
    position_x_q: u32,
    position_y_q: u32,
    position_z_q: u32,
    lineage_trace_lighting_rgb: u32,
) -> u32 {
    var weighted_rgb = vec3<u32>(0u);
    var total_weight = 0u;
    let position_hash = position_x_q ^ (position_y_q << 3u) ^ (position_z_q << 6u);

    for (var tile_index = 0u; tile_index < params.tile_count; tile_index = tile_index + 1u) {
        let base = tile_index * WORDS_PER_TILE;
        let tile_id = probe_trace_tiles[base];
        let tile_probe_id = probe_trace_tiles[base + 1u];
        let tile_sample_id = probe_trace_tiles[base + 2u];
        let ray_count = max(probe_trace_tiles[base + 3u], 1u);
        var tile_sample = surface_cache_tile_sample(tile_sample_id, ray_count);
        if (tile_sample.valid == 0u) {
            tile_sample = voxel_fallback_tile_sample(tile_probe_id, tile_sample_id);
        }
        var tile_rgb = tile_sample.rgb;
        if (tile_sample.valid == 0u) {
            tile_rgb = fallback_tile_rgb(
                probe_id,
                position_hash,
                tile_id,
                tile_probe_id,
                tile_sample_id,
                ray_count,
            );
        }
        let weight = min(255u, 24u + min(ray_budget, 192u) / 2u + min(ray_count, 128u));
        weighted_rgb = vec3<u32>(
            weighted_rgb.x + tile_rgb.x * weight,
            weighted_rgb.y + tile_rgb.y * weight,
            weighted_rgb.z + tile_rgb.z * weight,
        );
        total_weight = total_weight + weight;
    }

    if (total_weight == 0u) {
        return lineage_trace_lighting_rgb;
    }

    let traced = pack_rgb8(vec3<u32>(
        (weighted_rgb.x + total_weight / 2u) / total_weight,
        (weighted_rgb.y + total_weight / 2u) / total_weight,
        (weighted_rgb.z + total_weight / 2u) / total_weight,
    ));

    if (lineage_trace_lighting_rgb == 0u) {
        return traced;
    }

    let lineage = unpack_rgb8(lineage_trace_lighting_rgb);
    let traced_rgb = unpack_rgb8(traced);
    return pack_rgb8(vec3<u32>(
        (traced_rgb.x * 3u + lineage.x + 2u) / 4u,
        (traced_rgb.y * 3u + lineage.y + 2u) / 4u,
        (traced_rgb.z * 3u + lineage.z + 2u) / 4u,
    ));
}

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let entry_count = params.resident_probe_count + params.completed_probe_count;
    if (index == 0u) {
        probe_trace_lighting_updates[0] = entry_count;
    }
    if (index >= entry_count || params.tile_count == 0u) {
        return;
    }

    var probe_id = 0u;
    var ray_budget = 0u;
    var position_x_q = 0u;
    var position_y_q = 0u;
    var position_z_q = 0u;
    var lineage_trace_lighting_rgb = 0u;
    if (index < params.resident_probe_count) {
        let probe = resident_probe_inputs[index];
        probe_id = probe.probe_id;
        ray_budget = probe.ray_budget;
        position_x_q = probe.position_x_q;
        position_y_q = probe.position_y_q;
        position_z_q = probe.position_z_q;
        lineage_trace_lighting_rgb = probe.lineage_trace_lighting_rgb;
    } else {
        let probe = pending_probe_updates[index - params.resident_probe_count];
        probe_id = probe.probe_id;
        ray_budget = probe.ray_budget;
        position_x_q = probe.position_x_q;
        position_y_q = probe.position_y_q;
        position_z_q = probe.position_z_q;
        lineage_trace_lighting_rgb = probe.lineage_trace_lighting_rgb;
    }

    let entry_offset = 1u + index * 2u;
    probe_trace_lighting_updates[entry_offset] = probe_id;
    probe_trace_lighting_updates[entry_offset + 1u] = tile_trace_rgb(
        probe_id,
        ray_budget,
        position_x_q,
        position_y_q,
        position_z_q,
        lineage_trace_lighting_rgb,
    );
}
