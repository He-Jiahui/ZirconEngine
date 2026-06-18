struct ColorNeighborhood {
    minimum: vec3<f32>,
    maximum: vec3<f32>,
};

const SSR_HIT_REFINE_STEPS: u32 = 4u;

fn load_scene_normal(coord: vec2<i32>, viewport_size: vec2<u32>) -> vec3<f32> {
    let max_coord = vec2<i32>(viewport_size - vec2<u32>(1u, 1u));
    let clamped = clamp(coord, vec2<i32>(0, 0), max_coord);
    let encoded = textureLoad(scene_normal_tex, physical_coord_i32(clamped), 0).rgb;
    if (max(encoded.x, max(encoded.y, encoded.z)) <= 0.001) {
        return vec3<f32>(0.0, 0.0, 1.0);
    }
    let decoded = encoded * 2.0 - vec3<f32>(1.0, 1.0, 1.0);
    let normal_length = length(decoded);
    if (normal_length <= 0.001) {
        return vec3<f32>(0.0, 0.0, 1.0);
    }
    return decoded / normal_length;
}

fn world_normal_to_view_space(world_normal: vec3<f32>) -> vec3<f32> {
    let view_normal = vec3<f32>(
        dot(params.effect_view_x.xyz, world_normal),
        dot(params.effect_view_y.xyz, world_normal),
        dot(params.effect_view_z.xyz, world_normal)
    );
    let normal_length = length(view_normal);
    if (normal_length <= 0.001) {
        return vec3<f32>(0.0, 0.0, 1.0);
    }
    return view_normal / normal_length;
}

fn load_scene_material_roughness(coord: vec2<i32>, viewport_size: vec2<u32>) -> f32 {
    let max_coord = vec2<i32>(viewport_size - vec2<u32>(1u, 1u));
    let clamped = clamp(coord, vec2<i32>(0, 0), max_coord);
    let material = textureLoad(scene_material_tex, physical_coord_i32(clamped), 0).rgb;
    if (max(material.r, max(material.g, material.b)) <= 0.001) {
        return 1.0;
    }
    return clamp(max(material.g, 0.04), 0.04, 1.0);
}

fn load_screen_space_reflection_ambient_occlusion(
    coord: vec2<i32>,
    viewport_size: vec2<u32>
) -> f32 {
    let max_coord = vec2<i32>(viewport_size - vec2<u32>(1u, 1u));
    let clamped = clamp(coord, vec2<i32>(0, 0), max_coord);
    return clamp(
        textureLoad(ambient_occlusion_tex, physical_coord_i32(clamped), 0).r,
        0.0,
        1.0
    );
}

fn screen_space_reflection_specular_occlusion_factors(
    coord: vec2<i32>,
    viewport_size: vec2<u32>,
    current_depth: f32,
    roughness: f32
) -> vec2<f32> {
    let ambient_occlusion = load_screen_space_reflection_ambient_occlusion(coord, viewport_size);
    let roughness_response = 1.0 - smoothstep(0.35, 0.9, roughness);
    let depth_response = 1.0 - smoothstep(0.78, 1.0, normalized_view_depth(current_depth));
    let occlusion_response = clamp(roughness_response * depth_response, 0.0, 1.0);
    return vec2<f32>(ambient_occlusion, occlusion_response);
}

fn load_screen_space_reflection_specular_occlusion(
    coord: vec2<i32>,
    viewport_size: vec2<u32>,
    traced_visibility: f32
) -> f32 {
    let max_coord = vec2<i32>(viewport_size - vec2<u32>(1u, 1u));
    let clamped = clamp(coord, vec2<i32>(0, 0), max_coord);
    let factors = textureLoad(screen_space_reflection_specular_occlusion_tex, clamped, 0).rg;
    return mix(
        1.0,
        clamp(factors.r, 0.0, 1.0),
        clamp(factors.g, 0.0, 1.0) * clamp(traced_visibility, 0.0, 1.0)
    );
}

fn resolve_screen_space_reflection_specular_occlusion(
    coord: vec2<u32>,
    viewport_size: vec2<u32>
) -> vec4<f32> {
    let coord_i32 = vec2<i32>(coord);
    let roughness = load_scene_material_roughness(coord_i32, viewport_size);
    let current_depth = load_scene_view_depth(coord_i32, viewport_size);
    let factors = screen_space_reflection_specular_occlusion_factors(
        coord_i32,
        viewport_size,
        current_depth,
        roughness
    );
    return vec4<f32>(
        factors.r,
        factors.g,
        mix(1.0, factors.r, factors.g),
        1.0
    );
}

