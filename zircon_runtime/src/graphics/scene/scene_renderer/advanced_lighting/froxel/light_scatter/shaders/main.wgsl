fn punctual_visibility(light: ZrGpuLightData, light_type: u32, world_position: vec3<f32>, distance_to_light: f32) -> f32 {
    let range = max(light.position_range.w, VOLUMETRIC_EPSILON);
    if (distance_to_light >= range) {
        return 0.0;
    }
    var visibility = pow(clamp(1.0 - distance_to_light / range, 0.0, 1.0), 2.0);
    let light_to_froxel = normalize_or_zero(world_position - light.position_range.xyz);
    if (light_type == ZR_GPU_LIGHT_TYPE_SPOT) {
        let cone = dot(normalize_or_zero(light.direction_type.xyz), light_to_froxel);
        visibility *= clamp(
            (cone - light.spot_angles_size.y)
                / max(light.spot_angles_size.x - light.spot_angles_size.y, VOLUMETRIC_EPSILON),
            0.0,
            1.0,
        );
    } else if (light_type == ZR_GPU_LIGHT_TYPE_RECT) {
        visibility *= max(dot(normalize_or_zero(light.direction_type.xyz), light_to_froxel), 0.0);
    }
    return visibility;
}

fn scatter_light(light_index: u32, world_position: vec3<f32>, view_direction: vec3<f32>, view_z: f32) -> vec3<f32> {
    if (light_index >= min(params.grid_and_light_count.w, arrayLength(&zr_light_data))) {
        return vec3<f32>(0.0);
    }
    let light = zr_light_data[light_index];
    if (light.cookie_misc.z == 0u) {
        return vec3<f32>(0.0);
    }
    let light_type = zr_gpu_light_type(light);
    let radiance = max(light.color_intensity.rgb, vec3<f32>(0.0)) * max(light.color_intensity.w, 0.0);
    var incoming = vec3<f32>(0.0);
    var visibility = 1.0;
    if (light_type == ZR_GPU_LIGHT_TYPE_DIRECTIONAL) {
        incoming = normalize_or_zero(-light.direction_type.xyz);
    } else {
        let to_light = light.position_range.xyz - world_position;
        let distance_to_light = length(to_light);
        incoming = normalize_or_zero(to_light);
        visibility = punctual_visibility(light, light_type, world_position, distance_to_light);
    }
    if (visibility <= 0.0 || length(incoming) <= VOLUMETRIC_EPSILON) {
        return vec3<f32>(0.0);
    }
    visibility *= zr_gpu_light_shadow_visibility(light, light_type, world_position, view_z);
    let phase = henyey_greenstein(params.phase_and_ambient.x, dot(incoming, -view_direction));
    return radiance * visibility * phase;
}

fn temporal_scattering(current: vec4<f32>, world_position: vec3<f32>, grid: vec3<u32>) -> vec4<f32> {
    let history_weight = params.temporal.jitter_and_history.w;
    if (history_weight <= 0.0) {
        return current;
    }
    let previous_clip = params.temporal.previous_clip_from_world * vec4<f32>(world_position, 1.0);
    if (previous_clip.w <= VOLUMETRIC_EPSILON) {
        return current;
    }
    let previous_ndc = previous_clip.xy / previous_clip.w;
    let previous_uv = vec2<f32>(
        previous_ndc.x * 0.5 + 0.5,
        0.5 - previous_ndc.y * 0.5,
    );
    let previous_view_depth = dot(
        world_position - params.temporal.previous_camera_position.xyz,
        normalize_or_zero(params.temporal.previous_camera_forward.xyz),
    );
    let near_depth = max(params.temporal.previous_depth.x, VOLUMETRIC_EPSILON);
    let far_depth = max(params.temporal.previous_depth.y, near_depth + VOLUMETRIC_EPSILON);
    let logarithmic_depth = log(max(previous_view_depth / near_depth, 1.0))
        / log(far_depth / near_depth);
    let previous_slice = pow(
        clamp(logarithmic_depth, 0.0, 1.0),
        1.0 / max(params.temporal.previous_depth.z, 0.01),
    );
    let previous_coord = vec3<f32>(previous_uv, previous_slice);
    if (any(previous_coord < vec3<f32>(0.0)) || any(previous_coord >= vec3<f32>(1.0))) {
        return current;
    }
    let history_coord = min(
        vec3<u32>(previous_coord * vec3<f32>(grid)),
        grid - vec3<u32>(1u),
    );
    let history = textureLoad(previous_froxel_scattering, vec3<i32>(history_coord), 0);
    let extinction_threshold = max(0.02, current.a * 0.25);
    if (abs(history.a - current.a) > extinction_threshold) {
        return current;
    }
    return vec4<f32>(mix(current.rgb, history.rgb, history_weight), current.a);
}

@compute @workgroup_size(4, 4, 4)
fn cs_main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let grid = params.grid_and_light_count.xyz;
    if (any(invocation >= grid)) {
        return;
    }

    let normalized = (vec3<f32>(invocation) + vec3<f32>(0.5)) / vec3<f32>(grid);
    let world_position = zr_froxel_world_position_jittered(
        invocation,
        grid,
        params.view,
        params.temporal.jitter_and_history.xyz,
    );
    let view_direction = normalize_or_zero(params.view.camera_position_projection.xyz - world_position);
    let view_z = zr_light_view_z(world_position, zr_light_grid_params);
    let frag_coord = normalized.xy * vec2<f32>(params.viewport_size.xy);
    let bin = zr_light_zbin_index(view_z, zr_light_grid_params);
    let header = zr_light_zbin_header(bin, zr_light_grid_params);
    let tile_base = zr_light_tile_base(frag_coord, zr_light_grid_params);
    var lighting = max(params.phase_and_ambient.yzw, vec3<f32>(0.0));
    if (header.x != 0xFFFFu && header.x <= header.y) {
        for (var word = header.x / 32u; word <= header.y / 32u; word += 1u) {
            var mask = zr_light_mask_word(tile_base, bin, word, zr_light_grid_params);
            while (mask != 0u) {
                let bit_index = firstTrailingBit(mask);
                lighting += scatter_light(word * 32u + bit_index, world_position, view_direction, view_z);
                mask &= mask - 1u;
            }
        }
    }
    let media = textureLoad(froxel_media, vec3<i32>(invocation), 0);
    let current = vec4<f32>(media.rgb * lighting, media.a);
    textureStore(froxel_scattering, invocation, temporal_scattering(current, world_position, grid));
}
