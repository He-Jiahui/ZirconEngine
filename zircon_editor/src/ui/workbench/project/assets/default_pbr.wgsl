fn zr_material_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    var surface = zr_surface_from_base_color(
        zr_mat_base_color() * zr_sample_base_color(input.uv0) * input.tint * input.color,
    );
    surface.normal_ws = zr_normalize_or_zero(input.normal_ws);
    surface.shading_model_id = 2u;
    return surface;
}