fn screen_space_reflection_depth_pyramid_size(viewport_size: vec2<u32>) -> vec2<u32> {
    return (viewport_size + vec2<u32>(1u, 1u)) / vec2<u32>(2u, 2u);
}

fn screen_space_reflection_depth_pyramid_coarse_size(viewport_size: vec2<u32>) -> vec2<u32> {
    let pyramid_size = screen_space_reflection_depth_pyramid_size(viewport_size);
    return (pyramid_size + vec2<u32>(1u, 1u)) / vec2<u32>(2u, 2u);
}

fn screen_space_reflection_downsampled_size(source_size: vec2<u32>) -> vec2<u32> {
    return max(
        (source_size + vec2<u32>(1u, 1u)) / vec2<u32>(2u, 2u),
        vec2<u32>(1u, 1u)
    );
}

fn screen_space_reflection_mip_coord(coord: vec2<i32>, mip_level: u32) -> vec2<u32> {
    let safe_coord = vec2<u32>(max(coord, vec2<i32>(0, 0)));
    let scale = 1u << min(mip_level + 1u, 30u);
    return safe_coord / vec2<u32>(scale, scale);
}

fn load_screen_space_reflection_depth_pyramid_cell(
    pyramid_coord: vec2<u32>,
    viewport_size: vec2<u32>
) -> vec2<f32> {
    let pyramid_size = screen_space_reflection_depth_pyramid_size(viewport_size);
    let safe_coord = min(pyramid_coord, pyramid_size - vec2<u32>(1u, 1u));
    let range = textureLoad(
        screen_space_reflection_depth_pyramid_tex,
        vec2<i32>(safe_coord),
        0
    ).rg;
    return vec2<f32>(min(range.x, range.y), max(range.x, range.y));
}

fn load_screen_space_reflection_depth_pyramid_cell_at_mip(
    pyramid_coord: vec2<u32>,
    mip_level: u32
) -> vec2<f32> {
    let pyramid_size = max(
        textureDimensions(screen_space_reflection_depth_pyramid_tex, mip_level),
        vec2<u32>(1u, 1u)
    );
    let safe_coord = min(pyramid_coord, pyramid_size - vec2<u32>(1u, 1u));
    let range = textureLoad(
        screen_space_reflection_depth_pyramid_tex,
        vec2<i32>(safe_coord),
        mip_level
    ).rg;
    return vec2<f32>(min(range.x, range.y), max(range.x, range.y));
}

fn load_screen_space_reflection_depth_pyramid_source_cell(
    pyramid_coord: vec2<u32>
) -> vec2<f32> {
    let source_size = max(
        textureDimensions(screen_space_reflection_depth_pyramid_tex),
        vec2<u32>(1u, 1u)
    );
    let safe_coord = min(pyramid_coord, source_size - vec2<u32>(1u, 1u));
    let range = textureLoad(
        screen_space_reflection_depth_pyramid_tex,
        vec2<i32>(safe_coord),
        0
    ).rg;
    return vec2<f32>(min(range.x, range.y), max(range.x, range.y));
}

fn load_screen_space_reflection_depth_pyramid(
    coord: vec2<i32>,
    viewport_size: vec2<u32>
) -> vec2<f32> {
    let pyramid_size = screen_space_reflection_depth_pyramid_size(viewport_size);
    let safe_coord = vec2<u32>(max(coord, vec2<i32>(0, 0)));
    let pyramid_coord =
        min(safe_coord / vec2<u32>(2u, 2u), pyramid_size - vec2<u32>(1u, 1u));
    return load_screen_space_reflection_depth_pyramid_cell(pyramid_coord, viewport_size);
}

fn load_screen_space_reflection_depth_pyramid_mip(
    coord: vec2<i32>,
    mip_level: u32
) -> vec2<f32> {
    let mip_count = textureNumLevels(screen_space_reflection_depth_pyramid_tex);
    let safe_mip = min(mip_level, max(mip_count, 1u) - 1u);
    let pyramid_coord = screen_space_reflection_mip_coord(coord, safe_mip);
    return load_screen_space_reflection_depth_pyramid_cell_at_mip(pyramid_coord, safe_mip);
}

