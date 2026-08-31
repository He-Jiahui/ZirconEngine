struct ZrHitProxyOutput {
    @location(0) token: u32,
    @location(1) world_position_depth: vec4<f32>,
    @location(2) world_normal: vec4<f32>,
};

fn zr_hit_proxy_vs_main_impl(v: ZrVertexInput, instance_index: u32) -> ZrVertexOutput {
    return zr_build_vertex_output(
        instance_index,
        fetch_position(v, instance_index),
        fetch_normal(v, instance_index),
        fetch_tangent(v, instance_index),
        fetch_uv0(v),
        fetch_uv1(v),
        fetch_color(v, instance_index),
    );
}

@vertex
fn zr_vs_main(v: ZrVertexInput, @builtin(instance_index) instance_index: u32) -> ZrVertexOutput {
    return zr_hit_proxy_vs_main_impl(v, instance_index);
}

@vertex
fn vs_main(v: ZrVertexInput, @builtin(instance_index) instance_index: u32) -> ZrVertexOutput {
    return zr_hit_proxy_vs_main_impl(v, instance_index);
}

fn zr_hit_proxy_output(input: ZrVertexOutput, front_facing: bool) -> ZrHitProxyOutput {
    let surface = zr_material_surface(input);
    if (zr_surface_fails_alpha_clip(surface)) {
        discard;
    }
    var output: ZrHitProxyOutput;
    output.token = zr_hit_proxy_token(input.instance_index);
    output.world_position_depth = vec4<f32>(input.position_ws, input.clip_position.z);
    output.world_normal = vec4<f32>(
        zr_raster_facing_normal(input.normal_ws, front_facing),
        0.0,
    );
    return output;
}

@fragment
fn zr_fs_main(
    input: ZrVertexOutput,
    @builtin(front_facing) front_facing: bool,
) -> ZrHitProxyOutput {
    return zr_hit_proxy_output(input, front_facing);
}

@fragment
fn fs_main(
    input: ZrVertexOutput,
    @builtin(front_facing) front_facing: bool,
) -> ZrHitProxyOutput {
    return zr_hit_proxy_output(input, front_facing);
}
