struct ZrFroxelViewParams {
    world_from_clip: mat4x4<f32>,
    camera_position_projection: vec4<f32>,
    camera_forward: vec4<f32>,
    depth: vec4<f32>,
};

const ZR_FROXEL_RECONSTRUCT_EPSILON: f32 = 0.000001;

fn zr_froxel_slice_depth(normalized_slice: f32, view: ZrFroxelViewParams) -> f32 {
    let near_depth = max(view.depth.x, ZR_FROXEL_RECONSTRUCT_EPSILON);
    let far_depth = max(view.depth.y, near_depth + ZR_FROXEL_RECONSTRUCT_EPSILON);
    let distribution = pow(clamp(normalized_slice, 0.0, 1.0), max(view.depth.z, 0.01));
    return near_depth * pow(far_depth / near_depth, distribution);
}

fn zr_froxel_unproject(uv: vec2<f32>, device_depth: f32, view: ZrFroxelViewParams) -> vec3<f32> {
    let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    let world_h = view.world_from_clip * vec4<f32>(ndc, device_depth, 1.0);
    return world_h.xyz / max(abs(world_h.w), ZR_FROXEL_RECONSTRUCT_EPSILON) * sign(world_h.w);
}

fn zr_froxel_world_position_at_depth(
    uv: vec2<f32>,
    view_depth: f32,
    view: ZrFroxelViewParams,
) -> vec3<f32> {
    let near_world = zr_froxel_unproject(uv, 0.0, view);
    let far_world = zr_froxel_unproject(uv, 1.0, view);
    let ray = normalize(far_world - near_world);
    let forward = normalize(view.camera_forward.xyz);
    let forward_projection = max(dot(ray, forward), ZR_FROXEL_RECONSTRUCT_EPSILON);
    if (view.camera_position_projection.w > 0.5) {
        return near_world + ray * ((view_depth - view.depth.x) / forward_projection);
    }
    return view.camera_position_projection.xyz + ray * (view_depth / forward_projection);
}

fn zr_froxel_world_position(
    invocation: vec3<u32>,
    grid: vec3<u32>,
    view: ZrFroxelViewParams,
) -> vec3<f32> {
    let uv = (vec2<f32>(invocation.xy) + vec2<f32>(0.5)) / vec2<f32>(grid.xy);
    let normalized_slice = (f32(invocation.z) + 0.5) / f32(grid.z);
    return zr_froxel_world_position_at_depth(
        uv,
        zr_froxel_slice_depth(normalized_slice, view),
        view,
    );
}

fn zr_froxel_world_position_jittered(
    invocation: vec3<u32>,
    grid: vec3<u32>,
    view: ZrFroxelViewParams,
    jitter: vec3<f32>,
) -> vec3<f32> {
    let uv = (vec2<f32>(invocation.xy) + vec2<f32>(0.5) + jitter.xy) / vec2<f32>(grid.xy);
    let normalized_slice =
        (f32(invocation.z) + 0.5 + jitter.z) / f32(grid.z);
    return zr_froxel_world_position_at_depth(
        uv,
        zr_froxel_slice_depth(normalized_slice, view),
        view,
    );
}

fn zr_froxel_step_length(
    invocation_xy: vec2<u32>,
    slice: u32,
    grid: vec3<u32>,
    view: ZrFroxelViewParams,
) -> f32 {
    let uv = (vec2<f32>(invocation_xy) + vec2<f32>(0.5)) / vec2<f32>(grid.xy);
    let start_depth = zr_froxel_slice_depth(f32(slice) / f32(grid.z), view);
    let end_depth = zr_froxel_slice_depth(f32(slice + 1u) / f32(grid.z), view);
    let start_world = zr_froxel_world_position_at_depth(uv, start_depth, view);
    let end_world = zr_froxel_world_position_at_depth(uv, end_depth, view);
    return max(distance(start_world, end_world), ZR_FROXEL_RECONSTRUCT_EPSILON);
}