fn load_screen_space_reflection_depth_pyramid_coarse(
    coord: vec2<i32>,
    viewport_size: vec2<u32>
) -> vec2<f32> {
    if (textureNumLevels(screen_space_reflection_depth_pyramid_tex) > 1u) {
        return load_screen_space_reflection_depth_pyramid_mip(coord, 1u);
    }
    return load_screen_space_reflection_depth_pyramid_mip(coord, 0u);
}

fn screen_space_reflection_depth_pyramid_trace_mip(
    roughness: f32,
    ray_distance: f32,
    max_view_distance: f32
) -> u32 {
    let mip_count = textureNumLevels(screen_space_reflection_depth_pyramid_tex);
    if (mip_count <= 1u) {
        return 0u;
    }
    let distance_factor = clamp(ray_distance / max(max_view_distance, 0.001), 0.0, 1.0);
    let biased_roughness = clamp(roughness + params.effect_ssr_limits.w, 0.0, 1.0);
    let roughness_factor = smoothstep(0.2, 0.95, biased_roughness);
    let selected = u32(floor(max(distance_factor, roughness_factor) * f32(mip_count - 1u)));
    return min(selected, mip_count - 1u);
}

fn screen_space_reflection_depth_pyramid_visibility(
    ray_depth: f32,
    depth_range: vec2<f32>
) -> f32 {
    if (depth_range.y <= 0.0) {
        return 1.0;
    }
    let thickness = max(params.effect_dither_ssr.w, 0.0001);
    let miss_distance = max(max(depth_range.x - ray_depth, ray_depth - depth_range.y), 0.0);
    return 1.0 - smoothstep(
        thickness,
        thickness * max(params.effect_ssr_limits.y, 2.0),
        miss_distance
    );
}

fn resolve_screen_space_reflection_depth_pyramid(
    pyramid_coord: vec2<u32>,
    viewport_size: vec2<u32>
) -> vec4<f32> {
    let base_coord = vec2<i32>(pyramid_coord * vec2<u32>(2u, 2u));
    let depth_00 = load_scene_view_depth(base_coord, viewport_size);
    let depth_10 = load_scene_view_depth(base_coord + vec2<i32>(1, 0), viewport_size);
    let depth_01 = load_scene_view_depth(base_coord + vec2<i32>(0, 1), viewport_size);
    let depth_11 = load_scene_view_depth(base_coord + vec2<i32>(1, 1), viewport_size);
    let min_depth = min(min(depth_00, depth_10), min(depth_01, depth_11));
    let max_depth = max(max(depth_00, depth_10), max(depth_01, depth_11));
    return vec4<f32>(min_depth, max_depth, max_depth - min_depth, 1.0);
}

fn resolve_screen_space_reflection_depth_pyramid_coarse(
    pyramid_coord: vec2<u32>,
    viewport_size: vec2<u32>
) -> vec4<f32> {
    let base_coord = pyramid_coord * vec2<u32>(2u, 2u);
    let range_00 = load_screen_space_reflection_depth_pyramid_source_cell(base_coord);
    let range_10 = load_screen_space_reflection_depth_pyramid_source_cell(
        base_coord + vec2<u32>(1u, 0u)
    );
    let range_01 = load_screen_space_reflection_depth_pyramid_source_cell(
        base_coord + vec2<u32>(0u, 1u)
    );
    let range_11 = load_screen_space_reflection_depth_pyramid_source_cell(
        base_coord + vec2<u32>(1u, 1u)
    );
    let min_depth = min(min(range_00.x, range_10.x), min(range_01.x, range_11.x));
    let max_depth = max(max(range_00.y, range_10.y), max(range_01.y, range_11.y));
    return vec4<f32>(min_depth, max_depth, max_depth - min_depth, 1.0);
}

fn screen_space_reflection_reflection_pyramid_size(viewport_size: vec2<u32>) -> vec2<u32> {
    return (viewport_size + vec2<u32>(1u, 1u)) / vec2<u32>(2u, 2u);
}

fn screen_space_reflection_reflection_pyramid_coarse_size(viewport_size: vec2<u32>) -> vec2<u32> {
    let pyramid_size = screen_space_reflection_reflection_pyramid_size(viewport_size);
    return (pyramid_size + vec2<u32>(1u, 1u)) / vec2<u32>(2u, 2u);
}

fn load_screen_space_reflection_reflection_pyramid_cell(
    pyramid_coord: vec2<u32>,
    viewport_size: vec2<u32>
) -> vec3<f32> {
    let pyramid_size = screen_space_reflection_reflection_pyramid_size(viewport_size);
    let safe_coord = min(pyramid_coord, pyramid_size - vec2<u32>(1u, 1u));
    return textureLoad(
        screen_space_reflection_reflection_pyramid_tex,
        vec2<i32>(safe_coord),
        0
    ).rgb;
}

