const BUILTIN_PBR_MATERIAL_SURFACE_WGSL: &str = r#"
fn zr_material_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    var surface = zr_surface_from_base_color(input.tint * input.color);
    surface.normal_ws = zr_normalize_or_zero(input.normal_ws);
    return surface;
}
"#;

pub(in crate::asset::pipeline::manager) const fn builtin_pbr_wgsl() -> &'static str {
    BUILTIN_PBR_MATERIAL_SURFACE_WGSL
}

#[cfg(test)]
mod tests {
    use super::builtin_pbr_wgsl;

    const MAX_BUILTIN_PBR_SURFACE_SOURCE_BYTES: usize = 512;

    #[test]
    fn builtin_pbr_shader_is_a_template_material_function() {
        let shader = builtin_pbr_wgsl();

        assert_eq!(shader.matches("fn zr_material_surface(").count(), 1);
        assert!(shader.contains("input: ZrVertexOutput) -> ZrSurfaceOutput"));
        assert!(shader.contains("zr_surface_from_base_color(input.tint * input.color)"));
        assert!(shader.contains("surface.normal_ws = zr_normalize_or_zero(input.normal_ws)"));
        assert!(!shader.contains("@vertex"));
        assert!(!shader.contains("@fragment"));
        assert!(!shader.contains("@group("));
    }

    #[test]
    fn builtin_pbr_shader_keeps_fallback_asset_source_bounded() {
        assert!(builtin_pbr_wgsl().len() <= MAX_BUILTIN_PBR_SURFACE_SOURCE_BYTES);
    }
}
