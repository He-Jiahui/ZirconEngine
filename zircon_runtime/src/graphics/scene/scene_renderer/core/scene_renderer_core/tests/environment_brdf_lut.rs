#[test]
fn scene_brdf_binding_is_a_read_only_generation_lease_projection() {
    let source = include_str!("../environment_brdf_lut.rs");
    let production = source.split("#[cfg(test)]").next().unwrap_or_default();

    assert!(production.contains("from_system_textures"));
    assert!(production.contains("system_textures.brdf_lut_texture().clone()"));
    assert!(production.contains("system_textures.brdf_lut_view().clone()"));
    assert!(!production.contains("create_texture"));
    assert!(!production.contains("write_texture"));
    assert!(!production.contains("wgpu::Queue"));
}