fn load_screen_space_reflection_reflection_pyramid_cell_at_mip(
    pyramid_coord: vec2<u32>,
    mip_level: u32
) -> vec3<f32> {
    let pyramid_size = max(
        textureDimensions(screen_space_reflection_reflection_pyramid_tex, mip_level),
        vec2<u32>(1u, 1u)
    );
    let safe_coord = min(pyramid_coord, pyramid_size - vec2<u32>(1u, 1u));
    return textureLoad(
        screen_space_reflection_reflection_pyramid_tex,
        vec2<i32>(safe_coord),
        mip_level
    ).rgb;
}

fn load_screen_space_reflection_reflection_pyramid_source_cell(
    pyramid_coord: vec2<u32>
) -> vec3<f32> {
    let source_size = max(
        textureDimensions(screen_space_reflection_reflection_pyramid_tex),
        vec2<u32>(1u, 1u)
    );
    let safe_coord = min(pyramid_coord, source_size - vec2<u32>(1u, 1u));
    return textureLoad(
        screen_space_reflection_reflection_pyramid_tex,
        vec2<i32>(safe_coord),
        0
    ).rgb;
}

fn load_screen_space_reflection_reflection_pyramid(
    coord: vec2<i32>,
    viewport_size: vec2<u32>
) -> vec3<f32> {
    let pyramid_size = screen_space_reflection_reflection_pyramid_size(viewport_size);
    let safe_coord = vec2<u32>(max(coord, vec2<i32>(0, 0)));
    let pyramid_coord =
        min(safe_coord / vec2<u32>(2u, 2u), pyramid_size - vec2<u32>(1u, 1u));
    return load_screen_space_reflection_reflection_pyramid_cell(pyramid_coord, viewport_size);
}

fn load_screen_space_reflection_reflection_pyramid_mip(
    coord: vec2<i32>,
    mip_level: u32
) -> vec3<f32> {
    let mip_count = textureNumLevels(screen_space_reflection_reflection_pyramid_tex);
    let safe_mip = min(mip_level, max(mip_count, 1u) - 1u);
    let pyramid_coord = screen_space_reflection_mip_coord(coord, safe_mip);
    return load_screen_space_reflection_reflection_pyramid_cell_at_mip(
        pyramid_coord,
        safe_mip
    );
}

fn load_screen_space_reflection_reflection_pyramid_coarse(
    coord: vec2<i32>,
    viewport_size: vec2<u32>
) -> vec3<f32> {
    if (textureNumLevels(screen_space_reflection_reflection_pyramid_tex) > 1u) {
        return load_screen_space_reflection_reflection_pyramid_mip(coord, 1u);
    }
    let source_size = screen_space_reflection_reflection_pyramid_size(viewport_size);
    let target_size = screen_space_reflection_downsampled_size(source_size);
    let coarse_coord = min(
        screen_space_reflection_mip_coord(coord, 1u),
        target_size - vec2<u32>(1u, 1u)
    );
    return textureLoad(
        screen_space_reflection_reflection_pyramid_coarse_tex,
        vec2<i32>(coarse_coord),
        0
    ).rgb;
}

fn screen_space_reflection_reflection_pyramid_rough_mip(roughness: f32) -> u32 {
    let mip_count = textureNumLevels(screen_space_reflection_reflection_pyramid_tex);
    if (mip_count <= 1u) {
        return 0u;
    }
    let biased_roughness = clamp(roughness + params.effect_ssr_limits.w, 0.0, 1.0);
    let selected = u32(floor(smoothstep(0.18, 1.0, biased_roughness) * f32(mip_count - 1u)));
    return min(selected, mip_count - 1u);
}

fn resolve_screen_space_reflection_reflection_pyramid(
    pyramid_coord: vec2<u32>,
    viewport_size: vec2<u32>
) -> vec4<f32> {
    let base_coord = vec2<i32>(pyramid_coord * vec2<u32>(2u, 2u));
    let color_00 = load_scene_rgb(base_coord, viewport_size);
    let color_10 = load_scene_rgb(base_coord + vec2<i32>(1, 0), viewport_size);
    let color_01 = load_scene_rgb(base_coord + vec2<i32>(0, 1), viewport_size);
    let color_11 = load_scene_rgb(base_coord + vec2<i32>(1, 1), viewport_size);
    let average_color = (color_00 + color_10 + color_01 + color_11) * 0.25;
    return vec4<f32>(average_color, 1.0);
}

