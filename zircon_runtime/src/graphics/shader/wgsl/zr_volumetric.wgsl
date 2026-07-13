struct ZrVolumetricApplyParams {
    depth: vec4<f32>,
    viewport: vec4<f32>,
};

@group(1) @binding(25) var<uniform> zr_volumetric_apply_params: ZrVolumetricApplyParams;
@group(1) @binding(26) var zr_volumetric_integrated: texture_3d<f32>;
@group(1) @binding(27) var zr_volumetric_sampler: sampler;

const ZR_VOLUMETRIC_APPLY_EPSILON: f32 = 0.000001;

fn zr_volumetric_unproject(uv: vec2<f32>, device_depth: f32) -> vec3<f32> {
    let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    let world_h = scene.inverse_view_proj * vec4<f32>(ndc, device_depth, 1.0);
    let safe_w = select(
        -ZR_VOLUMETRIC_APPLY_EPSILON,
        ZR_VOLUMETRIC_APPLY_EPSILON,
        world_h.w >= 0.0,
    );
    return world_h.xyz / select(safe_w, world_h.w, abs(world_h.w) > ZR_VOLUMETRIC_APPLY_EPSILON);
}

fn zr_volumetric_uv(fragment_position: vec2<f32>) -> vec2<f32> {
    return clamp(
        (fragment_position - zr_volumetric_apply_params.viewport.xy) *
            zr_volumetric_apply_params.viewport.zw,
        vec2<f32>(0.0),
        vec2<f32>(1.0),
    );
}

fn zr_volumetric_slice_coordinate(uv: vec2<f32>, device_depth: f32) -> f32 {
    let world_position = zr_volumetric_unproject(uv, clamp(device_depth, 0.0, 1.0));
    let camera_forward = normalize(-scene.camera_view_direction.xyz);
    let view_depth = max(
        dot(world_position - scene.camera_world_position.xyz, camera_forward),
        zr_volumetric_apply_params.depth.x,
    );
    let near_depth = max(zr_volumetric_apply_params.depth.x, ZR_VOLUMETRIC_APPLY_EPSILON);
    let far_depth = max(
        zr_volumetric_apply_params.depth.y,
        near_depth + ZR_VOLUMETRIC_APPLY_EPSILON,
    );
    let exponential_depth = clamp(
        log(view_depth / near_depth) / log(far_depth / near_depth),
        0.0,
        1.0,
    );
    return pow(
        exponential_depth,
        1.0 / max(zr_volumetric_apply_params.depth.z, 0.01),
    );
}

fn zr_volumetric_integrated_sample(
    fragment_position: vec2<f32>,
    device_depth: f32,
) -> vec4<f32> {
    if (zr_volumetric_apply_params.depth.w < 0.5) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    let uv = zr_volumetric_uv(fragment_position);
    return textureSampleLevel(
        zr_volumetric_integrated,
        zr_volumetric_sampler,
        vec3<f32>(uv, zr_volumetric_slice_coordinate(uv, device_depth)),
        0.0,
    );
}

fn zr_volumetric_transmittance(fragment_position: vec2<f32>, device_depth: f32) -> f32 {
    return clamp(zr_volumetric_integrated_sample(fragment_position, device_depth).a, 0.0, 1.0);
}

fn zr_volumetric_scattering(fragment_position: vec2<f32>, device_depth: f32) -> vec3<f32> {
    return max(
        zr_volumetric_integrated_sample(fragment_position, device_depth).rgb,
        vec3<f32>(0.0),
    );
}

fn zr_volumetric_apply(
    color: vec3<f32>,
    fragment_position: vec2<f32>,
    device_depth: f32,
) -> vec3<f32> {
    return color * zr_volumetric_transmittance(fragment_position, device_depth)
        + zr_volumetric_scattering(fragment_position, device_depth);
}
