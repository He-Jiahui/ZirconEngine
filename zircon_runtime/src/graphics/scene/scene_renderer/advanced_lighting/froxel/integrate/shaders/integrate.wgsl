struct VolumetricIntegrateParams {
    grid_dimensions: vec4<u32>,
    view: ZrFroxelViewParams,
};

@group(0) @binding(0) var<uniform> params: VolumetricIntegrateParams;
@group(0) @binding(1) var froxel_scattering: texture_3d<f32>;
@group(0) @binding(2) var integrated_output: texture_storage_3d<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let grid = params.grid_dimensions.xyz;
    if (any(invocation.xy >= grid.xy)) {
        return;
    }

    var radiance = vec3<f32>(0.0);
    var transmittance = 1.0;
    for (var slice = 0u; slice < grid.z; slice += 1u) {
        let sample = textureLoad(
            froxel_scattering,
            vec3<i32>(i32(invocation.x), i32(invocation.y), i32(slice)),
            0,
        );
        let extinction = max(sample.a, 0.0);
        let step_length = zr_froxel_step_length(invocation.xy, slice, grid, params.view);
        let step_transmittance = exp(-extinction * step_length);
        var radiance_scale = step_length;
        if (extinction > 0.000001) {
            radiance_scale = (1.0 - step_transmittance) / extinction;
        }
        radiance += transmittance * max(sample.rgb, vec3<f32>(0.0)) * radiance_scale;
        transmittance *= step_transmittance;
        textureStore(
            integrated_output,
            vec3<u32>(invocation.xy, slice),
            vec4<f32>(radiance, transmittance),
        );
    }
}