fn resolve_screen_space_reflection_reflection_pyramid_coarse(
    pyramid_coord: vec2<u32>,
    viewport_size: vec2<u32>
) -> vec4<f32> {
    let base_coord = pyramid_coord * vec2<u32>(2u, 2u);
    let color_00 =
        load_screen_space_reflection_reflection_pyramid_source_cell(base_coord);
    let color_10 = load_screen_space_reflection_reflection_pyramid_source_cell(
        base_coord + vec2<u32>(1u, 0u)
    );
    let color_01 = load_screen_space_reflection_reflection_pyramid_source_cell(
        base_coord + vec2<u32>(0u, 1u)
    );
    let color_11 = load_screen_space_reflection_reflection_pyramid_source_cell(
        base_coord + vec2<u32>(1u, 1u)
    );
    let average_color = (color_00 + color_10 + color_01 + color_11) * 0.25;
    return vec4<f32>(average_color, 1.0);
}

fn coord_to_screen_uv(coord: vec2<u32>, viewport_size: vec2<u32>) -> vec2<f32> {
    return (vec2<f32>(coord) + vec2<f32>(0.5, 0.5)) / vec2<f32>(viewport_size);
}

fn screen_uv_to_ndc(uv: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
}

fn reconstruct_view_position(coord: vec2<u32>, viewport_size: vec2<u32>, view_depth: f32) -> vec3<f32> {
    let ndc = screen_uv_to_ndc(coord_to_screen_uv(coord, viewport_size));
    let safe_depth = max(view_depth, params.effect_depth.x);
    if (params.effect_depth.w > 0.5) {
        return vec3<f32>(
            ndc.x * safe_depth / max(params.effect_projection.x, 0.001),
            ndc.y * safe_depth / max(params.effect_projection.y, 0.001),
            -safe_depth
        );
    }
    return vec3<f32>(
        ndc.x * max(params.effect_projection.z, 0.001),
        ndc.y * max(params.effect_projection.w, 0.001),
        -safe_depth
    );
}

fn project_view_position_to_pixel(view_position: vec3<f32>, viewport_size: vec2<u32>) -> vec2<f32> {
    var ndc = vec2<f32>(0.0, 0.0);
    if (params.effect_depth.w > 0.5) {
        let safe_depth = max(-view_position.z, params.effect_depth.x);
        ndc = vec2<f32>(
            view_position.x * params.effect_projection.x / safe_depth,
            view_position.y * params.effect_projection.y / safe_depth
        );
    } else {
        ndc = vec2<f32>(
            view_position.x / max(params.effect_projection.z, 0.001),
            view_position.y / max(params.effect_projection.w, 0.001)
        );
    }
    return vec2<f32>(
        (ndc.x * 0.5 + 0.5) * f32(viewport_size.x) - 0.5,
        (0.5 - ndc.y * 0.5) * f32(viewport_size.y) - 0.5
    );
}

fn screen_edge_fade(sample_position: vec2<f32>, viewport_size: vec2<u32>) -> f32 {
    let uv = (sample_position + vec2<f32>(0.5, 0.5)) / vec2<f32>(viewport_size);
    let edge_distance = min(min(uv.x, uv.y), min(1.0 - uv.x, 1.0 - uv.y));
    return smoothstep(0.0, 0.08, edge_distance);
}

fn screen_space_reflection_hit_visibility(
    ray_depth: f32,
    sample_depth: f32,
    current_depth: f32,
    ray_distance: f32,
    max_view_distance: f32,
    sample_position: vec2<f32>,
    viewport_size: vec2<u32>
) -> f32 {
    let thickness = max(params.effect_dither_ssr.w, 0.0001);
    let thickness_window = max(thickness, ray_depth * 0.01);
    let depth_visibility =
        1.0
        - smoothstep(
            thickness_window,
            thickness_window * max(params.effect_ssr_limits.y, 2.0),
            abs(sample_depth - ray_depth)
        );
    let behind_origin = step(
        current_depth + thickness * 0.25,
        sample_depth
    );
    let distance_visibility =
        1.0 - clamp(ray_distance / max(max_view_distance, 0.001), 0.0, 1.0);
    return depth_visibility
        * behind_origin
        * distance_visibility
        * screen_edge_fade(sample_position, viewport_size);
}

