fn zr_vs_main_impl(v: ZrVertexInput, instance_index: u32) -> ZrVertexOutput {
    return zr_build_vertex_output(
        instance_index,
        fetch_position(v, instance_index),
        fetch_normal(v, instance_index),
        fetch_tangent(v, instance_index),
        fetch_uv0(v),
        fetch_uv1(v),
        fetch_color(v),
    );
}

@vertex
fn zr_vs_main(v: ZrVertexInput, @builtin(instance_index) instance_index: u32) -> ZrVertexOutput {
    return zr_vs_main_impl(v, instance_index);
}

@vertex
fn vs_main(v: ZrVertexInput, @builtin(instance_index) instance_index: u32) -> ZrVertexOutput {
    return zr_vs_main_impl(v, instance_index);
}

fn zr_apply_alpha_clip(surface: ZrSurfaceOutput) {
    if (zr_surface_fails_alpha_clip(surface)) {
        discard;
    }
}

fn zr_fs_main_impl(input: ZrVertexOutput) {
    let surface = zr_material_surface(input);
    zr_apply_alpha_clip(surface);
}

@fragment
fn zr_fs_main(input: ZrVertexOutput) {
    zr_fs_main_impl(input);
}

@fragment
fn fs_main(input: ZrVertexOutput) {
    zr_fs_main_impl(input);
}
