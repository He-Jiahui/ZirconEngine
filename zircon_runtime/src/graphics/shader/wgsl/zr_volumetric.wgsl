struct ZrVolumetricApplyParams {
    // 与 GpuVolumetricApplyParams 的字段顺序保持一致：depth 为近/远深度、切片指数、积分纹理可用位；
    // viewport 为应用区域的物理原点和尺寸倒数，片元坐标必须使用同一坐标空间。
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

// 求 zr_froxel_slice_depth 的逆映射；近/远深度与指数必须和生成积分纹理的 froxel 网格一致。
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
    // 缺少积分结果时用中性值 S=0、T=1 保持场景颜色，并在采样前跳过仅用于维持绑定的占位纹理。
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

// 积分 RGB 为累计散射辐射，A 为透射率。SSS MRT 分别衰减 diffuse/retained，且只给 retained 叠加散射，
// 因此保留独立的透射率和散射入口，普通输出再由 zr_volumetric_apply 合成。
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