fn sample_screen_space_reflection_hit(
    ray_position: vec3<f32>,
    current_depth: f32,
    ray_distance: f32,
    max_view_distance: f32,
    roughness: f32,
    viewport_size: vec2<u32>
) -> vec4<f32> {
    let ray_depth = -ray_position.z;
    if (ray_depth <= params.effect_depth.x || ray_depth >= params.effect_depth.y) {
        return vec4<f32>(0.0, 0.0, 0.0, -1.0);
    }

    let sample_position = project_view_position_to_pixel(ray_position, viewport_size);
    if (
        sample_position.x < 0.0
        || sample_position.y < 0.0
        || sample_position.x >= f32(viewport_size.x)
        || sample_position.y >= f32(viewport_size.y)
    ) {
        return vec4<f32>(0.0, 0.0, 0.0, -1.0);
    }

    let sample_coord = vec2<i32>(round(sample_position));
    let sample_depth = load_scene_view_depth(sample_coord, viewport_size);
    let depth_pyramid_range =
        load_screen_space_reflection_depth_pyramid_mip(sample_coord, 0u);
    let trace_mip = screen_space_reflection_depth_pyramid_trace_mip(
        roughness,
        ray_distance,
        max_view_distance
    );
    let depth_pyramid_coarse_range =
        load_screen_space_reflection_depth_pyramid_mip(sample_coord, trace_mip);
    let depth_pyramid_visibility =
        screen_space_reflection_depth_pyramid_visibility(ray_depth, depth_pyramid_range);
    let depth_pyramid_coarse_visibility =
        screen_space_reflection_depth_pyramid_visibility(ray_depth, depth_pyramid_coarse_range);
    let visibility = screen_space_reflection_hit_visibility(
        ray_depth,
        sample_depth,
        current_depth,
        ray_distance,
        max_view_distance,
        sample_position,
        viewport_size
    ) * depth_pyramid_visibility * depth_pyramid_coarse_visibility;
    return vec4<f32>(
        f32(sample_coord.x),
        f32(sample_coord.y),
        sample_depth - ray_depth,
        visibility
    );
}

fn refine_screen_space_reflection_hit(
    view_origin: vec3<f32>,
    ray_direction: vec3<f32>,
    previous_distance: f32,
    hit_distance: f32,
    current_depth: f32,
    max_view_distance: f32,
    roughness: f32,
    viewport_size: vec2<u32>
) -> vec4<f32> {
    var lower_distance = max(previous_distance, 0.0);
    var upper_distance = max(hit_distance, lower_distance + 0.0001);
    var best_hit = sample_screen_space_reflection_hit(
        view_origin + ray_direction * upper_distance,
        current_depth,
        upper_distance,
        max_view_distance,
        roughness,
        viewport_size
    );
    var best_error = abs(best_hit.z);

    for (var refine_index = 0u; refine_index < SSR_HIT_REFINE_STEPS; refine_index = refine_index + 1u) {
        let candidate_distance = (lower_distance + upper_distance) * 0.5;
        let candidate_hit = sample_screen_space_reflection_hit(
            view_origin + ray_direction * candidate_distance,
            current_depth,
            candidate_distance,
            max_view_distance,
            roughness,
            viewport_size
        );

        if (candidate_hit.w < 0.0) {
            upper_distance = candidate_distance;
            continue;
        }
        if (candidate_hit.w > 0.01 && abs(candidate_hit.z) < best_error) {
            best_hit = candidate_hit;
            best_error = abs(candidate_hit.z);
        }

        if (
            (candidate_hit.z > 0.0 && ray_direction.z < 0.0)
            || (candidate_hit.z <= 0.0 && ray_direction.z >= 0.0)
        ) {
            lower_distance = candidate_distance;
        } else {
            upper_distance = candidate_distance;
        }
    }

    return best_hit;
}

