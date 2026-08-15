fn voxel_cell_manhattan_distance(a: u32, b: u32) -> u32 {
    let a_coord = vec3<u32>(
        a % VOXEL_CLIPMAP_CELL_RESOLUTION,
        (a / VOXEL_CLIPMAP_CELL_RESOLUTION) % VOXEL_CLIPMAP_CELL_RESOLUTION,
        a / (VOXEL_CLIPMAP_CELL_RESOLUTION * VOXEL_CLIPMAP_CELL_RESOLUTION),
    );
    let b_coord = vec3<u32>(
        b % VOXEL_CLIPMAP_CELL_RESOLUTION,
        (b / VOXEL_CLIPMAP_CELL_RESOLUTION) % VOXEL_CLIPMAP_CELL_RESOLUTION,
        b / (VOXEL_CLIPMAP_CELL_RESOLUTION * VOXEL_CLIPMAP_CELL_RESOLUTION),
    );
    let delta = vec3<u32>(
        voxel_cell_axis_distance(a_coord.x, b_coord.x),
        voxel_cell_axis_distance(a_coord.y, b_coord.y),
        voxel_cell_axis_distance(a_coord.z, b_coord.z),
    );
    return delta.x + delta.y + delta.z;
}

fn voxel_cell_axis_distance(a: u32, b: u32) -> u32 {
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

    let cell_distance = voxel_cell_manhattan_distance(
        descriptor.secondary_id,
        tile_sample_id,
    );
    let cone_radius = max(1u, min(8u, descriptor.scalar3 / 32u));
    if (cell_distance > cone_radius) {
        return 0u;
    }

    let occupancy_weight = min(descriptor.tertiary_id, 16u);
    let distance_weight = cone_radius + 1u - cell_distance;
    let exact_weight = select(0u, 32u, cell_distance == 0u);
    return occupancy_weight * distance_weight + exact_weight;
}

fn voxel_cell_lookup_base(clipmap_id: u32) -> u32 {
    let clipmap_count = min(
        params.voxel_cell_lookup_clipmap_count,
        VOXEL_CELL_LOOKUP_MAX_CLIPMAPS,
    );
    for (var lookup_slot = 0u; lookup_slot < VOXEL_CELL_LOOKUP_MAX_CLIPMAPS; lookup_slot = lookup_slot + 1u) {
        if (lookup_slot >= clipmap_count) {
            break;
        }
        let base = lookup_slot * VOXEL_CELL_LOOKUP_WORDS_PER_CLIPMAP;
        if (voxel_cell_lookup[base] == clipmap_id) {
            return base;
        }
    }
    return VOXEL_CELL_LOOKUP_INVALID_DESCRIPTOR_INDEX;
}

fn dequantize_scene_prepare_position(value: u32) -> f32 {
    return (f32(i32(value)) - SCENE_PREPARE_SIGNED_POSITION_BIAS) /
        SCENE_PREPARE_POSITION_QUANTIZATION_SCALE;
}

fn voxel_cell_world_distance(
    probe_position: vec3<f32>,
    descriptor: ScenePrepareDescriptor,
) -> f32 {
    let cell_center = vec3<f32>(
        dequantize_scene_prepare_position(descriptor.scalar0),
        dequantize_scene_prepare_position(descriptor.scalar1),
        dequantize_scene_prepare_position(descriptor.scalar2),
    );
    let half_extent = f32(descriptor.scalar3) /
        SCENE_VOXEL_CELL_HALF_EXTENT_QUANTIZATION_SCALE;
    let outside = max(abs(probe_position - cell_center) - vec3<f32>(half_extent), vec3<f32>(0.0));
    return length(outside);
}

fn voxel_fallback_tile_sample(
    probe_position: vec3<f32>,
    tile_probe_id: u32,
    tile_sample_id: u32,
) -> TraceRgbSample {
    if ((params.intersection_backend_mask & TRACE_BACKEND_VOXEL_CLIPMAP) == 0u) {
        return invalid_trace_sample();
    }
    var weighted_rgb = vec3<u32>(0u);
    var total_weight = 0u;
    var weighted_distance = 0.0;
    var voxel_candidates = 0u;
    if (tile_sample_id >= VOXEL_CLIPMAP_CELL_COUNT) {
        return invalid_trace_sample();
    }
    let lookup_base = voxel_cell_lookup_base(tile_probe_id);
    if (lookup_base == VOXEL_CELL_LOOKUP_INVALID_DESCRIPTOR_INDEX) {
        return invalid_trace_sample();
    }

    for (var cell_index = 0u; cell_index < VOXEL_CLIPMAP_CELL_COUNT; cell_index = cell_index + 1u) {
        let descriptor_index = voxel_cell_lookup[lookup_base + 1u + cell_index];
        if (descriptor_index == VOXEL_CELL_LOOKUP_INVALID_DESCRIPTOR_INDEX) {
            continue;
        }
        let descriptor = scene_prepare_descriptors[descriptor_index];
        voxel_candidates = voxel_candidates + 1u;
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
        weighted_distance = weighted_distance +
            voxel_cell_world_distance(probe_position, descriptor) * f32(contribution_weight);
    }

    if (total_weight > 0u) {
        return voxel_trace_sample(
            vec3<u32>(
                (weighted_rgb.x + total_weight / 2u) / total_weight,
                (weighted_rgb.y + total_weight / 2u) / total_weight,
                (weighted_rgb.z + total_weight / 2u) / total_weight,
            ),
            weighted_distance / f32(total_weight),
            clamp(f32(total_weight) / 256.0, 0.0, 1.0),
            voxel_candidates,
        );
    }

    return invalid_trace_sample_with_cost(
        0u,
        0u,
        0u,
        voxel_candidates,
    );
}
