const ZR_TAA_REACTIVE_MASK_EPSILON: f32 = 0.000001;

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

fn zr_taa_reactive_mask_from_surface(surface: ZrSurfaceOutput) -> f32 {
    let alpha = clamp(surface.base_color.a, 0.0, 1.0);
    let authored_strength = clamp(surface.custom0.x, 0.0, 1.0);
    return max(alpha, authored_strength);
}

fn zr_discard_empty_taa_reactive_mask(reactive_mask: f32) {
    if (reactive_mask <= ZR_TAA_REACTIVE_MASK_EPSILON) {
        discard;
    }
}

@fragment
fn fs_taa_reactive_mask(input: ZrVertexOutput) -> @location(0) f32 {
    let surface = zr_material_surface(input);
    let reactive_mask = zr_taa_reactive_mask_from_surface(surface);
    zr_discard_empty_taa_reactive_mask(reactive_mask);
    return reactive_mask;
}

@fragment
fn fs_taa_reactive_material_mask(_input: ZrVertexOutput) -> @location(0) f32 {
    let reactive_mask = clamp(standard_material_properties.data8.x, 0.0, 1.0);
    zr_discard_empty_taa_reactive_mask(reactive_mask);
    return reactive_mask;
}