fn trace_screen_space_reflection(
    coord: vec2<u32>,
    viewport_size: vec2<u32>,
    view_origin: vec3<f32>,
    reflected_direction: vec3<f32>,
    roughness: f32
) -> vec4<f32> {
    let max_steps = min(params.effect_flags.z, 128u);
    if (max_steps == 0u) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let reflected_direction_length = length(reflected_direction);
    if (reflected_direction_length <= 0.001) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let ray_direction = reflected_direction / reflected_direction_length;
    let max_view_distance = max(params.effect_ssr_limits.x, 1.0);
    let step_distance = max(max_view_distance / f32(max_steps), params.effect_depth.x * 0.25);
    let current_depth = -view_origin.z;

    var hit_color = vec3<f32>(0.0, 0.0, 0.0);
    var hit_visibility = 0.0;
    var previous_distance = 0.0;
    for (var step_index = 1u; step_index <= max_steps; step_index = step_index + 1u) {
        let ray_distance = step_distance * f32(step_index);
        let ray_position = view_origin + ray_direction * ray_distance;
        let hit = sample_screen_space_reflection_hit(
            ray_position,
            current_depth,
            ray_distance,
            max_view_distance,
            roughness,
            viewport_size
        );
        if (hit.w < 0.0) {
            break;
        }

        if (hit.w > 0.01) {
            let refined_hit = refine_screen_space_reflection_hit(
                view_origin,
                ray_direction,
                previous_distance,
                ray_distance,
                current_depth,
                max_view_distance,
                roughness,
                viewport_size
            );
            var resolved_hit = hit;
            if (refined_hit.w > 0.01) {
                resolved_hit = refined_hit;
            }
            let hit_coord = vec2<i32>(i32(resolved_hit.x), i32(resolved_hit.y));
            let exact_hit_color = load_scene_rgb(hit_coord, viewport_size);
            let pyramid_hit_color =
                load_screen_space_reflection_reflection_pyramid(hit_coord, viewport_size);
            let coarse_pyramid_hit_color =
                load_screen_space_reflection_reflection_pyramid_mip(
                    hit_coord,
                    screen_space_reflection_reflection_pyramid_rough_mip(roughness)
                );
            let rough_reflection_weight = smoothstep(0.18, 0.75, roughness);
            let coarse_reflection_weight = smoothstep(0.55, 1.0, roughness);
            hit_color = mix(
                mix(exact_hit_color, pyramid_hit_color, rough_reflection_weight),
                coarse_pyramid_hit_color,
                coarse_reflection_weight
            );
            hit_visibility = resolved_hit.w;
            break;
        }
        previous_distance = ray_distance;
    }

    return vec4<f32>(hit_color, hit_visibility);
}

fn scene_rgb_neighborhood(coord: vec2<i32>, viewport_size: vec2<u32>) -> ColorNeighborhood {
    let center = load_scene_rgb(coord, viewport_size);
    var bounds = ColorNeighborhood(center, center);

    for (var y: i32 = -1; y <= 1; y = y + 1) {
        for (var x: i32 = -1; x <= 1; x = x + 1) {
            let sample_rgb = load_scene_rgb(coord + vec2<i32>(x, y), viewport_size);
            bounds.minimum = min(bounds.minimum, sample_rgb);
            bounds.maximum = max(bounds.maximum, sample_rgb);
        }
    }

    return bounds;
}

fn reproject_ssr_history_coord(
    coord: vec2<u32>,
    viewport_size: vec2<u32>,
    motion_vector: vec2<f32>
) -> vec2<f32> {
    return vec2<f32>(coord) - motion_vector * vec2<f32>(viewport_size);
}

fn sample_reprojected_ssr_history(
    coord: vec2<u32>,
    viewport_size: vec2<u32>,
    motion_vector: vec2<f32>
) -> vec4<f32> {
    if (params.feature_flags.z == 0u) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let history_pixel = reproject_ssr_history_coord(coord, viewport_size, motion_vector);
    if (
        history_pixel.x < 0.0
        || history_pixel.y < 0.0
        || history_pixel.x >= f32(viewport_size.x)
        || history_pixel.y >= f32(viewport_size.y)
    ) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let history_coord = vec2<i32>(
        clamp(
            round(history_pixel),
            vec2<f32>(0.0, 0.0),
            vec2<f32>(viewport_size) - vec2<f32>(1.0, 1.0)
        )
    );
    let neighborhood = scene_rgb_neighborhood(vec2<i32>(coord), viewport_size);
    let history = textureLoad(history_screen_space_reflection_tex, history_coord, 0);
    return vec4<f32>(
        clamp(history.rgb, neighborhood.minimum, neighborhood.maximum),
        clamp(history.a, 0.0, 1.0)
    );
}

