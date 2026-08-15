fn dequantize_probe_position(value: u32) -> f32 {
    return (f32(i32(value)) - 2048.0) / 64.0;
}

fn global_sdf_trace_direction(tile_sample_id: u32) -> vec3<f32> {
    let planar = surface_cache_ray_direction(tile_sample_id);
    let vertical = select(0.5, -0.5, (tile_sample_id & 1u) != 0u);
    return normalize(vec3<f32>(f32(planar.x), f32(planar.y), vertical));
}

fn invalid_global_sdf_trace_sample() -> GlobalSdfTraceSample {
    return GlobalSdfTraceSample(0.0, 0.0, 0u, 0u);
}

fn sample_global_sdf(world_position: vec3<f32>) -> GlobalSdfTraceSample {
    var selected = invalid_global_sdf_trace_sample();
    var selected_cell_size = 3.402823466e+38;
    var page_tests = 0u;
    for (var clipmap_index = 0u; clipmap_index < GLOBAL_SDF_CLIPMAP_COUNT; clipmap_index += 1u) {
        page_tests = page_tests + 1u;
        let clipmap = params.global_sdf_clipmaps[clipmap_index];
        let page_world_size = clipmap.page_world_size_and_padding.x;
        if (page_world_size <= 0.0) {
            continue;
        }
        let global_page_coordinate = vec3<i32>(floor(world_position / page_world_size));
        let local_page_coordinate = global_page_coordinate
            - clipmap.page_coordinate_origin_and_padding.xyz;
        if (any(local_page_coordinate < vec3<i32>(0)) ||
            any(local_page_coordinate >= vec3<i32>(i32(GLOBAL_SDF_PAGES_PER_EDGE)))) {
            continue;
        }
        let page_table_index = clipmap_index * GLOBAL_SDF_PAGES_PER_CLIPMAP
            + u32(local_page_coordinate.x)
            + u32(local_page_coordinate.y) * GLOBAL_SDF_PAGES_PER_EDGE
            + u32(local_page_coordinate.z) * GLOBAL_SDF_PAGES_PER_EDGE * GLOBAL_SDF_PAGES_PER_EDGE;
        let atlas_slot = global_sdf_page_table[page_table_index];
        let cell_size = page_world_size / f32(GLOBAL_SDF_PAGE_CELLS_PER_EDGE);
        if (atlas_slot == GLOBAL_SDF_PAGE_UNAVAILABLE_SLOT || cell_size >= selected_cell_size) {
            continue;
        }
        let page_world_min = vec3<f32>(global_page_coordinate) * page_world_size;
        let local_position = world_position - page_world_min;
        let cell = min(
            vec3<u32>(floor(local_position / cell_size)),
            vec3<u32>(GLOBAL_SDF_PAGE_CELLS_PER_EDGE - 1u),
        );
        let cell_index = cell.x
            + cell.y * GLOBAL_SDF_PAGE_CELLS_PER_EDGE
            + cell.z * GLOBAL_SDF_PAGE_CELLS_PER_EDGE * GLOBAL_SDF_PAGE_CELLS_PER_EDGE;
        let atlas_index = atlas_slot * GLOBAL_SDF_PAGE_VOXEL_COUNT + cell_index;
        selected = GlobalSdfTraceSample(
            bitcast<f32>(global_sdf_atlas[atlas_index]),
            cell_size,
            1u,
            0u,
        );
        selected_cell_size = cell_size;
    }
    selected.page_tests = page_tests;
    return selected;
}

fn global_sdf_tile_sample(
    probe_position: vec3<f32>,
    tile_sample_id: u32,
    ray_count: u32,
    lineage_trace_lighting_rgb: u32,
) -> TraceRgbSample {
    if ((params.intersection_backend_mask & TRACE_BACKEND_GLOBAL_SDF) == 0u ||
        params.global_sdf_page_count == 0u) {
        return invalid_trace_sample();
    }
    let direction = global_sdf_trace_direction(tile_sample_id);
    let max_distance = max(4.0, min(32.0, f32(ray_count) * 0.5));
    var trace_distance = 0.0;
    var page_tests = 0u;
    var sdf_steps = 0u;
    for (var step_index = 0u; step_index < GLOBAL_SDF_MAX_TRACE_STEPS; step_index += 1u) {
        sdf_steps = step_index + 1u;
        let sample = sample_global_sdf(probe_position + direction * trace_distance);
        page_tests = page_tests + sample.page_tests;
        if (sample.valid == 0u) {
            return invalid_trace_sample_with_cost(0u, page_tests, step_index + 1u, 0u);
        }
        if (abs(sample.distance) <= sample.cell_size * 0.75) {
            let visibility = 255u - min(176u, step_index * 11u);
            if (params.global_sdf_lighting_source == TRACE_LIGHTING_PROBE_LINEAGE &&
                lineage_trace_lighting_rgb != 0u) {
                let lineage = unpack_rgb8(lineage_trace_lighting_rgb);
                return global_sdf_trace_sample(
                    vec3<u32>(
                        (lineage.x * visibility + 127u) / 255u,
                        (lineage.y * visibility + 127u) / 255u,
                        (lineage.z * visibility + 127u) / 255u,
                    ),
                    TRACE_LIGHTING_PROBE_LINEAGE,
                    trace_distance,
                    f32(visibility) / 255.0,
                    page_tests,
                    step_index + 1u,
                );
            }
            return global_sdf_trace_sample(
                vec3<u32>(
                    32u + visibility / 3u,
                    40u + visibility / 2u,
                    52u + visibility * 2u / 3u,
                ),
                0u,
                trace_distance,
                f32(visibility) / 255.0,
                page_tests,
                step_index + 1u,
            );
        }
        trace_distance = trace_distance
            + max(abs(sample.distance), sample.cell_size * 0.5);
        if (trace_distance > max_distance) {
            break;
        }
    }
    return invalid_trace_sample_with_cost(0u, page_tests, sdf_steps, 0u);
}
