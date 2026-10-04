// 此片段只提供 PBR 材质 surface 函数；模板组装层负责补充 shader 入口和绑定布局。
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
#[path = "tests/builtin_pbr_wgsl.rs"]
mod tests;