fn ssr_temporal_blend_weight(
    motion_vector: vec2<f32>,
    traced_visibility: f32,
    roughness: f32,
    viewport_size: vec2<u32>
) -> f32 {
    let pixel_motion = length(motion_vector * vec2<f32>(viewport_size));
    let motion_stability = 1.0 - smoothstep(8.0, 32.0, pixel_motion);
    let roughness_stability = 1.0 - smoothstep(0.55, 0.95, roughness);
    let temporal_blend_max = clamp(params.effect_ssr_limits.z, 0.0, 1.0);
    return clamp(
        temporal_blend_max * traced_visibility * motion_stability * roughness_stability,
        0.0,
        temporal_blend_max
    );
}

fn resolve_screen_space_reflection_history(
    coord: vec2<u32>,
    viewport_size: vec2<u32>
) -> vec4<f32> {
    let intensity = params.effect_dither_ssr.z;
    if (intensity <= 0.001 || params.effect_flags.z == 0u) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    let coord_i32 = vec2<i32>(coord);
    let normal = world_normal_to_view_space(load_scene_normal(coord_i32, viewport_size));
    let roughness = load_scene_material_roughness(coord_i32, viewport_size);
    let current_depth = load_scene_view_depth(coord_i32, viewport_size);
    let view_position = reconstruct_view_position(coord, viewport_size, current_depth);
    let view_direction = normalize(view_position);
    let reflected_direction = reflect(view_direction, normal);
    let traced_reflection = trace_screen_space_reflection(
        coord,
       viewport_size,
       view_position,
       reflected_direction,
       roughness
   );
    let motion_vector = load_motion_vector_neighbor_max(coord_i32, viewport_size);
    let temporal_history = sample_reprojected_ssr_history(coord, viewport_size, motion_vector);
    let temporal_weight = ssr_temporal_blend_weight(
        motion_vector,
        traced_reflection.a * temporal_history.a,
        roughness,
        viewport_size
    );
    let reflection_rgb = mix(traced_reflection.rgb, temporal_history.rgb, temporal_weight);
    let roughness_visibility = 1.0 - smoothstep(0.45, 1.0, roughness);
    let specular_occlusion = load_screen_space_reflection_specular_occlusion(
        coord_i32,
        viewport_size,
        traced_reflection.a
    );
    let reflection_visibility = clamp(
        intensity * traced_reflection.a * roughness_visibility * specular_occlusion * 0.18,
        0.0,
        0.35
    );
    return vec4<f32>(reflection_rgb, reflection_visibility);
}

@fragment
fn fs_screen_space_reflection_depth_pyramid(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let viewport_size = viewport_size();
    let pyramid_size = screen_space_reflection_depth_pyramid_size(viewport_size);
    let pyramid_coord = min(vec2<u32>(position.xy), pyramid_size - vec2<u32>(1u, 1u));
    return resolve_screen_space_reflection_depth_pyramid(pyramid_coord, viewport_size);
}

@fragment
fn fs_screen_space_reflection_depth_pyramid_coarse(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let source_size = max(
        textureDimensions(screen_space_reflection_depth_pyramid_tex),
        vec2<u32>(1u, 1u)
    );
    let target_size = screen_space_reflection_downsampled_size(source_size);
    let pyramid_coord = min(vec2<u32>(position.xy), target_size - vec2<u32>(1u, 1u));
    return resolve_screen_space_reflection_depth_pyramid_coarse(
        pyramid_coord,
        viewport_size()
    );
}

@fragment
fn fs_screen_space_reflection_reflection_pyramid(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let viewport_size = viewport_size();
    let pyramid_size = screen_space_reflection_reflection_pyramid_size(viewport_size);
    let pyramid_coord = min(vec2<u32>(position.xy), pyramid_size - vec2<u32>(1u, 1u));
    return resolve_screen_space_reflection_reflection_pyramid(pyramid_coord, viewport_size);
}

@fragment
fn fs_screen_space_reflection_reflection_pyramid_coarse(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let source_size = max(
        textureDimensions(screen_space_reflection_reflection_pyramid_tex),
        vec2<u32>(1u, 1u)
    );
    let target_size = screen_space_reflection_downsampled_size(source_size);
    let pyramid_coord = min(vec2<u32>(position.xy), target_size - vec2<u32>(1u, 1u));
    return resolve_screen_space_reflection_reflection_pyramid_coarse(
        pyramid_coord,
        viewport_size()
    );
}

@fragment
fn fs_screen_space_reflection_specular_occlusion(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let viewport_size = viewport_size();
    let coord = min(vec2<u32>(position.xy), viewport_size - vec2<u32>(1u, 1u));
    return resolve_screen_space_reflection_specular_occlusion(coord, viewport_size);
}
